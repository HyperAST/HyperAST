use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::AddAssign;
use std::ops::Index;
use std::ops::SubAssign;

use num::ToPrimitive;
use tree_sitter::QueryProperty;

use super::MAX_STEP_CAPTURE_COUNT;
use super::Query;
use super::TextPredicateCapture;
use super::indexed::CaptureId;
use super::indexed::StepId;
use crate::CaptureQuantifier;
use crate::Language;
use crate::PATTERN_DONE_MARKER;
use crate::PatternId;
use crate::Precomps;
use crate::QueryError;
use crate::QueryErrorKind;
use crate::ffi;
use crate::indexed;
use crate::indexed::PredStepId;
use crate::predicate::PerPatternBuilder;
use crate::predicate_error;
use crate::utils::Array;
use crate::utils::ArrayStr;
use crate::utils::SafeUpcast;

type SmallDepth = u16;

#[derive(Clone, Debug)]
pub(crate) struct QueryPattern {
    steps: crate::Slice<StepId>,
    predicate_steps: crate::Slice<indexed::PredStepId>,
    start_byte: u32,
    end_byte: u32,
    is_non_local: bool,
}
impl QueryPattern {
    pub(crate) fn adapt(mut self, offset: StepId, byte_offset: u32) -> QueryPattern {
        self.steps.offset += offset;
        self.start_byte += byte_offset;
        self.end_byte += byte_offset;
        if self.predicate_steps.length != PredStepId::new(0) {
            todo!() // Not even sure if predicate_steps matter at this point
        }
        self
    }

    fn is_empty(&self) -> bool {
        self.steps.length == StepId::new(1)
    }
}

impl From<&crate::ffi_extra::QueryPattern> for QueryPattern {
    fn from(value: &crate::ffi_extra::QueryPattern) -> Self {
        // let offset = indexed::StepId::new(value.offset as u16);
        // let length = indexed::StepId::new(value.length as u16);
        let f = |x| indexed::StepId::new(x as u16);
        let g = |x| indexed::PredStepId::new(x as u16);
        Self {
            steps: super::Slice::new(f(value.steps.offset), f(value.steps.length)),
            predicate_steps: super::Slice::new(
                g(value.predicate_steps.offset),
                g(value.predicate_steps.length),
            ),
            start_byte: value.start_byte,
            end_byte: value.end_byte,
            is_non_local: value.is_non_local,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PatternEntry {
    pub(crate) step_index: StepId,
    pub(crate) pattern_index: PatternId,
    pub(crate) is_rooted: bool,
    pub(crate) precomputed: Precomps,
}
impl PatternEntry {
    pub(crate) fn precomputed(&self) -> Option<Precomps> {
        (self.precomputed != 0).then_some(self.precomputed)
    }
}

impl From<&crate::ffi_extra::TSPatternEntry> for PatternEntry {
    fn from(value: &crate::ffi_extra::TSPatternEntry) -> Self {
        Self {
            step_index: StepId::new(value.step_index),
            pattern_index: PatternId::new(value.pattern_index.to()),
            is_rooted: value.is_rooted,
            precomputed: Default::default(),
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct QueryStep {
    // optional when done variant
    symbol: ffi::TSSymbol,
    // optional
    supertype_symbol: ffi::TSSymbol,
    // optional
    pub(crate) field: ffi::TSFieldId,
    // optional
    capture_ids: [CaptureId; MAX_STEP_CAPTURE_COUNT],
    // done variant marker
    pub(crate) depth: SmallDepth,
    // optional, madatory when dead_end or pass_through
    alternative_index: StepId,
    // optional
    negated_field_list_id: u16, // TODO use a custom id
    /// bitfield corresponding to the 9 following flags
    bit_field: u16,
    // is_named: bool,
    // is_immediate: bool,
    // is_last_child: bool,
    // is_pass_through: bool,
    // is_dead_end: bool,
    // alternative_is_immediate: bool,
    // contains_captures: bool,
    // root_pattern_guaranteed: bool,
    // parent_pattern_guaranteed: bool,
}

impl QueryStep {
    pub(crate) fn capture_ids(&self) -> impl Iterator<Item = CaptureId> + '_ {
        self.capture_ids
            .iter()
            .take_while(|c| **c != super::indexed::CaptureId::NONE)
            .copied()
    }
    pub(crate) fn has_capture_ids(&self) -> bool {
        self.capture_ids[0] != CaptureId::NONE
    }

    pub(crate) fn done(&self) -> bool {
        self.depth == PATTERN_DONE_MARKER
    }

    pub(crate) fn depth(&self) -> u32 {
        assert!(!self.done());
        self.depth as u32
    }

    pub(crate) fn alternative_index(&self) -> Option<StepId> {
        (self.alternative_index != StepId::NONE).then_some(self.alternative_index)
    }

    pub(crate) fn negated_field_list_id(&self) -> Option<u16> {
        (self.negated_field_list_id > 0).then_some(self.negated_field_list_id)
    }

    pub(crate) fn supertype_symbol(&self) -> Option<crate::Symbol> {
        use crate::Symbol;
        (Symbol::from(self.supertype_symbol) != Symbol::END)
            .then_some(Symbol::from(self.supertype_symbol))
    }
    pub(crate) fn is_wildcard(&self) -> bool {
        use crate::Symbol;
        Symbol::from(self.symbol) == Symbol::WILDCARD_SYMBOL
    }

    pub(crate) fn is(&self, symbol: crate::Symbol) -> bool {
        use crate::Symbol;
        symbol == Symbol::from(self.symbol)
    }

    pub(crate) fn field(&self) -> Option<ffi::TSFieldId> {
        (self.field > 0).then_some(self.field)
    }

    pub(crate) fn constrained(&self, field: ffi::TSFieldId) -> bool {
        !self.field > 0 || field == self.field
    }

    pub(crate) fn immediate_pred(&self) -> Option<u16> {
        self.has_immediate_pred().then_some(
            if self.capture_ids[1] == super::indexed::CaptureId::NONE {
                self.capture_ids[2].0
            } else {
                if cfg!(debug_assertions) {
                    log::error!("{:?}", self);
                }
                return None;
                // unreachable!()
            },
        )
    }

    pub(crate) fn set_immediate_pred(&mut self, i: u32) -> bool {
        if self.has_immediate_pred() {
            return true;
        }
        assert_eq!(self.capture_ids[1], CaptureId::NONE);
        self.capture_ids[2] = CaptureId::new(i);
        self.bit_field |= StepFlags::is_immediate_pred;
        self.bit_field &=
            !(StepFlags::root_pattern_guaranteed | StepFlags::parent_pattern_guaranteed);
        false
    }

    fn normed_alternative_index(&self, stepid: StepId) -> Option<i16> {
        self.alternative_index().map(|x| x.sub(stepid))
    }

    fn remap_negative_fields(&mut self, neg_map: &[u16]) {
        let Some(i) = self.negated_field_list_id() else {
            return;
        };
        todo!();
    }

    fn remap_imm_pred(&mut self, imm_pred_offset: usize) {
        if !self.has_immediate_pred() {
            return;
        }
        if self.capture_ids[1] == super::indexed::CaptureId::NONE {
            assert!(imm_pred_offset < u16::MAX as usize);
            self.capture_ids[2].0 = self.capture_ids[2]
                .0
                .checked_add(imm_pred_offset as u16)
                .unwrap();
        } else {
            unreachable!()
        }
    }
}

impl QueryStep {
    pub fn adapt(mut self, old: StepId, new: StepId) -> QueryStep {
        if self.alternative_index().is_some() {
            self.alternative_index.sub_assign(old);
            self.alternative_index.add_assign(new);
        }
        self
    }

    fn remap_captures(&mut self, map: &[CaptureId]) {
        for c in &mut self.capture_ids {
            if *c == CaptureId::NONE {
                break;
            }
            *c = map[c.to_usize()];
        }
    }
}

impl StepId {
    fn sub(&self, o: Self) -> i16 {
        self.0 as i16 - o.0 as i16
    }
}

struct PosedQueryStep(StepId, QueryStep);
impl std::hash::Hash for PosedQueryStep {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.0.hash(state);
        let s = &self.1;
        s.is_dead_end().hash(state);
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct PrecomputedPatterns {
    map: Vec<(u64, PatternId)>,
    intermediate_hashes: Vec<u64>,
    max_sub_len: u16,
}

/// WIP genericise PrecomputedPatterns for testing purposes
struct SubFinder<I, T, const INTERM: u16> {
    map: Vec<(T, I)>,
    intermediate_hashes: Vec<u64>,
    max_sub_len: u16,
}

impl<I, T, const INTERM: u16> SubFinder<I, T, INTERM> {
    pub(crate) fn register_subs<Q, SId, F>(&mut self, query: &Q, subid: I, get_sid: F)
    where
        Q: Index<I> + Index<SId>,
        F: Fn(&<Q as Index<I>>::Output) -> SId,
    {
        let pattern = &query[subid];
        let hasher = IncHasher(std::hash::DefaultHasher::new(), 0);

        let mut stack = vec![(hasher, get_sid(pattern))];

        loop {
            let Some((mut hasher, id)) = stack.pop() else {
                return;
            };
            let step = &query[id];

            let k = todo!();
            let len = todo!(); //pattern.steps.length.0

            self.map.push((k, subid));
            self.max_sub_len = self.max_sub_len.max(len);
        }
    }

    pub(crate) fn matches<Q, SId>(&self, query: &Q, sid: SId) -> Vec<PatternId> {
        let mut res = vec![];
        let hasher = IncHasher(std::hash::DefaultHasher::new(), 0);
        let mut stack = vec![(hasher, sid)];
        loop {}
        res
    }
}

struct PrecomputedPatterns2(SubFinder<PatternId, u64, { PrecomputedPatterns2::INTERM }>);
impl PrecomputedPatterns2 {
    const INTERM: u16 = 1;
}
#[derive(Clone)]
struct IncHasher(std::hash::DefaultHasher, u16);

impl IncHasher {
    fn maybe_inc<'a>(&'a mut self, inc_v: &mut Vec<u64>) -> &'a mut std::hash::DefaultHasher {
        if self.1 == PrecomputedPatterns::INTERM {
            self.1 = Default::default();
            inc_v.push(self.0.clone().finish());
        } else {
            self.1 += 1;
        }
        &mut self.0
    }
}
impl IncHasher {
    fn inc<'a>(&'a mut self, inc_v: &mut Vec<u64>) -> &'a mut std::hash::DefaultHasher {
        if self.1 % PrecomputedPatterns::INTERM == 1 {
            inc_v.push(self.0.clone().finish());
        }
        self.1 += 1;
        &mut self.0
    }
    fn div(&self) -> Self {
        let mut s = self.clone();
        s.1 += 1;
        s
    }
}

impl PrecomputedPatterns {
    const INTERM: u16 = 1; // TODO the other numbers do not always works, need some unit tests for a generic PrecomputedSlices
    pub(crate) fn add_precomputed_pattern(&mut self, query: &Query, patternid: PatternId) {
        // dbg!(patternid);
        let pattern = &query.patterns[patternid];
        let stepid = pattern.steps.offset;
        let endstepid = pattern.steps.offset + pattern.steps.length;
        assert!(query.steps.contains(endstepid));
        let hasher = IncHasher(std::hash::DefaultHasher::new(), 0);
        let mut stack = vec![(hasher, stepid)];
        loop {
            let Some((mut hasher, id)) = stack.pop() else {
                // dbg!(&self.intermediate_hashes);
                return;
            };
            if hasher.1 % PrecomputedPatterns::INTERM == 0 {
                self.intermediate_hashes.push(hasher.0.clone().finish());
            }
            let step = &query.steps[id];
            if step.done() {
                // finish current
                let k = hasher.0.finish();
                self.map.push((k, patternid));
                self.max_sub_len = self.max_sub_len.max(hasher.1);
                continue;
            }
            if let Some(alt) = step.alternative_index() {
                unimplemented!("WIP just simple precomputing for now, no quantifiers");
                // branch
                // - forward is a ? or * quant
                // - backward is a + or * quant
            } else {
                hash_single_step(query, id, &mut hasher.0);
                hasher.1 += 1;
                // dbg!(hasher.0.clone().finish());
            }
            let mut id = id;
            id.inc();
            stack.push((hasher, id));
        }
    }

    pub(crate) fn finish_preparation(&mut self) {
        self.intermediate_hashes.sort();
        self.intermediate_hashes.dedup();
    }

    pub(crate) fn matches(&self, query: &Query, stepid: StepId) -> Vec<PatternId> {
        log::debug!("matching subpatts for stepid {}", stepid);
        let mut res = vec![];
        let hasher = IncHasher(std::hash::DefaultHasher::new(), 0);
        let mut stack = vec![(hasher, stepid)];
        loop {
            let Some((mut hasher, id)) = stack.pop() else {
                return res;
            };
            if hasher.1 >= self.max_sub_len {
                // TODO every x steps check if there is a sub that will be matched,
                // need to do this preparation in the add step
                continue;
            }
            if hasher.1 % PrecomputedPatterns::INTERM == 0 && hasher.1 > 0 {
                let hash = hasher.0.clone().finish();
                // dbg!(&hash);
                if !self.intermediate_hashes.binary_search(&hash).is_ok() {
                    continue;
                }
            }
            if id != stepid {
                let k = hasher.0.clone().finish();
                let iter = self.map.iter().filter_map(|(h, p)| (k == *h).then_some(p));
                res.extend(iter);
            }
            let step = &query.steps[id];
            if step.done() {
                // finished
                let k = hasher.0.finish();
                let iter = self.map.iter().filter_map(|(h, p)| (k == *h).then_some(p));
                res.extend(iter);
                continue;
            }
            if id != stepid && step.depth <= query.steps[stepid].depth {
                // should stop to avoid matching more than expected
                let k = hasher.0.finish();
                let iter = self.map.iter().filter_map(|(h, p)| (k == *h).then_some(p));
                res.extend(iter);
                continue;
            }
            if id != stepid {
                // prevents skiping first step
                let mut id = id.clone();
                id.inc();
                stack.push((hasher.clone(), id));
            }
            if step.field != 0 {
                let mut hasher = hasher.clone();
                hasher.1 += 1;
                hash_single_step1(query, id, &mut hasher.0);
                let mut id = id.clone();
                id.inc();
                stack.push((hasher, id));
            }
            if step.symbol != 0 {
                let mut hasher = hasher.clone();
                hasher.1 += 1;
                hash_single_step2(query, id, &mut hasher.0);
                let mut id = id.clone();
                id.inc();
                stack.push((hasher, id));
            }
            if step.symbol != 0 {
                let mut hasher = hasher.clone();
                hasher.1 += 1;
                hash_single_step12(query, id, &mut hasher.0);
                let mut id = id.clone();
                id.inc();
                stack.push((hasher, id));
            }
            hash_single_step(query, id, &mut hasher.0);
            let mut id = id;
            id.inc();
            stack.push((hasher, id));
        }
        // self.matches_aux(stepid, stepid, query, hasher, &mut res);
        // TODO add eq impl to avoid collisions
    }

    fn matches_aux(
        &self,
        stepid: StepId,
        mut id: StepId,
        query: &Query,
        mut hasher: IncHasher,
        res: &mut Vec<PatternId>,
    ) {
        if hasher.1 % PrecomputedPatterns::INTERM == 1 {
            if !self
                .intermediate_hashes
                .contains(&hasher.0.clone().finish())
            {
                return;
            }
        }

        if id != stepid {
            let k = hasher.0.clone().finish();
            let iter = self.map.iter().filter_map(|(h, p)| (k == *h).then_some(p));
            res.extend(iter);
        }
        loop {
            let step = &query.steps[id];
            if hasher.1 >= stepid.0 + self.max_sub_len + 5 {
                // TODO every x steps check if there is a sub that will be matched,
                // need to do this preparation in the add step
                return;
            } else if step.done() {
                // finished
                let k = hasher.0.finish();
                let iter = self.map.iter().filter_map(|(h, p)| (k == *h).then_some(p));
                res.extend(iter);
                return;
            } else if id != stepid && step.depth <= query.steps[stepid].depth {
                // should stop to avoid matching more than expected
                let k = hasher.0.finish();
                let iter = self.map.iter().filter_map(|(h, p)| (k == *h).then_some(p));
                res.extend(iter);
                return;
            } else if let Some(alt) = step.alternative_index() {
                // branch
                // WIP for now skip complex queries
                return;
                // TODO handle complex queries
                if alt < stepid {
                    // should probably break
                } else if step.is_pass_through() {
                } else if step.is_dead_end() {
                } else {
                }
            } else {
                if id != stepid {
                    // prevents skiping first step
                    let mut id = id.clone();
                    id.inc();
                    self.matches_aux(stepid, id, query, hasher.clone(), res);
                }
                if step.field != 0 {
                    let mut hasher = hasher.div();
                    hash_single_step1(query, id, &mut hasher.0);
                    let mut id = id.clone();
                    id.inc();
                    self.matches_aux(stepid, id, query, hasher, res);
                }
                if step.symbol != 0 {
                    let mut hasher = hasher.div();
                    hash_single_step2(query, id, &mut hasher.0);
                    let mut id = id.clone();
                    id.inc();
                    self.matches_aux(stepid, id, query, hasher, res);
                }
                if step.symbol != 0 {
                    let mut hasher = hasher.div();
                    hash_single_step12(query, id, &mut hasher.0);
                    let mut id = id.clone();
                    id.inc();
                    self.matches_aux(stepid, id, query, hasher, res);
                }
                hash_single_step(query, id, &mut hasher.0);
            }
            id.inc();
        }
    }
}

fn hash_single_step(query: &Query, stepid: StepId, hasher: &mut std::hash::DefaultHasher) {
    let step = &query.steps[stepid];
    step.is_dead_end().hash(hasher);
    step.is_immediate().hash(hasher);
    step.is_pass_through().hash(hasher);
    step.is_last_child().hash(hasher);
    step.field().hash(hasher);
    step.normed_alternative_index(stepid).hash(hasher);
    step.supertype_symbol().hash(hasher);
    step.symbol.hash(hasher);
    step.immediate_pred().hash(hasher);
    step.is_named().hash(hasher);
}

fn hash_single_step1(query: &Query, stepid: StepId, hasher: &mut std::hash::DefaultHasher) {
    let step = &query.steps[stepid];
    step.is_dead_end().hash(hasher);
    step.is_immediate().hash(hasher);
    step.is_pass_through().hash(hasher);
    step.is_last_child().hash(hasher);
    0u16.hash(hasher);
    step.normed_alternative_index(stepid).hash(hasher);
    step.supertype_symbol().hash(hasher);
    step.symbol.hash(hasher);
    step.immediate_pred().hash(hasher);
    step.is_named().hash(hasher);
}

fn hash_single_step2(query: &Query, stepid: StepId, hasher: &mut std::hash::DefaultHasher) {
    let step = &query.steps[stepid];
    step.is_dead_end().hash(hasher);
    step.is_immediate().hash(hasher);
    step.is_pass_through().hash(hasher);
    step.is_last_child().hash(hasher);
    step.field().hash(hasher);
    step.normed_alternative_index(stepid).hash(hasher);
    step.supertype_symbol().hash(hasher);
    0u16.hash(hasher);
    step.immediate_pred().hash(hasher);
    true.hash(hasher);
}
fn hash_single_step12(query: &Query, stepid: StepId, hasher: &mut std::hash::DefaultHasher) {
    let step = &query.steps[stepid];
    step.is_dead_end().hash(hasher);
    step.is_immediate().hash(hasher);
    step.is_pass_through().hash(hasher);
    step.is_last_child().hash(hasher);
    0u16.hash(hasher);
    step.normed_alternative_index(stepid).hash(hasher);
    step.supertype_symbol().hash(hasher);
    0u16.hash(hasher);
    step.immediate_pred().hash(hasher);
    true.hash(hasher);
}

impl QueryStep {
    pub(crate) fn is_named(&self) -> bool {
        self.bit_field & StepFlags::is_named != 0
    }
    pub(crate) fn is_immediate(&self) -> bool {
        self.bit_field & StepFlags::is_immediate != 0
    }
    pub(crate) fn is_last_child(&self) -> bool {
        self.bit_field & StepFlags::is_last_child != 0
    }
    pub(crate) fn is_pass_through(&self) -> bool {
        self.bit_field & StepFlags::is_pass_through != 0
        //  && self.negated_field_list_id == 42 * 2
    }
    pub(crate) fn is_dead_end(&self) -> bool {
        self.bit_field & StepFlags::is_dead_end != 0
        // && self.negated_field_list_id == 42
    }
    pub(crate) fn alternative_is_immediate(&self) -> bool {
        self.bit_field & StepFlags::alternative_is_immediate != 0
    }
    pub(crate) fn contains_captures(&self) -> bool {
        self.bit_field & StepFlags::contains_captures != 0
    }
    pub(crate) fn root_pattern_guaranteed(&self) -> bool {
        self.bit_field & StepFlags::root_pattern_guaranteed != 0
    }
    pub(crate) fn parent_pattern_guaranteed(&self) -> bool {
        self.bit_field & StepFlags::parent_pattern_guaranteed != 0
    }
    pub(crate) fn has_immediate_pred(&self) -> bool {
        self.bit_field & StepFlags::is_immediate_pred != 0
    }
    // pub(crate) fn is_neg(&self) -> bool {
    //     self.bit_field & StepFlags::is_neg != 0
    // }
    // pub(crate) fn set_neg(&mut self) {
    //     assert!(!self.is_neg());
    //     self.bit_field |= StepFlags::is_neg
    // }
}

#[repr(packed)]
pub(crate) struct StepFlags {
    // is_named: bool,
    // is_immediate: bool,
    // is_last_child: bool,
    // is_pass_through: bool,
    // is_dead_end: bool,
    // alternative_is_immediate: bool,
    // contains_captures: bool,
    // root_pattern_guaranteed: bool,
    // parent_pattern_guaranteed: bool,
}
#[allow(non_upper_case_globals)]
impl StepFlags {
    const is_named: u16 = 1 << 0;
    const is_immediate: u16 = 1 << 1;
    const is_last_child: u16 = 1 << 2;
    const is_pass_through: u16 = 1 << 3;
    const is_dead_end: u16 = 1 << 4;
    const alternative_is_immediate: u16 = 1 << 5;
    const contains_captures: u16 = 1 << 6;
    const root_pattern_guaranteed: u16 = 1 << 7;
    const parent_pattern_guaranteed: u16 = 1 << 8;
    const is_immediate_pred: u16 = 1 << 9;
    // const is_neg: u16 = 1 << 10;

    // const is_special: u8 = 1 << 3;
    // const alternative_is_immediate: u8 = 1 << 4;
    // const contains_captures: u8 = 1 << 5;
    // const root_pattern_guaranteed: u8 = 1 << 6;
    // const parent_pattern_guaranteed: u8 = 1 << 7;
}

#[derive(Clone)]
pub(crate) struct StepOffset {
    pub(crate) byte_offset: u32,
    pub(crate) step_index: StepId,
}

impl Query {
    pub(super) fn field_name(&self, field_id: ffi::TSFieldId) -> &str {
        let ptr = unsafe { ffi::ts_language_field_name_for_id(self.language, field_id) };
        if !ptr.is_null() {
            unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap()
        } else {
            ""
        }
    }

    pub(super) fn step_is_fallible(&self, step_index: StepId) -> bool {
        assert!(self.steps.contains(step_index.next_step_index()));
        let step = &self.steps[step_index];
        let next_step = &self.steps[step_index.next_step_index()];
        return next_step.depth != PATTERN_DONE_MARKER
            && next_step.depth > step.depth
            && !next_step.parent_pattern_guaranteed();
    }

    pub(super) fn pattern_map_search(&self, needle: super::Symbol) -> Option<usize> {
        // dbg!(query_step::symbol_name(self, needle.0));
        let mut base_index = self.wildcard_root_pattern_count.to();
        let mut size: usize = self.pattern_map.len() - base_index;
        // dbg!(needle.to_usize(), base_index, size);
        if size == 0 {
            return None;
        }
        while size > 1 {
            let half_size = size / 2;
            let mid_index = base_index + half_size;
            let pattern_entry: &PatternEntry = &self.pattern_map[mid_index];
            let mid_symbol = self.steps[pattern_entry.step_index].symbol as usize;
            // dbg!(mid_symbol);
            // dbg!(query_step::symbol_name(self, mid_symbol as u16));
            if needle.to_usize() > mid_symbol {
                base_index = mid_index
            };
            size -= half_size;
        }
        // dbg!(base_index, size);
        // dbg!(
        //     self.pattern_map[base_index].step_index,
        //     self.pattern_map[base_index].pattern_index
        // );

        let pattern_entry: &PatternEntry = &self.pattern_map[base_index];
        let mut symbol = self.steps[pattern_entry.step_index].symbol as usize;
        // dbg!(symbol);
        // dbg!(query_step::symbol_name(self, symbol as u16));

        if needle.to_usize() > symbol {
            base_index += 1;
            if base_index < self.pattern_map.len() {
                let pattern_entry: &PatternEntry = &self.pattern_map[base_index];
                symbol = self.steps[pattern_entry.step_index].symbol as usize;
            }
        }

        if needle.to_usize() == symbol {
            // dbg!(base_index);
            Some(base_index)
        } else {
            None
        }
    }

    pub(crate) fn text_predicates_for_pattern_id<'a>(
        &'a self,
        pattern_index: indexed::PatternId,
    ) -> impl Iterator<Item = &'a TextPredicateCapture> {
        self.text_predicates.preds_for_patern_id(pattern_index)
    }
}

impl Query {
    pub fn big(source: &[&str], language: Language) -> Result<Self, QueryError> {
        let mut source = source.into_iter();
        let s = source.next().unwrap_or(&"");
        let mut byte_offset = s.as_bytes().len();
        let mut query = Self::new(s, language.clone())?;
        for source in source {
            let step_offset = query.steps.count();
            let mut q = Self::new(source, language.clone())?;
            let mut capture_map = vec![];
            for c in q.capture_names {
                if let Some(i) = query.capture_names.iter().position(|x| x == &c) {
                    capture_map.push(CaptureId::new(num::cast(i).unwrap()));
                } else {
                    capture_map.push(CaptureId::new(
                        num::cast(query.capture_names.len()).unwrap(),
                    ));
                    query.capture_names.push(c);
                }
            }
            let neg_map = query.negated_fields.extend(q.negated_fields);
            for s in q.steps.iter_mut() {
                s.remap_captures(&capture_map);
            }
            for s in q.steps.iter_mut() {
                s.remap_negative_fields(&neg_map);
            }
            let imm_pred_offset = query.immediate_predicates.len();
            query.immediate_predicates.extend(q.immediate_predicates);
            // TODO dedup imm preds

            for s in q.steps.iter_mut() {
                s.remap_imm_pred(imm_pred_offset);
            }
            query.steps.extend(q.steps);
            for quant in q.capture_quantifiers_vec {
                let mut q = vec![CaptureQuantifier::Zero; query.capture_names.len()];
                for (i, quant) in quant.into_iter().enumerate() {
                    q[capture_map[i].0 as usize] = quant;
                }
                query.capture_quantifiers_vec.push(q);
            }
            query
                .patterns
                .extend(q.patterns, step_offset, num::cast(byte_offset).unwrap());

            {
                let pat_offset = query.pattern_count() - 1;
                for (i, mut p) in q.pattern_map.into_iter().enumerate() {
                    if p.precomputed != 0 {
                        todo!()
                    }
                    p.pattern_index =
                        PatternId::new(p.pattern_index.to_usize() + pat_offset.to_usize());
                    p.step_index += step_offset;
                    if i < q.wildcard_root_pattern_count as usize {
                        query
                            .pattern_map
                            .insert(i + query.wildcard_root_pattern_count as usize, p);
                    } else {
                        query.pattern_map.push(p);
                    }
                }
            }

            for mut o in q.step_offsets {
                o.byte_offset += byte_offset.to_u32().unwrap();
                o.step_index.add_assign(step_offset);
                query.step_offsets.push(o);
            }

            query.wildcard_root_pattern_count += q.wildcard_root_pattern_count;

            if !q.pattern_map2.is_empty() {
                todo!() // NOTE probably better to process precomputeds after Self::big
            }
            if q.precomputed_patterns.is_some() {
                todo!() // NOTE probably better to process precomputeds after Self::big
            }
            for p in q.text_predicates.iter_mut() {
                for p in p {
                    p.remap(&capture_map);
                }
            }
            query.text_predicates.extend(q.text_predicates);
            q.property_predicates.check_empty();
            query.property_predicates.extend(q.property_predicates);
            q.general_predicates.check_empty();
            query.general_predicates.extend(q.general_predicates);
            q.property_settings.check_empty();
            query.property_settings.extend(q.property_settings);
            if q.used_precomputed != 0 {
                todo!() // NOTE probably better to process precomputeds after Self::big
            }

            byte_offset = source.as_bytes().len();
        }
        Ok(query)
    }
    pub fn new(source: &str, language: Language) -> Result<Self, QueryError> {
        let ptr: *mut ffi::TSQuery = Self::init_tsquery(source, language)?;
        let query: *mut super::ffi_extra::TSQuery = unsafe { std::mem::transmute(ptr) };
        let ptr = {
            struct TSQueryDrop(*mut ffi::TSQuery);
            impl Drop for TSQueryDrop {
                fn drop(&mut self) {
                    unsafe { ffi::ts_query_delete(self.0) }
                }
            }
            TSQueryDrop(ptr)
        };

        let string_count = unsafe { ffi::ts_query_string_count(ptr.0) };
        let capture_count = unsafe { ffi::ts_query_capture_count(ptr.0) };
        let pattern_count = unsafe { ffi::ts_query_pattern_count(ptr.0) as usize };
        // dbg!(string_count, capture_count, pattern_count, unsafe {
        //     (*query).steps.len()
        // });
        let mut capture_names = Vec::with_capacity(capture_count as usize);
        let mut capture_quantifiers_vec = Vec::with_capacity(pattern_count as usize);
        let mut text_predicates_vec = PerPatternBuilder::with_patt_count(pattern_count);
        let mut property_predicates_vec = PerPatternBuilder::with_patt_count(pattern_count);
        let mut property_settings_vec = PerPatternBuilder::with_patt_count(pattern_count);
        let mut general_predicates_vec = PerPatternBuilder::with_patt_count(pattern_count);
        let mut immediate_predicates = vec![];
        let mut immediate_pred_steps = vec![];

        // Build a vector of strings to store the capture names.
        for i in 0..capture_count {
            unsafe {
                let mut length = 0u32;
                let name =
                    ffi::ts_query_capture_name_for_id(ptr.0, i, std::ptr::addr_of_mut!(length))
                        .cast::<u8>();
                let name = std::slice::from_raw_parts(name, length as usize);
                let name = std::str::from_utf8_unchecked(name);
                capture_names.push(name);
            }
        }

        // Build a vector to store capture qunatifiers.
        for i in 0..pattern_count {
            let mut capture_quantifiers = Vec::with_capacity(capture_count as usize);
            for j in 0..capture_count {
                unsafe {
                    let quantifier = ffi::ts_query_capture_quantifier_for_id(ptr.0, i as u32, j);
                    capture_quantifiers.push(quantifier.into());
                }
            }
            capture_quantifiers_vec.push(capture_quantifiers.into());
        }

        // Build a vector of strings to represent literal values used in predicates.
        let string_values = (0..string_count)
            .map(|i| unsafe {
                let mut length = 0u32;
                let value =
                    ffi::ts_query_string_value_for_id(ptr.0, i, std::ptr::addr_of_mut!(length))
                        .cast::<u8>();
                let value = std::slice::from_raw_parts(value, length as usize);
                let value = std::str::from_utf8_unchecked(value);
                value
            })
            .collect::<Vec<_>>();

        // Build a vector of predicates for each pattern.
        for i in 0..pattern_count {
            let predicate_steps = unsafe {
                let mut length = 0u32;
                let raw_predicates = ffi::ts_query_predicates_for_pattern(
                    ptr.0,
                    i as u32,
                    std::ptr::addr_of_mut!(length),
                );
                (length > 0)
                    .then(|| std::slice::from_raw_parts(raw_predicates, length as usize))
                    .unwrap_or_default()
            };

            let byte_offset = unsafe { ffi::ts_query_start_byte_for_pattern(ptr.0, i as u32) };
            let row = source
                .char_indices()
                .take_while(|(i, _)| *i < byte_offset as usize)
                .filter(|(_, c)| *c == '\n')
                .count();

            use ffi::TSQueryPredicateStepType as T;
            const TYPE_DONE: T = ffi::TSQueryPredicateStepTypeDone;
            const TYPE_CAPTURE: T = ffi::TSQueryPredicateStepTypeCapture;
            const TYPE_STRING: T = ffi::TSQueryPredicateStepTypeString;

            text_predicates_vec.prep();
            property_predicates_vec.prep();
            property_settings_vec.prep();
            general_predicates_vec.prep();
            let mut immediate_matches_calls = vec![];
            for p in predicate_steps.split(|s| s.type_ == TYPE_DONE) {
                if p.is_empty() {
                    continue;
                }

                if p[0].type_ != TYPE_STRING {
                    return Err(predicate_error(
                        row,
                        format!(
                            "Expected predicate to start with a function name. Got @{}.",
                            capture_names[p[0].value_id as usize],
                        ),
                    ));
                }

                // Build a predicate for each of the known predicate function names.
                let operator_name = string_values[p[0].value_id as usize];
                match operator_name {
                    "eq?" | "not-eq?" | "any-eq?" | "any-not-eq?" => {
                        if p.len() != 3 {
                            return Err(predicate_error(
                                row,
                                format!(
                                    "Wrong number of arguments to #eq? predicate. Expected 2, got {}.",
                                    p.len() - 1
                                ),
                            ));
                        }
                        if p[1].type_ != TYPE_CAPTURE {
                            return Err(predicate_error(
                                row,
                                format!(
                                    "First argument to #eq? predicate must be a capture name. Got literal \"{}\".",
                                    string_values[p[1].value_id as usize],
                                ),
                            ));
                        }

                        let is_positive = operator_name == "eq?" || operator_name == "any-eq?";
                        let match_all_nodes = match operator_name {
                            "eq?" | "not-eq?" => true,
                            "any-eq?" | "any-not-eq?" => false,
                            _ => unreachable!(),
                        };
                        let p1 = super::indexed::CaptureId::new(p[1].value_id);
                        text_predicates_vec.push(if p[2].type_ == TYPE_CAPTURE {
                            let p2 = super::indexed::CaptureId::new(p[2].value_id);
                            TextPredicateCapture::EqCapture(p1, p2, is_positive, match_all_nodes)
                        } else {
                            let p2 = string_values[p[2].value_id as usize].to_string().into();
                            TextPredicateCapture::EqString(p1, p2, is_positive, match_all_nodes)
                        });
                    }
                    "match?" | "not-match?" | "any-match?" | "any-not-match?" => {
                        if p.len() != 3 {
                            return Err(predicate_error(
                                row,
                                format!(
                                    "Wrong number of arguments to #match? predicate. Expected 2, got {}.",
                                    p.len() - 1
                                ),
                            ));
                        }
                        if p[1].type_ != TYPE_CAPTURE {
                            return Err(predicate_error(
                                row,
                                format!(
                                    "First argument to #match? predicate must be a capture name. Got literal \"{}\".",
                                    string_values[p[1].value_id as usize],
                                ),
                            ));
                        }
                        if p[2].type_ == TYPE_CAPTURE {
                            return Err(predicate_error(
                                row,
                                format!(
                                    "Second argument to #match? predicate must be a literal. Got capture @{}.",
                                    capture_names[p[2].value_id as usize],
                                ),
                            ));
                        }

                        let is_positive =
                            operator_name == "match?" || operator_name == "any-match?";
                        let match_all = match operator_name {
                            "match?" | "not-match?" => true,
                            "any-match?" | "any-not-match?" => false,
                            _ => unreachable!(),
                        };
                        let regex = &string_values[p[2].value_id as usize];
                        let p1 = super::indexed::CaptureId::new(p[1].value_id);
                        text_predicates_vec.push(TextPredicateCapture::MatchString(
                            p1,
                            regex::bytes::Regex::new(regex).map_err(|_| {
                                predicate_error(row, format!("Invalid regex '{regex}'"))
                            })?,
                            is_positive,
                            match_all,
                        ));
                    }

                    "set!" => property_settings_vec.push(Self::parse_property(
                        row,
                        operator_name,
                        &capture_names,
                        &string_values,
                        &p[1..],
                    )?),

                    "is?" | "is-not?" => property_predicates_vec.push((
                        Self::parse_property(
                            row,
                            operator_name,
                            &capture_names,
                            &string_values,
                            &p[1..],
                        )?,
                        operator_name == "is?",
                    )),

                    "any-of?" | "not-any-of?" => {
                        if p.len() < 2 {
                            return Err(predicate_error(
                                row,
                                format!(
                                    "Wrong number of arguments to #any-of? predicate. Expected at least 1, got {}.",
                                    p.len() - 1
                                ),
                            ));
                        }
                        if p[1].type_ != TYPE_CAPTURE {
                            return Err(predicate_error(
                                row,
                                format!(
                                    "First argument to #any-of? predicate must be a capture name. Got literal \"{}\".",
                                    string_values[p[1].value_id as usize],
                                ),
                            ));
                        }

                        let is_positive = operator_name == "any-of?";
                        let mut values = Vec::new();
                        for arg in &p[2..] {
                            if arg.type_ == TYPE_CAPTURE {
                                return Err(predicate_error(
                                    row,
                                    format!(
                                        "Arguments to #any-of? predicate must be literals. Got capture @{}.",
                                        capture_names[arg.value_id as usize],
                                    ),
                                ));
                            }
                            values.push(string_values[arg.value_id as usize]);
                        }
                        let p1 = super::indexed::CaptureId::new(p[1].value_id);
                        text_predicates_vec.push(TextPredicateCapture::AnyString(
                            p1,
                            values
                                .iter()
                                .map(|x| (*x).to_string().into())
                                .collect::<Vec<_>>()
                                .into(),
                            is_positive,
                        ));
                    }
                    "EQ?" | "NOT-EQ?" | "MATCH?" | "ANY" => {
                        // dbg!(byte_offset, row, operator_name);
                        let p1 = string_values[p[1].value_id as usize].to_string();
                        immediate_matches_calls.push((operator_name, p1));
                        // dbg!(&immediate_matches_calls);
                    }
                    _ => general_predicates_vec.push(crate::predicate::QueryPredicate {
                        operator: operator_name.to_string().into(),
                        args: p[1..]
                            .iter()
                            .map(|a| {
                                if a.type_ == TYPE_CAPTURE {
                                    crate::predicate::QueryPredicateArg::Capture(a.value_id)
                                } else {
                                    crate::predicate::QueryPredicateArg::String(
                                        string_values[a.value_id as usize].to_string().into(),
                                    )
                                }
                            })
                            .collect(),
                    }),
                }
            }

            let next_pat_index = i + 1;
            let max_pattern_byte = if next_pat_index < pattern_count {
                let b =
                    unsafe { ffi::ts_query_start_byte_for_pattern(ptr.0, next_pat_index as u32) };
                (b + 1) as usize
            } else {
                source.len()
            };
            Self::compute_immediate_preds(
                i,
                max_pattern_byte,
                source,
                query,
                immediate_matches_calls,
                row,
                &mut immediate_predicates,
                &mut immediate_pred_steps,
                pattern_count,
            )?;
            // property_predicates_vec.push(property_predicates.into());
            // property_settings_vec.push(property_settings.into());
            // general_predicates_vec.push(general_predicates.into());
        }

        let text_predicates = text_predicates_vec.build();
        let general_predicates = general_predicates_vec.build();
        let property_predicates = property_predicates_vec.build();
        let property_settings = property_settings_vec.build();

        let step_offsets = unsafe { &(*query).step_offsets }.into();
        // log::trace!("{}", ptr);
        let steps: indexed::Steps = unsafe { &(*query).steps }.into();
        let patterns: indexed::Patterns = unsafe { &(*query).patterns }.into();
        let pattern_map: Vec<PatternEntry> = unsafe { &(*query).pattern_map }.into();
        let pattern_map2 = pattern_map
            .iter()
            .filter_map(|x| {
                if steps[x.step_index].done() {
                    None
                } else if steps[x.step_index].depth > 0 {
                    None
                    // let step_index = patterns[x.pattern_index].steps.offset;
                    // assert_eq!(steps[step_index].depth, 0);
                    // Some(PatternEntry {
                    //     step_index,
                    //     pattern_index: x.pattern_index,
                    //     is_rooted: x.is_rooted,
                    //     precomputed: x.precomputed,
                    // })
                } else {
                    None
                }
            })
            .collect();
        // dbg!(&patterns);
        let mut enabled_pattern_map = vec![];
        let mut enabled_pattern_count = 0;
        for i in 0..patterns.len() {
            let i = PatternId::new(i);
            assert_ne!(patterns[i].steps.length, StepId::new(0));
            if patterns[i].is_empty() {
                enabled_pattern_map.push(u16::MAX);
                continue;
            }
            enabled_pattern_map.push(enabled_pattern_count);
            enabled_pattern_count += 1;
        }
        let mut query = Query {
            steps,
            pattern_map,
            pattern_map2,
            patterns,
            step_offsets,
            negated_fields: unsafe { &(*query).negated_fields }.into(),
            language: unsafe { ffi::ts_language_copy((*query).language) },
            wildcard_root_pattern_count: unsafe { (*query).wildcard_root_pattern_count },

            capture_names,
            capture_quantifiers_vec,
            text_predicates,
            general_predicates,
            property_predicates,
            property_settings,
            immediate_predicates,
            precomputed_patterns: Default::default(),
            used_precomputed: Default::default(),
            enabled_pattern_map,
            enabled_pattern_count,
        };
        for (s, i) in immediate_pred_steps {
            if query.steps.set_immediate_pred(s, i as u32) {}
        }
        std::mem::forget(ptr);
        log::info!("\n{}", query);
        Ok(query)
    }

    pub fn with_precomputed(
        query: &str,
        language: Language,
        precomputeds: impl ArrayStr,
    ) -> Result<(Self, Self), QueryError> {
        let source = &(format!(
            "{}\n\n{}",
            precomputeds
                .iter()
                .map(|x| format!("{}\n", x))
                .collect::<String>(),
            query
        ));
        log::trace!("parse query");
        let query = Self::new(source, language)?;
        log::trace!("prepare subqueries");

        let mut precomputed_patterns = PrecomputedPatterns::default();

        for i in query
            .enabled_pattern_map
            .iter()
            .copied()
            .filter(|x| *x != u16::MAX)
            .take(precomputeds.len())
        {
            let i = i as usize;
            precomputed_patterns.add_precomputed_pattern(&query, PatternId::new(i));
        }

        precomputed_patterns.finish_preparation();

        // dbg!(&precomputed_patterns);
        let max_sub_len = precomputed_patterns.max_sub_len;
        let mut query = query;
        query.precomputed_patterns = Some(precomputed_patterns);

        // for i in 0..precomputeds.len() {
        //     let r = query.precomputed_patterns.as_ref().unwrap().get(&query, query.patterns[PatternId::new(i)].steps.offset);
        //     dbg!(r);
        // }
        let precomp_len = precomputeds.len();
        if max_sub_len > 0 {
            log::trace!("started searching for subqueries");
            find_precomputed_uses(&mut query, precomputeds);
            log::trace!("finished searching for subqueries");
        }
        // let hasher = &mut std::hash::DefaultHasher::new();
        // hash_single_step(&query, StepId::new(1), hasher);
        // dbg!(hasher.finish());
        // let hasher = &mut std::hash::DefaultHasher::new();
        // hash_single_step(&query, StepId::new(24), hasher);
        // dbg!(hasher.finish());
        let mut precomp = query.clone();
        for i in query
            .enabled_pattern_map
            .iter()
            .copied()
            .filter(|x| *x != u16::MAX)
            .take(precomp_len)
            .collect::<Vec<_>>()
        {
            let i = i as usize;
            // dbg!(i);
            query.disable_pattern(PatternId::new(i));
        }
        for i in precomp
            .enabled_pattern_map
            .iter()
            .copied()
            .filter(|x| *x != u16::MAX)
            .skip(precomp_len)
            .collect::<Vec<_>>()
        {
            let i = i as usize;
            // dbg!(i);
            precomp.disable_pattern(PatternId::new(i));
        }
        log::trace!("finished query building");

        // dbg!(query.wildcard_root_pattern_count);
        // dbg!(&query.pattern_map);

        Ok((precomp, query))
    }

    pub fn disable_pattern(&mut self, pattern_index: PatternId) {
        for (i, pattern) in self.pattern_map.iter().enumerate() {
            if pattern.pattern_index == pattern_index {
                if i < self.wildcard_root_pattern_count as usize {
                    self.wildcard_root_pattern_count -= 1;
                    break;
                }
            }
        }
        if self.enabled_pattern_map[pattern_index.to_usize()] != u16::MAX {
            self.enabled_pattern_count -= 1;
            self.enabled_pattern_map[pattern_index.to_usize()] = u16::MAX;
            for m in &mut self.enabled_pattern_map[pattern_index.to_usize()..] {
                if *m != u16::MAX {
                    *m -= 1;
                }
            }
        }
        // Remove the given pattern from the pattern map. Its steps will still
        // be in the `steps` array, but they will never be read.
        self.pattern_map
            .retain(|pattern| pattern.pattern_index != pattern_index)
        // TODO check if the quantifier vec should also be updated
    }

    pub fn init_tsquery(source: &str, language: Language) -> Result<*mut ffi::TSQuery, QueryError> {
        // log::trace!("{:?}", language);
        // log::trace!("{:?}", source);
        let mut error_offset = 0u32;
        let mut error_type: ffi::TSQueryError = 0;
        let bytes = source.as_bytes();
        // Compile the query.
        let ptr = unsafe {
            ffi::ts_query_new(
                language.into_raw(),
                bytes.as_ptr().cast::<std::ffi::c_char>(),
                bytes.len() as u32,
                std::ptr::addr_of_mut!(error_offset),
                std::ptr::addr_of_mut!(error_type),
            )
        };

        // On failure, build an error based on the error code and offset.
        if ptr.is_null() {
            if error_type == ffi::TSQueryErrorLanguage {
                panic!();
            }

            let offset = error_offset as usize;
            let mut line_start = 0;
            let mut row = 0;
            let mut line_containing_error = None;
            for line in source.lines() {
                let line_end = line_start + line.len() + 1;
                if line_end > offset {
                    line_containing_error = Some(line);
                    break;
                }
                line_start = line_end;
                row += 1;
            }
            let column = offset - line_start;

            let kind;
            let message;
            match error_type {
                // Error types that report names
                ffi::TSQueryErrorNodeType | ffi::TSQueryErrorField | ffi::TSQueryErrorCapture => {
                    let suffix = source.split_at(offset).1;
                    let end_offset = suffix
                        .find(|c| !char::is_alphanumeric(c) && c != '_' && c != '-')
                        .unwrap_or(suffix.len());
                    message = suffix.split_at(end_offset).0.to_string();
                    kind = match error_type {
                        ffi::TSQueryErrorNodeType => QueryErrorKind::NodeType,
                        ffi::TSQueryErrorField => QueryErrorKind::Field,
                        ffi::TSQueryErrorCapture => QueryErrorKind::Capture,
                        _ => unreachable!(),
                    };
                }

                // Error types that report positions
                _ => {
                    message = line_containing_error.map_or_else(
                        || "Unexpected EOF".to_string(),
                        |line| {
                            line.to_string() + "\n" + " ".repeat(offset - line_start).as_str() + "^"
                        },
                    );
                    kind = match error_type {
                        ffi::TSQueryErrorStructure => QueryErrorKind::Structure,
                        _ => QueryErrorKind::Syntax,
                    };
                }
            };

            return Err(QueryError {
                row,
                column,
                offset,
                message,
                kind,
            });
        };
        Ok(ptr)
    }

    pub fn quant(&self, pid: PatternId, cid: CaptureId) -> CaptureQuantifier {
        self.capture_quantifiers_vec[pid.to_usize()][cid.to_usize()]
    }

    pub fn quants(&self, cid: CaptureId) -> impl Iterator<Item = PatternId> {
        self.capture_quantifiers_vec
            .iter()
            .enumerate()
            .filter_map(move |(i, c)| {
                matches!(
                    c[cid.to_usize()],
                    CaptureQuantifier::One | CaptureQuantifier::OneOrMore
                )
                .then_some(PatternId::new(i))
            })
    }

    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    pub fn enabled_pattern_count(&self) -> usize {
        self.enabled_pattern_count as usize
    }
    pub fn enabled_pattern_index(&self, pid: PatternId) -> Option<u16> {
        // log::error!("{:?}", self.enabled_pattern_map);
        // log::error!("{}", pid.to_usize());
        let i = self.enabled_pattern_map[pid.to_usize()];
        (i != u16::MAX).then_some(i)
    }
    pub fn with_one_pattern_enabled(mut self, i: u16) -> Result<Self, Self> {
        if i == u16::MAX
            || self.enabled_pattern_count() == 0
            || self.enabled_pattern_count() <= i as usize
        {
            return Err(self);
        }
        self.wildcard_root_pattern_count = if i < self.wildcard_root_pattern_count {
            1
        } else {
            0
        };
        let pattern = self.pattern_map.swap_remove(i as usize);
        self.enabled_pattern_map[pattern.pattern_index.to_usize()] = 0;
        for pattern in self.pattern_map.drain(..) {
            self.enabled_pattern_map[pattern.pattern_index.to_usize()] = u16::MAX;
        }
        self.pattern_map = vec![pattern];
        Ok(self)
    }
    pub fn get_each_pat_start_byte(&self) -> Vec<usize> {
        let mut r = vec![];
        for (i, j) in self.enabled_pattern_map.iter().enumerate() {
            if *j != u16::MAX {
                r.push(self.patterns[PatternId::new(i)].start_byte as usize);
            }
        }
        r
    }
    pub fn capture_index_for_name(&self, name: &str) -> Option<CaptureId> {
        self.capture_names
            .iter()
            .position(|x| *x == name)
            .map(|i| CaptureId::new(i as u32))
    }

    pub fn capture_quantifiers(
        &self,
        index: usize,
    ) -> impl std::ops::Index<usize, Output = CaptureQuantifier> {
        self.capture_quantifiers_vec[index].clone()
    }

    pub fn capture_name(&self, i: CaptureId) -> &'static str {
        self.capture_names[i.to_usize()]
    }

    fn parse_property(
        row: usize,
        function_name: &str,
        capture_names: &[&str],
        string_values: &[&str],
        args: &[ffi::TSQueryPredicateStep],
    ) -> Result<QueryProperty, QueryError> {
        if args.is_empty() || args.len() > 3 {
            return Err(predicate_error(
                row,
                format!(
                    "Wrong number of arguments to {function_name} predicate. Expected 1 to 3, got {}.",
                    args.len(),
                ),
            ));
        }

        let mut capture_id = None;
        let mut key = None;
        let mut value = None;

        for arg in args {
            if arg.type_ == ffi::TSQueryPredicateStepTypeCapture {
                if capture_id.is_some() {
                    return Err(predicate_error(
                        row,
                        format!(
                            "Invalid arguments to {function_name} predicate. Unexpected second capture name @{}",
                            capture_names[arg.value_id as usize]
                        ),
                    ));
                }
                capture_id = Some(arg.value_id as usize);
            } else if key.is_none() {
                key = Some(&string_values[arg.value_id as usize]);
            } else if value.is_none() {
                value = Some(string_values[arg.value_id as usize]);
            } else {
                return Err(predicate_error(
                    row,
                    format!(
                        "Invalid arguments to {function_name} predicate. Unexpected third argument @{}",
                        string_values[arg.value_id as usize]
                    ),
                ));
            }
        }

        if let Some(key) = key {
            Ok(QueryProperty::new(key, value, capture_id))
        } else {
            Err(predicate_error(
                row,
                format!("Invalid arguments to {function_name} predicate. Missing key argument",),
            ))
        }
    }

    fn compute_immediate_preds(
        i: usize,
        max_pattern_byte: usize,
        source: &str,
        query: *mut crate::ffi_extra::TSQuery,
        immediate_matches_calls: Vec<(&str, String)>,
        row: usize,
        immediate_predicates: &mut Vec<crate::predicate::ImmediateTextPredicate>,
        immediate_pred_steps: &mut Vec<(StepId, usize)>,
        pattern_count: usize,
    ) -> Result<(), QueryError> {
        let start_byte = unsafe { &(*query).patterns }[i].start_byte as usize;
        let comment_line_removed = &comment_lines_removed(source);
        let re = regex::Regex::new("[(]#(EQ|NOT-EQ|MATCH|ANY)[?]").unwrap();
        let haystack = &comment_line_removed[start_byte..max_pattern_byte];
        let glob_caps = re.find_iter(haystack);
        let glob_caps_count = glob_caps.count();
        assert_eq!(immediate_matches_calls.len(), glob_caps_count); // Not compatible with comments
        let step_id = {
            let mut i = i;
            let mut step_id = unsafe { &(*query).patterns }[i].steps.offset;
            while unsafe { &(*query).patterns }[i].steps.length == 1 {
                // dbg!(&immediate_matches_calls);
                if step_id == 0 {
                    return Err(QueryError {
                        row: 0,
                        column: 0,
                        offset: 0,
                        message: "Predicates have to be applied on something".into(),
                        kind: QueryErrorKind::Structure,
                    });
                }
                step_id -= 1;
                assert_ne!(i, 0);
                i -= 1;
            }
            step_id
        };
        let limit_step_id = {
            let slice = &unsafe { &(*query).patterns }[i].steps;
            // dbg!(&slice);
            let mut limit_step_id = slice.offset + slice.length;
            let mut i = i + 1;
            while i < unsafe { (*query).patterns.len() } {
                let length = unsafe { &(*query).patterns }[i].steps.length;
                assert_eq!(limit_step_id, unsafe { &(*query).patterns }[i].steps.offset);
                limit_step_id += length;
                if length > 1 {
                    break;
                }
                i += 1;
            }
            limit_step_id as usize
        };
        // if pattern_count > 10 && i > 30 {
        //     dbg!(&glob_caps.count());
        //     panic!()
        // }
        let step_offsets = &unsafe { &(*query).step_offsets }[..];
        let mut stp_id = 0;
        loop {
            if (step_offsets[stp_id].step_index as u32) < step_id {
                stp_id += 1;
            } else {
                break;
            }
        }
        // dbg!(limit_step_id);
        let step_offsets = step_offsets;
        assert_eq!(glob_caps_count, immediate_matches_calls.len(), "{}", i);
        // dbg!(&step_offsets[stp_id..]);
        // dbg!(
        //     step_offsets[stp_id].step_index,
        //     immediate_matches_calls.len()
        // );
        let mut aaa = 0;
        while stp_id < step_offsets.len()
            && (step_offsets[stp_id].step_index as usize) < limit_step_id
            && aaa < immediate_matches_calls.len()
        {
            // dbg!(stp_id);
            let stpid = stp_id;
            let so = &step_offsets[stpid];
            // dbg!(so.step_index as usize, stpid);
            // assert_eq!(so.step_index as usize, step_id);
            let re = regex::Regex::new("^[(]#(EQ|NOT-EQ|MATCH|ANY)[?]").unwrap();
            let haystack = &comment_line_removed[so.byte_offset as usize..];
            // if pattern_count > 10 {
            //     dbg!(stpid, so.step_index, &haystack[..10]);
            // }
            let cap = re.captures(haystack);
            if let Some(cap) = &cap {
                // source[so.byte_offset as usize..].starts_with("(#EQ?")
                let op = cap.get(1).unwrap().as_str();
                // dbg!(op, so.byte_offset);
                let so2 = if stpid as usize + 1 < step_offsets.len() {
                    let x = step_offsets[stpid + 1].byte_offset;
                    x as usize
                } else {
                    max_pattern_byte
                };
                let mut quoted = false;
                let mut escaped = false;
                let mut j = so.byte_offset as usize + 1;
                for (i, c) in source[so.byte_offset as usize + 1..so2].char_indices() {
                    if quoted {
                        if escaped {
                            escaped = false
                        } else if c == '"' {
                            quoted = false
                        } else if c == '\\' {
                            escaped = true
                        }
                    } else if c == ')' {
                        j += i + 1;
                        break;
                    } else if c == '\"' {
                        quoted = true
                    }
                }
                if stpid == 0 {
                    return Err(QueryError {
                        row: 0,
                        column: 0,
                        offset: 0,
                        message: "You cannot use a single predicate as a query".into(),
                        kind: QueryErrorKind::Structure,
                    });
                }
                let so2 = &step_offsets[stpid - 1];
                let s = &unsafe { &(*query).steps }[so2.step_index as usize];
                assert_eq!(immediate_matches_calls[aaa].0, &format!("{}?", op));
                let value = if op == "EQ" || op == "NOT-EQ" {
                    // dbg!(immediate_matches_calls[aaa].1.clone());
                    crate::predicate::ImmediateTextPredicate::EqString {
                        is_named: s.is_named(),
                        str: immediate_matches_calls[aaa].1.clone().into(),
                        is_positive: op == "EQ",
                    }
                } else if op == "MATCH" {
                    let regex = &immediate_matches_calls[aaa].1;
                    if s.is_named() {
                        crate::predicate::ImmediateTextPredicate::MatchString {
                            re: regex::bytes::Regex::new(regex).map_err(|_| {
                                predicate_error(row, format!("Invalid regex '{regex}'"))
                            })?,
                        }
                    } else {
                        crate::predicate::ImmediateTextPredicate::MatchStringUnamed {
                            re: regex::bytes::Regex::new(regex).map_err(|_| {
                                predicate_error(row, format!("Invalid regex '{regex}'"))
                            })?,
                        }
                    }
                } else {
                    todo!("{}", op)
                };
                let sym = symbol_name(unsafe { &(*query).language }, s.symbol);
                // if pattern_count > 10 {
                //     dbg!(sym, so2.step_index);
                // }
                // if pattern_count > 10 && s.symbol != 0 && sym != Some("identifier") {
                //     dbg!(sym);
                //     panic!()
                // }
                if let Some(i) = immediate_predicates.iter().position(|x| x == &value) {
                    immediate_pred_steps.push((StepId::new(so2.step_index), i));
                } else {
                    immediate_pred_steps
                        .push((StepId::new(so2.step_index), immediate_predicates.len()));
                    immediate_predicates.push(value);
                }
                aaa += 1;
            } else if pattern_count > 10 {
                // dbg!(cap, &haystack[..10]);
                // let s = &unsafe { &(*query).steps }[step_id as usize];
                // let sym = symbol_name(unsafe { &(*query).language }, s.symbol);
                // if s.symbol != 0 && sym == Some("identifier") {
                //     dbg!(i, sym);
                //     panic!()
                // }
            }
            stp_id += 1;
        }
        pub(crate) fn symbol_name(
            lang: &*const tree_sitter::ffi::TSLanguage,
            symbol: tree_sitter::ffi::TSSymbol,
        ) -> Option<&str> {
            let ptr = unsafe { tree_sitter::ffi::ts_language_symbol_name(*lang, symbol) };
            if !ptr.is_null() {
                Some(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap())
            } else {
                None
            }
        }
        // dbg!(step_id, step_offsets);
        // assert_eq!(aaa, immediate_matches_calls.len(), "{}", i);
        // assert_eq!(glob_caps_count, aaa);
        Ok(())
    }
}

fn comment_lines_removed(src: &str) -> String {
    let r = src
        .split_inclusive("\n")
        .map(|line| {
            if line.starts_with(';') {
                if line.ends_with("\r\n") {
                    " ".repeat(line.len() - 2) + "\r\n"
                } else if line.ends_with("\n") {
                    " ".repeat(line.len() - 1) + "\n"
                } else {
                    " ".repeat(line.len())
                }
            } else {
                line.to_string()
            }
        })
        .collect::<String>();
    assert_eq!(r.len(), src.len());
    r
}

fn find_precomputed_uses(query: &mut Query, precomputeds: impl ArrayStr) {
    query.used_precomputed = Precomps::MAX;
    for i in query
        .enabled_pattern_map
        .iter()
        .copied()
        .filter(|x| *x != u16::MAX)
        .skip(precomputeds.len())
    {
        let i = i as usize;
        log::trace!("[{}:{}:{}] {}", file!(), line!(), column!(), i);
        let patid = PatternId::new(i);
        let slice = &query.patterns[patid].steps;
        let mut j = slice.offset;
        let mut res = vec![];
        while j < slice.offset + slice.length {
            let r = query
                .precomputed_patterns
                .as_ref()
                .unwrap()
                .matches(&*query, j);
            res.extend(r.into_iter().map(|x| (x, j)));
            j.inc();
        }
        if let Some(m_pat) = &mut query
            .pattern_map
            .iter_mut()
            .find(|x| x.pattern_index == patid)
        {
            assert_eq!(m_pat.precomputed, 0);
            for r in &res {
                let r = r.0.to_usize();
                assert!(r < 16);
                m_pat.precomputed |= 1 << r as Precomps;
            }
            log::debug!(
                "found subpatts [{:0>16b}] for pattern {}",
                m_pat.precomputed,
                i
            );
            query.used_precomputed &= m_pat.precomputed;
        }
    }
}

impl Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Query")
            // .field("steps", &self.steps)
            .field("pattern_map", &self.pattern_map)
            .field("pattern_map2", &self.pattern_map2)
            .field("patterns", &self.patterns)
            // .field("step_offsets", &self.step_offsets)
            // .field("negated_fields", &self.negated_fields)
            .field("language", &self.language)
            .field(
                "wildcard_root_pattern_count",
                &self.wildcard_root_pattern_count,
            )
            .field("capture_names", &self.capture_names)
            .field("capture_quantifiers_vec", &self.capture_quantifiers_vec)
            .field("text_predicates", &self.text_predicates)
            .field("property_predicates", &self.property_predicates)
            .field("property_settings", &self.property_settings)
            .field("general_predicates", &self.general_predicates)
            .field("immediate_predicates", &self.immediate_predicates)
            .field("precomputed_patterns", &self.precomputed_patterns)
            .field("used_precomputed", &self.used_precomputed)
            .field("enabled_pattern_map", &self.enabled_pattern_map)
            .field("enabled_pattern_count", &self.enabled_pattern_count)
            .finish()
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        pub(crate) fn print_query_step(
            query: &Query,
            step: &QueryStep,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            const WILDCARD_SYMBOL: u16 = 0;
            write!(f, "{{")?;
            if step.done() {
                write!(f, "   ")?;
            } else {
                write!(f, "{:>2} ", step.depth)?;
            }
            if step.done() {
                write!(f, "DONE")?;
            } else if step.is_dead_end() {
                write!(f, "dead_end")?;
            } else if step.is_pass_through() {
                write!(f, "pass_through")?;
            } else {
                write!(f, "symbol: ")?;
                if step.supertype_symbol != WILDCARD_SYMBOL {
                    if let Some(s) = symbol_name(query, step.supertype_symbol) {
                        write!(f, "{}/", s)?
                    } else {
                        write!(f, "{}/", step.supertype_symbol)?
                    }
                }
                if step.symbol != WILDCARD_SYMBOL {
                    if let Some(s) = symbol_name(query, step.symbol) {
                        write!(f, "{}", s)?
                    } else {
                        write!(f, "{}", step.symbol)?
                    }
                } else {
                    write!(f, "*")?
                }
            }
            if step.is_named() {
                write!(f, ", named")?;
            }
            if step.is_immediate() {
                write!(f, ", immediate")?;
            }
            if step.is_last_child() {
                write!(f, ", last_child")?;
            }
            if step.alternative_is_immediate() {
                write!(f, ", alternative_is_immediate")?;
            }
            if step.contains_captures() {
                write!(f, ", contains_captures")?;
            }
            if step.root_pattern_guaranteed() {
                write!(f, ", root_pattern_guaranteed")?;
            }
            if step.parent_pattern_guaranteed() {
                write!(f, ", parent_pattern_guaranteed")?;
            }
            // if step.is_neg() {
            //     write!(f, ", neg")?;
            // }
            if let Some(imm) = step.immediate_pred() {
                write!(f, ", imm:{}", imm)?;
            }

            if step.field > 0 {
                if let Some(s) = field_name(query, step.field) {
                    write!(f, ", field: {}", s)?
                } else {
                    write!(f, ", field: {}", step.field)?
                }
            }
            if let Some(alt) = step.alternative_index() {
                write!(f, ", alternative: {}", alt)?;
            }
            write!(f, "}}")?;
            // NOTE C is not always zerowing the 7 unused bits so lets mask them
            write!(f, " bitfield: {:b}", step.bit_field)
        }

        pub(crate) fn symbol_name<'a>(
            query: &'a Query,
            symbol: tree_sitter::ffi::TSSymbol,
        ) -> Option<&'a str> {
            let ptr = unsafe { tree_sitter::ffi::ts_language_symbol_name(query.language, symbol) };
            if !ptr.is_null() {
                Some(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap())
            } else {
                None
            }
        }

        pub(crate) fn field_name<'a>(
            query: &'a Query,
            field: tree_sitter::ffi::TSFieldId,
        ) -> Option<&'a str> {
            let ptr =
                unsafe { tree_sitter::ffi::ts_language_field_name_for_id(query.language, field) };
            if !ptr.is_null() {
                Some(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap())
            } else {
                None
            }
        }
        for (i, step) in self.steps.iter().enumerate() {
            write!(f, "  {:>2}: ", i)?;
            print_query_step(self, step, f)?;
            write!(f, ",\n")?;
        }
        Ok(())
    }
}

impl From<&crate::ffi_extra::TSQueryStep> for QueryStep {
    fn from(x: &crate::ffi_extra::TSQueryStep) -> Self {
        let capture_ids = [
            CaptureId::new(x.capture_ids[0].to()),
            CaptureId::new(x.capture_ids[1].to()),
            CaptureId::new(x.capture_ids[2].to()),
        ];
        let negated_field_list_id;
        let mut r = 0;
        if x.is_named() {
            r |= StepFlags::is_named;
        }
        if x.is_immediate() {
            r |= StepFlags::is_immediate;
        }
        if x.is_last_child() {
            r |= StepFlags::is_last_child;
        }
        if x.depth == PATTERN_DONE_MARKER {
            assert!(!x.is_pass_through());
            assert!(!x.is_dead_end());
            negated_field_list_id = x.negated_field_list_id;
        } else if x.is_pass_through() {
            r |= StepFlags::is_pass_through;
            assert_eq!(0, x.negated_field_list_id);
            negated_field_list_id = 0;
        } else if x.is_dead_end() {
            r |= StepFlags::is_dead_end;
            assert_eq!(0, x.negated_field_list_id);
            negated_field_list_id = 0;
        } else {
            negated_field_list_id = x.negated_field_list_id;
        }
        if x.alternative_is_immediate() {
            r |= StepFlags::alternative_is_immediate;
        }
        if x.contains_captures() {
            r |= StepFlags::contains_captures;
        }
        if x.root_pattern_guaranteed() {
            r |= StepFlags::root_pattern_guaranteed;
        }
        if x.parent_pattern_guaranteed() {
            r |= StepFlags::parent_pattern_guaranteed;
        }
        let bit_field = r;
        QueryStep {
            symbol: x.symbol,
            supertype_symbol: x.supertype_symbol,
            field: x.field,
            capture_ids,
            depth: x.depth,
            alternative_index: StepId::new(x.alternative_index),
            negated_field_list_id,
            bit_field,
        }
    }
}

impl Into<Vec<PatternEntry>> for &Array<crate::ffi_extra::TSPatternEntry> {
    fn into(self) -> Vec<PatternEntry> {
        self.iter().map(|x| x.into()).collect()
    }
}

impl Into<Vec<StepOffset>> for &Array<crate::ffi_extra::TSStepOffset> {
    fn into(self) -> Vec<StepOffset> {
        self.iter()
            .map(|x| StepOffset {
                byte_offset: x.byte_offset,
                step_index: StepId::new(x.step_index),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immediate_preds() {
        // TODO make a bigger query with many steps
        // TODO each pattern should be identica excl. `alternative_index`
        let q = |s: &str| format!("(identifier) (#EQ? \"{}\")\n", s);
        let ts_query =
            tree_sitter::Query::new(&tree_sitter_java::language(), &q("a").repeat(100)).unwrap();
        assert_eq!(ts_query.pattern_count(), 200);
        let query = Query::new(&q("a").repeat(100), tree_sitter_java::language()).unwrap();
        println!("{}", query);
        dbg!(&query.pattern_map[0]);
        dbg!(&query.pattern_map[1]);
        assert_eq!(query.pattern_map.len(), 200);
        assert_eq!(query.patterns.len(), 200);
        assert_eq!(query.pattern_count(), 200);
        assert_eq!(query.enabled_pattern_count(), 100);
        dbg!(query.pattern_map[0].pattern_index);
        assert_eq!(
            query.enabled_pattern_index(query.pattern_map[0].pattern_index),
            Some(0)
        );
        for s in query.steps.iter() {
            if s.done() {
                continue;
            }
            if s.symbol == 0 {
                continue;
            }
            dbg!(s);
            if symbol_name(&query, s.symbol) == Some("identifier") {
                // in combination to adding imm pred to every identifier in the textual query format
                assert!(s.has_immediate_pred());
            }
        }
        pub(crate) fn symbol_name<'a>(
            query: &'a Query,
            symbol: tree_sitter::ffi::TSSymbol,
        ) -> Option<&'a str> {
            let ptr = unsafe { tree_sitter::ffi::ts_language_symbol_name(query.language, symbol) };
            if !ptr.is_null() {
                Some(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap())
            } else {
                None
            }
        }
    }
}

#[allow(unused)]
#[cfg(test)]
mod exp {
    use super::*;

    struct A {
        symbol: ffi::TSSymbol,
        supertype_symbol: ffi::TSSymbol,
        field: ffi::TSFieldId,
        capture_ids: [CaptureId; MAX_STEP_CAPTURE_COUNT],
        depth: SmallDepth,
        alternative_index: StepId,
        negated_field_list_id: u16,
        flags: u16,
    }

    struct AAA([QS; 4]);
    enum QS {
        // stoping
        Done,
        // jumping
        DeadEnd {
            alternative_index: StepId,
            flags: F,
        },
        // branching
        PassThrough {
            alternative_index: StepId,
            flags: F,
        },
        PreComputed {
            id: u32,
            depth: SmallDepth,
        },
        Split1 {
            symbol: ffi::TSSymbol,
            ss: ffi::TSSymbol,
            capture_ids: [CaptureId; MAX_STEP_CAPTURE_COUNT],
            field: ffi::TSFieldId,
        },
        Split2 {
            depth: SmallDepth,
            alternative_index: StepId,
            neg_field_list_id: u16,
            flags: F,
        },
        SupSymbol {
            symbol: ffi::TSSymbol,
            ss: ffi::TSSymbol,
            depth: SmallDepth,
            capture_ids: [CaptureId; 1],
            flags: F,
        },
        A {
            symbol: ffi::TSSymbol,
            ss: ffi::TSSymbol,
            depth: SmallDepth,
            field: ffi::TSFieldId,
            flags: F,
        },
        B {
            symbol: ffi::TSSymbol,
            ss: ffi::TSSymbol,
            depth: SmallDepth,
            neg_field_list_id: u16,
            flags: F,
        },
        C {
            symbol: ffi::TSSymbol,
            depth: u8,
            capture_ids: [CaptureId; 2],
            flags: F,
        },
        D {
            symbol: ffi::TSSymbol,
            field: ffi::TSFieldId,
            depth: SmallDepth,
            capture_ids: [CaptureId; 1],
            flags: F,
        },
        // Default {
        //     symbol: ffi::TSSymbol,
        //     ss: ffi::TSSymbol,
        //     field: ffi::TSFieldId,
        //     capture_ids: [CaptureId; MAX_STEP_CAPTURE_COUNT],
        //     depth: SmallDepth,
        //     alternative_index: StepId,
        //     neg_field_list_id: u16,
        //     flags: F,
        // },
    }

    #[repr(packed)]
    struct F {
        is_named: bool,
        is_immediate: bool,
        is_last_child: bool,
        alternative_is_immediate: bool,
        contains_captures: bool,
        root_pattern_guaranteed: bool,
        parent_pattern_guaranteed: bool,
    }

    impl QS {
        pub fn pre_computed(&self) -> Option<(u32, SmallDepth)> {
            match self {
                QS::PreComputed { id, depth } => Some((*id, *depth)),
                _ => None,
            }
        }
        pub fn symbol(&self) -> ffi::TSSymbol {
            match self {
                QS::Done => unreachable!(),
                QS::DeadEnd { .. } => unreachable!(),
                QS::PassThrough { .. } => unreachable!(),
                QS::PreComputed { .. } => unreachable!(),
                QS::Split1 { symbol, .. } => *symbol,
                QS::Split2 { .. } => todo!(),
                QS::SupSymbol { symbol, .. } => *symbol,
                QS::A { symbol, .. } => *symbol,
                QS::B { symbol, .. } => *symbol,
                QS::C { symbol, .. } => *symbol,
                QS::D { symbol, .. } => *symbol,
            }
        }
        pub fn supertype_symbol(&self) -> ffi::TSSymbol {
            match self {
                QS::Done => unreachable!(),
                QS::DeadEnd { .. } => unreachable!(),
                QS::PassThrough { .. } => unreachable!(),
                QS::PreComputed { .. } => unreachable!(),
                QS::Split1 { ss, .. } => *ss,
                QS::Split2 { .. } => todo!(),
                QS::SupSymbol { ss, .. } => *ss,
                QS::A { ss, .. } => *ss,
                QS::B { ss, .. } => *ss,
                QS::C { .. } => todo!(),
                QS::D { .. } => todo!(),
            }
        }
        pub fn field(&self) -> ffi::TSFieldId {
            match self {
                QS::Done => unreachable!(),
                QS::DeadEnd { .. } => unreachable!(),
                QS::PassThrough { .. } => unreachable!(),
                QS::PreComputed { .. } => todo!(),
                QS::Split1 { .. } => todo!(),
                QS::Split2 { .. } => todo!(),
                QS::SupSymbol { .. } => todo!(),
                QS::A { .. } => todo!(),
                QS::B { .. } => todo!(),
                QS::C { .. } => todo!(),
                QS::D { .. } => todo!(),
            }
        }
        pub fn capture_ids(&self) -> &[CaptureId; MAX_STEP_CAPTURE_COUNT] {
            match self {
                QS::Done => unreachable!(),
                QS::DeadEnd { .. } => unreachable!(),
                QS::PassThrough { .. } => unreachable!(),
                QS::PreComputed { .. } => unreachable!(),
                QS::Split1 { .. } => todo!(),
                QS::Split2 { .. } => todo!(),
                QS::SupSymbol { .. } => todo!(),
                QS::A { .. } => todo!(),
                QS::B { .. } => todo!(),
                QS::C { .. } => todo!(),
                QS::D { .. } => todo!(),
            }
        }
        pub fn depth(&self) -> SmallDepth {
            match self {
                QS::Done => unreachable!(),
                QS::DeadEnd { .. } => unreachable!(),
                QS::PassThrough { .. } => unreachable!(),
                QS::PreComputed { depth, .. } => *depth,
                QS::Split1 { .. } => todo!(),
                QS::Split2 { depth, .. } => *depth,
                QS::SupSymbol { depth, .. } => *depth,
                QS::A { depth, .. } => *depth,
                QS::B { depth, .. } => *depth,
                QS::C { depth, .. } => *depth as u16,
                QS::D { depth, .. } => *depth,
            }
        }
        pub fn alternative_index(&self) -> StepId {
            match self {
                QS::Done => unreachable!(),
                QS::DeadEnd {
                    alternative_index, ..
                } => *alternative_index,
                QS::PassThrough {
                    alternative_index, ..
                } => *alternative_index,
                QS::PreComputed { .. } => unreachable!(),
                QS::Split1 { .. } => todo!(),
                QS::Split2 {
                    alternative_index, ..
                } => *alternative_index,
                QS::SupSymbol { .. } => todo!(),
                QS::A { .. } => todo!(),
                QS::B { .. } => todo!(),
                QS::C { .. } => todo!(),
                QS::D { .. } => todo!(),
            }
        }
        pub fn negated_field_list_id(&self) -> u16 {
            match self {
                QS::Done => unreachable!(),
                QS::DeadEnd { .. } => unreachable!(),
                QS::PassThrough { .. } => unreachable!(),
                QS::PreComputed { .. } => unreachable!(),
                QS::Split1 { .. } => todo!(),
                QS::Split2 {
                    neg_field_list_id, ..
                } => *neg_field_list_id,
                QS::SupSymbol { .. } => todo!(),
                QS::A { .. } => todo!(),
                QS::B {
                    neg_field_list_id, ..
                } => *neg_field_list_id,
                QS::C { .. } => todo!(),
                QS::D { .. } => todo!(),
            }
        }
    }
}

#[allow(unused)]
mod exp_union {
    use super::*;

    #[repr(u8)]
    #[allow(non_camel_case_types)]
    pub(crate) enum Flags {
        is_named = 1,
        is_immediate = 1 << 1,
        is_last_child = 1 << 2,
        is_jump = 1 << 3,
        // is_pass_through = 1 << 3, // excl is_dead_end and done
        // is_dead_end = 1 << 4,
        alternative_is_immediate = 1 << 4,
        contains_captures = 1 << 5,
        root_pattern_guaranteed = 1 << 6,
        parent_pattern_guaranteed = 1 << 7,
    }

    struct QueryState(QS);

    impl QueryState {
        fn into(&self) -> super::QueryStep {
            let f = |x| todo!();
            let symbol;
            let supertype_symbol;
            let field;
            let capture_ids;
            let depth;
            let alternative_index;
            let negated_field_list_id;
            let bit_field: u16;
            match (self.0.depth, &self.0.flags, &self.0.u) {
                (u8::MAX, bf, U { done: _ }) => {
                    symbol = 0;
                    supertype_symbol = 0;
                    field = 0;
                    capture_ids = Default::default();
                    depth = u16::MAX;
                    alternative_index = StepId::NONE;
                    negated_field_list_id = 0;
                    bit_field = f(bf);
                }
                (d, bf, U { jump: _ }) if bf.0 & 1 << 3 != 0 => {
                    symbol = 0;
                    supertype_symbol = 0;
                    field = 0;
                    capture_ids = Default::default();
                    depth = u16::MAX;
                    alternative_index = StepId::NONE;
                    negated_field_list_id = 0;
                    bit_field = f(bf);
                }
                (d, bf, U { alt1: _ }) => {
                    symbol = 0;
                    supertype_symbol = 0;
                    field = 0;
                    capture_ids = Default::default();
                    depth = u16::MAX;
                    alternative_index = StepId::NONE;
                    negated_field_list_id = 0;
                    bit_field = f(bf);
                }
                (d, bf, U { alt2: _ }) => {
                    symbol = 0;
                    supertype_symbol = 0;
                    field = 0;
                    capture_ids = Default::default();
                    depth = u16::MAX;
                    alternative_index = StepId::NONE;
                    negated_field_list_id = 0;
                    bit_field = f(bf);
                }
                _ => panic!(),
            }
            super::QueryStep {
                symbol,
                supertype_symbol,
                field,
                capture_ids,
                depth,
                alternative_index,
                negated_field_list_id,
                bit_field,
            }
        }
    }

    struct QS {
        depth: u8,
        flags: F,
        // alt: u8,
        u: U,
    }

    const D_ALT1: u8 = u8::MAX - 1;
    const D_ALT2: u8 = u8::MAX - 2;

    struct F(u8);

    union U {
        done: Done,
        jump: Jump,
        // all: All,
        alt1: Alt1,
        alt2: Alt2,
    }
    #[derive(Copy, Clone)]
    struct Done {}
    #[derive(Copy, Clone)]
    struct Jump {
        alternative_index: StepId,
        pass_through: bool,
    }
    #[derive(Copy, Clone)]
    struct All {
        // optional when done variant
        symbol: ffi::TSSymbol,
        // optional
        supertype_symbol: ffi::TSSymbol,
        // optional
        field: ffi::TSFieldId,
        // optional
        capture_ids: [CaptureId; MAX_STEP_CAPTURE_COUNT],
        // optional, madatory when dead_end or pass_through
        alternative_index: StepId,
        // optional
        negated_field_list_id: u16,
    }
    #[derive(Copy, Clone)]
    struct Alt1 {
        // optional when done variant
        symbol: ffi::TSSymbol,
        // optional
        supertype_symbol: ffi::TSSymbol,
        // optional
        field: ffi::TSFieldId,
        // optional
        capture_ids: (),
        // optional, madatory when dead_end or pass_through
        alternative_index: StepId,
        // optional
        negated_field_list_id: u16,
    }
    #[derive(Copy, Clone)]
    struct Alt2 {
        // optional when done variant
        symbol: ffi::TSSymbol,
        // optional
        supertype_symbol: ffi::TSSymbol,
        // optional
        field: ffi::TSFieldId,
        // optional
        capture_ids: [CaptureId; 2],
        // optional, madatory when dead_end or pass_through
        alternative_index: (),
        // optional
        negated_field_list_id: (),
    }
    #[derive(Copy, Clone)]
    struct Alt3 {
        // optional when done variant
        symbol: ffi::TSSymbol,
        // optional
        supertype_symbol: ffi::TSSymbol,
        // optional
        field: (),
        // optional
        capture_ids: [CaptureId; MAX_STEP_CAPTURE_COUNT],
        // optional, madatory when dead_end or pass_through
        alternative_index: (),
        // optional
        negated_field_list_id: (),
    }
    #[derive(Copy, Clone)]
    struct Left {
        // optional when done variant
        symbol: ffi::TSSymbol,
        // optional
        supertype_symbol: ffi::TSSymbol,
        // optional
        field: ffi::TSFieldId,
        // optional, madatory when dead_end or pass_through
        alternative_index: StepId,
    }
    #[derive(Copy, Clone)]
    struct Right {
        // optional
        capture_ids: [CaptureId; MAX_STEP_CAPTURE_COUNT],
        // optional
        negated_field_list_id: u16,
    }
}
