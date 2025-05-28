use super::Capture;
use super::Node;
use super::indexed::CaptureListId;
use super::query::PatternEntry;
use super::{QueryCursor, QueryMatch, Status, TreeCursorStep};
use crate::Depth;
use crate::indexed::StepId;
use crate::indexed::{CaptureId, PatternId};

#[derive(Clone)]
pub struct State {
    pub(crate) id: super::indexed::StateId,
    pub(crate) capture_list_id: super::indexed::CaptureListId,
    pub(crate) start_depth: u16,
    pub(crate) step_index: super::indexed::StepId,
    pub(crate) pattern_index: PatternId,
    pub(crate) bitfield: u16,
    // consumed_capture_count: u16, // 12
    // seeking_immediate_match: bool, // 1
    // has_in_progress_alternatives: bool, // 1
    // dead: bool, // 1
    // needs_parent: bool, // 1
}

impl State {
    pub(crate) const SEEKING_IMMEDIATE_MATCH: u16 = 1 << (12 + 0);
    pub(crate) const IN_PROGRESS_ALTERNATIVES: u16 = 1 << (12 + 1);
    pub(crate) const DEAD: u16 = 1 << (12 + 2);
    pub(crate) const NEEDS_PARENT: u16 = 1 << (12 + 3);
    pub(crate) fn new(pattern: &PatternEntry, start_depth: Depth, depth: Depth) -> Self {
        let mut bitfield = Self::SEEKING_IMMEDIATE_MATCH;
        if depth == 1 {
            bitfield |= Self::NEEDS_PARENT;
        }
        // consumed_capture_count: 0,
        // seeking_immediate_match: true,
        // has_in_progress_alternatives: false,
        // needs_parent: step.depth == 1,
        // dead: false,
        assert!(start_depth < u16::MAX as u32);
        Self {
            id: crate::indexed::StateId::MAX,
            capture_list_id: super::indexed::CaptureListId::MAX,
            step_index: pattern.step_index,
            pattern_index: pattern.pattern_index,
            start_depth: start_depth as u16,
            bitfield,
        }
    }
    pub(crate) const fn pattern_index(&self) -> crate::indexed::PatternId {
        self.pattern_index
    }
    pub(crate) const fn consumed_capture_count(&self) -> u16 {
        self.bitfield & 4095 // ie 12 ones
    }
    pub(crate) fn inc_consumed_capture_count(&mut self) {
        // VALIDITY: consumed_capture_count is on the least significant digit.
        self.bitfield += 1
    }
    pub(crate) const fn is_seeking_immediate_match(&self) -> bool {
        self.bitfield & Self::SEEKING_IMMEDIATE_MATCH != 0
    }
    pub(crate) fn seeking_immediate_match(&mut self) {
        self.bitfield |= Self::SEEKING_IMMEDIATE_MATCH
    }
    pub(crate) fn not_seeking_immediate_match(&mut self) {
        self.bitfield &= !Self::SEEKING_IMMEDIATE_MATCH
    }
    pub(crate) const fn has_in_progress_alternatives(&self) -> bool {
        self.bitfield & Self::IN_PROGRESS_ALTERNATIVES != 0
    }
    pub(crate) fn in_progress_alternatives(&mut self) {
        self.bitfield |= Self::IN_PROGRESS_ALTERNATIVES
    }
    pub(crate) fn no_in_progress_alternatives(&mut self) {
        self.bitfield &= !Self::IN_PROGRESS_ALTERNATIVES
    }
    pub(crate) const fn dead(&self) -> bool {
        self.bitfield & Self::DEAD != 0
    }
    pub(crate) fn kill(&mut self) {
        self.bitfield |= Self::DEAD
    }
    pub(crate) const fn does_need_parent(&self) -> bool {
        self.bitfield & Self::NEEDS_PARENT != 0
    }
    pub(crate) fn no_need_parent(&mut self) {
        self.bitfield &= !Self::NEEDS_PARENT
    }

    fn start_depth(&self) -> Depth {
        self.start_depth as u32
    }
}

#[test]
pub(crate) fn state_bitfield() {
    println!("{:b}", 1 << 12);
    println!("{:b}", 0b01111_1111_1111);
    println!("{}", 0b01111_1111_1111);
}

// Walk the tree, processing patterns until at least one pattern finishes,
// If one or more patterns finish, return `true` and store their states in the
// `finished_states` array. Multiple patterns can finish on the same node. If
// there are no more matches, return `false`.
impl<'query, Cursor: super::Cursor> QueryCursor<'query, Cursor, Cursor::Node>
where
    <Cursor::Status as Status>::IdF: Into<u16> + From<u16>,
{
    #[allow(unused)]
    pub(crate) fn advance(&mut self, stop_on_definite_step: bool) -> bool {
        let mut did_match = false;
        loop {
            // dbg!();
            if self.halted {
                while (self.states.len() > 0) {
                    let state = self.states.pop().unwrap();
                    self.capture_list_pool.release(state.capture_list_id);
                }
            }

            if did_match || self.halted {
                return did_match;
            }

            // Exit the current node.
            if self.ascending {
                // dbg!();
                did_match |= self.when_ascending();
                // Leave this node by stepping to its next sibling or to its parent.
                match self.cursor.goto_next_sibling_internal() {
                    TreeCursorStep::TreeCursorStepVisible => {
                        if !self.on_visible_node {
                            self.depth += 1;
                            self.on_visible_node = true;
                        }
                        self.ascending = false;
                    }
                    TreeCursorStep::TreeCursorStepHidden => {
                        if self.on_visible_node {
                            if self.depth == 0 {
                                return did_match;
                            }
                            self.depth -= 1;
                            self.on_visible_node = false;
                        }
                        self.ascending = false;
                    }
                    TreeCursorStep::TreeCursorStepNone => {
                        if self.cursor.goto_parent() {
                            if self.depth == 0 {
                                return did_match;
                            }
                            self.depth -= 1;
                        } else {
                            log::trace!("halt at root");
                            self.halted = true;
                        }
                    }
                }
            }
            // Enter a new node.
            else {
                // dbg!();
                let (m, node_intersects_range) = self.when_entering(stop_on_definite_step);
                did_match |= m;
                if self.should_descend(node_intersects_range) {
                    // dbg!(self.cursor.current_node().str_symbol());
                    match self.cursor.goto_first_child_internal() {
                        TreeCursorStep::TreeCursorStepVisible => {
                            // dbg!(self.cursor.current_node().str_symbol());
                            self.depth += 1;
                            self.on_visible_node = true;
                            continue;
                        }
                        TreeCursorStep::TreeCursorStepHidden => {
                            // dbg!(self.cursor.current_node().str_symbol());
                            self.on_visible_node = false;
                            continue;
                        }
                        TreeCursorStep::TreeCursorStepNone => (),
                    }
                }
                self.ascending = true;
            }
        }
    }

    #[allow(unreachable_code)]
    pub(crate) fn when_ascending(&mut self) -> bool {
        let mut did_match = false;
        if self.on_visible_node {
            log::trace!(
                "leave node. depth:{}, type:{}\n",
                self.depth,
                self.cursor.current_node().str_symbol()
            );
            let steps = &self.query.steps;
            // After leaving a node, remove any states that cannot make further progress.
            self.states = std::mem::take(&mut self.states)
                .into_iter()
                .filter_map(|state| {
                    let step = &steps[state.step_index];

                    // If a state completed its pattern inside of this node, but was deferred from finishing
                    // in order to search for longer matches, mark it as finished.
                    if step.done() && (state.start_depth() > self.depth || self.depth == 0) {
                        log::trace!("  finish pattern {}", state.pattern_index);
                        self.finished_states.push_back(state);
                        did_match = true;
                        None
                    }
                    // If a state needed to match something within this node, then remove that state
                    // as it has failed to match.
                    else if !step.done() && state.start_depth() + step.depth() > self.depth {
                        log::trace!(
                            "  failed to match. pattern:{}, step:{}",
                            state.pattern_index,
                            state.step_index
                        );
                        self.capture_list_pool.release(state.capture_list_id);
                        drop(state);
                        None
                    } else {
                        Some(state)
                    }
                })
                .collect();
        }
        did_match
    }

    pub(crate) fn when_entering(&mut self, stop_on_definite_step: bool) -> (bool, bool) {
        let query = self.query;
        let mut did_match = false;
        let node_intersects_range;
        let parent_intersects_range;
        {
            // // Get the properties of the current node.
            // let node = self.cursor.current_node();
            // let parent_node = self.cursor.parent_node();

            let parent_precedes_range = false;
            // !ts_node_is_null(parent_node) && (
            //   ts_node_end_byte(parent_node) <= self->start_byte ||
            //   point_lte(ts_node_end_point(parent_node), self->start_point)
            // );
            let parent_follows_range = false;
            // !ts_node_is_null(parent_node) && (
            //   ts_node_start_byte(parent_node) >= self->end_byte ||
            //   point_gte(ts_node_start_point(parent_node), self->end_point)
            // );
            let node_precedes_range = false;
            // parent_precedes_range || (
            //   ts_node_end_byte(node) <= self->start_byte ||
            //   point_lte(ts_node_end_point(node), self->start_point)
            // );
            let node_follows_range = false;
            // parent_follows_range || (
            //   ts_node_start_byte(node) >= self->end_byte ||
            //   point_gte(ts_node_start_point(node), self->end_point)
            // );

            parent_intersects_range = !parent_precedes_range && !parent_follows_range;
            node_intersects_range = !node_precedes_range && !node_follows_range
        };

        if self.on_visible_node {
            // Get the properties of the current node.
            let node = self.cursor.current_node();
            let symbol = node.symbol();
            let is_named = node.is_named();
            let status = self.cursor.current_status();
            log::trace!(
                "enter node. depth:{}, type:{}, field:{}, row:{} state_count:{}, finished_state_count:{}",
                self.depth,
                node.str_symbol(),
                query.field_name(status.field_id().into()),
                node.start_point().row,
                self.states.len(),
                self.finished_states.len()
            );

            let node_is_error = symbol.is_error();
            let parent_is_error = self.cursor.parent_is_error();

            drop(node);

            // Add new states for any patterns whose root node is a wildcard.
            if !node_is_error {
                for i in 0..query.wildcard_root_pattern_count {
                    let pattern = &query.pattern_map[i as usize];

                    // If this node matches the first step of the pattern, then add a new
                    // state at the start of this pattern.
                    let step = &query.steps[pattern.step_index];
                    if step.done() {
                        // to level predicates are kind of considered as patterns...
                        // for now this mitigation is fine and avoid further deviation from ref. impl.
                        continue;
                    }
                    let start_depth = self.depth - step.depth();
                    let mut should_add = if pattern.is_rooted {
                        node_intersects_range
                    } else {
                        parent_intersects_range && !parent_is_error
                    };
                    should_add &= step.constrained(status.field_id().into())
                        && (step.supertype_symbol().is_some()
                            || status.has_supertypes()
                            || self.max_start_depth == 0)
                        && (start_depth <= self.max_start_depth);
                    if should_add {
                        self.add_state(pattern);
                    }
                }
            }
            if self.max_start_depth == 0 {
                for pattern in &query.pattern_map2 {
                    self.add_state(pattern);
                }
            }
            // Add new states for any patterns whose root node matches this node.
            if let Some(mut i) = query.pattern_map_search(symbol) {
                let mut pattern = &query.pattern_map[i];
                let mut pat = &query.patterns[pattern.pattern_index];
                let mut step = &query.steps[pattern.step_index];
                let start_depth = self.depth.wrapping_sub(step.depth());
                loop {
                    // If this node matches the first step of the pattern, then add a new
                    // state at the start of this pattern.
                    let can_start = if pattern.is_rooted {
                        node_intersects_range
                    } else {
                        parent_intersects_range && !parent_is_error
                    } && step.constrained(status.field_id().into())
                        && (start_depth <= self.max_start_depth);
                    if can_start {
                        if Self::wont_match(pattern, &self.cursor, pat) {
                            log::trace!(
                                "don't start. type:{}",
                                self.cursor.current_node().str_symbol()
                            );
                        } else {
                            self.add_state(pattern);
                        }
                    }

                    // Advance to the next pattern whose root node matches this node.
                    i += 1;
                    if i == query.pattern_map.len() {
                        break;
                    };
                    pattern = &query.pattern_map[i];
                    pat = &query.patterns[pattern.pattern_index];
                    step = &query.steps[pattern.step_index];
                    if !step.is(symbol) {
                        break;
                    }
                }
            }

            let mut j = 0;
            let mut copy_count = 0;
            let mut _next = 0;
            // Update all of the in-progress states with current node.
            while j < self.states.len() {
                let mut _j = j;
                // let state = &mut self.states[j];
                // let step = &mut query.steps[state!().step_index as usize];
                macro_rules! state {
                    ($i:expr) => {
                        self.states[$i]
                    };
                    () => {
                        self.states[_j]
                    };
                    (@index) => {
                        _j
                    };
                    (@step $i:expr) => {
                        query.steps[state!($i).step_index]
                    };
                    (@step) => {
                        query.steps[state!().step_index]
                    };
                }
                state!().no_in_progress_alternatives();
                copy_count = 0;

                // Check that the node matches all of the criteria for the next
                // step of the pattern.
                if state!(@step).done()
                    || state!().start_depth() + state!(@step).depth() != self.depth
                {
                    j += 1 + copy_count;
                    continue;
                }

                // Get the properties of the current node.
                let node = self.cursor.current_node();

                // Determine if this node matches this step of the pattern, and also
                // if this node can have later siblings that match this step of the
                // pattern.
                let mut node_does_match;
                if state!(@step).is_wildcard() {
                    node_does_match = !node_is_error && (is_named || !state!(@step).is_named());
                } else {
                    node_does_match = state!(@step).is(symbol);
                }
                let mut later_sibling_can_match = status.has_later_siblings();
                if (state!(@step).is_immediate() && is_named)
                    || state!().is_seeking_immediate_match()
                {
                    later_sibling_can_match = false;
                }
                if state!(@step).is_last_child() && status.has_later_named_siblings() {
                    node_does_match = false;
                }
                if let Some(ss) = state!(@step).supertype_symbol() {
                    let has_supertype = status.contains_supertype(ss);
                    if !has_supertype {
                        node_does_match = false
                    };
                }
                if let Some(field) = state!(@step).field() {
                    if field == status.field_id().into() {
                        if !status.can_have_later_siblings_with_this_field() {
                            later_sibling_can_match = false;
                        }
                    } else {
                        node_does_match = false;
                    }
                }

                if let Some(negated_field_list_id) = state!(@step).negated_field_list_id() {
                    for negated_field_id in self.query.negated_fields.get(negated_field_list_id) {
                        if node.has_child_with_field_id(negated_field_id.into()) {
                            node_does_match = false;
                            break;
                        }
                    }
                }

                // TODO is_empty but would need to know if there are named children/descendants
                // NOTE is_neg would also be useful in other cases but lets first make some test with alternative ways ie enumerate exhaustively the positive cases
                // if state!(@step).is_neg() {

                //     dbg!(&state!(@step), node_does_match, is_named);
                //     if is_named {
                //         node_does_match = !node_does_match;
                //     }
                //     // dbg!(symbol);
                // }

                if node_does_match {
                    if let Some(pred_id) = state!(@step).immediate_pred() {
                        let pred = &self.query.immediate_predicates[pred_id as usize];
                        match pred {
                            crate::predicate::ImmediateTextPredicate::EqString {
                                str,
                                is_named,
                                is_positive,
                            } => {
                                assert!(is_positive);
                                let current_node = &self.cursor.current_node();
                                let t = current_node.text(self.cursor.text_provider());
                                if t.as_bytes() != str.as_bytes() {
                                    node_does_match = false;
                                }
                            }
                            crate::predicate::ImmediateTextPredicate::MatchString { re } => todo!(),
                            crate::predicate::ImmediateTextPredicate::MatchStringUnamed { re } => {
                                todo!()
                            }
                            crate::predicate::ImmediateTextPredicate::AnyString(_) => todo!(),
                        }
                    }
                }
                drop(node);

                // Remove states immediately if it is ever clear that they cannot match.
                if !node_does_match {
                    if !later_sibling_can_match {
                        log::trace!(
                            "  discard state. pattern:{}, step:{}",
                            state!().pattern_index,
                            state!().step_index
                        );
                        self.capture_list_pool.release(state!().capture_list_id);
                        self.states.remove(j);
                        j += copy_count;
                    } else {
                        j += 1 + copy_count;
                    }
                    continue;
                }

                // Some patterns can match their root node in multiple ways, capturing different
                // children. If this pattern step could match later children within the same
                // parent, then this query state cannot simply be updated in place. It must be
                // split into two states: one that matches this node, and one which skips over
                // this node, to preserve the possibility of matching later siblings.
                if later_sibling_can_match
                    && (state!(@step).contains_captures()
                        || query.step_is_fallible(state!().step_index))
                {
                    if self.copy_state(&mut state!(@index)).is_some() {
                        // TODO check if it properly passes a double pointer
                        log::trace!(
                            "  split state for capture. pattern:{}, step:{} {} {} {}",
                            state!().pattern_index,
                            state!().step_index,
                            later_sibling_can_match,
                            state!(@step).contains_captures(),
                            query.step_is_fallible(state!().step_index),
                        );
                        copy_count += 1;
                    }
                }

                // If this pattern started with a wildcard, such that the pattern map
                // actually points to the *second* step of the pattern, then check
                // that the node has a parent, and capture the parent node if necessary.
                if state!().does_need_parent() {
                    if self.cursor.has_parent() {
                        state!().no_need_parent();
                        let mut skipped_wildcard = state!().step_index;
                        while skipped_wildcard.dec() {
                            if query.steps[skipped_wildcard].is_dead_end()
                                || query.steps[skipped_wildcard].is_pass_through()
                                || query.steps[skipped_wildcard].depth() > 0
                            {
                                continue;
                            }
                            if query.steps[skipped_wildcard].has_capture_ids() {
                                log::trace!("  capture wildcard parent");
                                self.capture(state!(@index), skipped_wildcard, true);
                            }
                            break;
                        }
                    } else {
                        log::trace!("  missing parent node");
                        state!().kill();
                    }
                }

                // If the current node is captured in this pattern, add it to the capture list.
                if state!(@step).has_capture_ids() {
                    self.capture(state!(@index), state!().step_index, false);
                }

                if state!().dead() {
                    self.states.remove(j);
                    j += copy_count;
                    continue;
                }

                // Advance this state to the next step of its pattern.
                state!().step_index.inc();
                state!().not_seeking_immediate_match();
                log::trace!(
                    "  advance state. pattern:{}, step:{}",
                    state!().pattern_index,
                    state!().step_index
                );

                // let next_step = &mut self.query.steps[state!().step_index as usize];
                if state!(@step).root_pattern_guaranteed() {
                    did_match |= stop_on_definite_step
                }

                // If this state's next step has an alternative step, then copy the state in order
                // to pursue both alternatives. The alternative step itself may have an alternative,
                // so this is an interactive process.
                let mut end_index = j + 1;
                let mut k = j;
                while k < end_index {
                    //   QueryState *child_state = &self->states.contents[k];
                    //   QueryStep *child_step = &self->query->steps.contents[child_state->step_index];
                    let mut _k = k;
                    let _s = state!(_k).step_index;
                    // let mut child_step = &mut self.query.steps[self.states[_k].step_index as usize];
                    // let mut child_state = &mut self.states[_k];
                    // dbg!(self.states.len());
                    // dbg!(_j, _k);
                    // dbg!(state!(_k).step_index);
                    // dbg!(state!(@step).alternative_index);
                    // dbg!(state!(@step _k).alternative_index);
                    if let Some(alternative_index) = state!(@step _k).alternative_index() {
                        // A "dead-end" step exists only to add a non-sequential jump into the step sequence,
                        // via its alternative index. When a state reaches a dead-end step, it jumps straight
                        // to the step's alternative.
                        if state!(@step _k).is_dead_end() {
                            state!(_k).step_index = alternative_index;
                            continue;
                        }

                        // A "pass-through" step exists only to add a branch into the step sequence,
                        // via its alternative_index. When a state reaches a pass-through step, it splits
                        // in order to process the alternative step, and then it advances to the next step.
                        let pt = state!(@step _k).is_pass_through();
                        if pt {
                            // dbg!();
                            state!(_k).step_index.inc();
                        }

                        if let Some(_copy) = self.copy_state(&mut _k) {
                            log::trace!(
                                "  split state for branch. pattern:{}, from_step:{}, to_step:{}, immediate:{}, capture_count: {}",
                                state!(_copy).pattern_index,
                                state!(_copy).step_index,
                                state!(@step).alternative_index().unwrap_or(StepId::NONE),
                                state!(@step).alternative_is_immediate(),
                                self.capture_list_pool
                                    .get(state!(_copy).capture_list_id)
                                    .len()
                            );
                            end_index += 1;
                            copy_count += 1;
                            // dbg!(state!(_k).step_index, _k, _copy);
                            // dbg!(state!(@step _k).alternative_index);
                            state!(_copy).step_index = query.steps[_s].alternative_index().unwrap();
                            if query.steps[_s].alternative_is_immediate() {
                                state!(_copy).seeking_immediate_match();
                            }
                        }
                        if !pt {
                            k += 1;
                        }
                    } else {
                        k += 1;
                    }
                }
                j += 1 + copy_count;
            }

            let mut j = 0;
            while j < self.states.len() {
                let _j = j;
                macro_rules! curr_state {
                    () => {
                        self.states[_j]
                    };
                }
                // let mut state = &mut self.states[j];
                if curr_state!().dead() {
                    // array_erase(&self->states, j);
                    self.states.remove(j);
                    continue;
                }

                // Enforce the longest-match criteria. When a query pattern contains optional or
                // repeated nodes, this is necessary to avoid multiple redundant states, where
                // one state has a strict subset of another state's captures.
                let mut did_remove = false;
                let mut k = j + 1;
                while k < self.states.len() {
                    let _k = k;
                    macro_rules! other_state {
                        () => {
                            self.states[_k]
                        };
                    }
                    // let other_state = &mut self.states[k];

                    // Query states are kept in ascending order of start_depth and pattern_index.
                    // Since the longest-match criteria is only used for deduping matches of the same
                    // pattern and root node, we only need to perform pairwise comparisons within a
                    // small slice of the states array.
                    if other_state!().start_depth != curr_state!().start_depth
                        || other_state!().pattern_index != curr_state!().pattern_index
                    {
                        break;
                    }

                    let (left_contains_right, right_contains_left) =
                        self.compare_captures(&curr_state!(), &other_state!());
                    if left_contains_right {
                        if curr_state!().step_index == other_state!().step_index {
                            log::trace!(
                                "  drop shorter state. pattern: {}, step_index: {}",
                                curr_state!().pattern_index,
                                curr_state!().step_index
                            );
                            self.capture_list_pool
                                .release(other_state!().capture_list_id);
                            self.states.remove(k);
                            continue;
                        }
                        other_state!().in_progress_alternatives();
                    }
                    if right_contains_left {
                        if curr_state!().step_index == other_state!().step_index {
                            log::trace!(
                                "  drop shorter state. pattern: {}, step_index: {}",
                                curr_state!().pattern_index,
                                curr_state!().step_index
                            );
                            self.capture_list_pool
                                .release(curr_state!().capture_list_id);
                            self.states.remove(j);
                            did_remove = true;
                            break;
                        }
                        curr_state!().in_progress_alternatives();
                    }
                    k += 1;
                }

                // If the state is at the end of its pattern, remove it from the list
                // of in-progress states and add it to the list of finished states.
                if !did_remove {
                    log::trace!(
                        "  keep state. pattern: {}, start_depth: {}, step_index: {}, capture_count: {}",
                        curr_state!().pattern_index,
                        curr_state!().start_depth,
                        curr_state!().step_index,
                        self.capture_list_pool
                            .get(curr_state!().capture_list_id)
                            .len()
                    );
                    let next_step = &query.steps[curr_state!().step_index];
                    if next_step.done() {
                        if curr_state!().has_in_progress_alternatives() {
                            log::trace!(
                                "  defer finishing pattern {}",
                                curr_state!().pattern_index
                            );
                            j += 1 + copy_count;
                        } else {
                            log::trace!("  finishing pattern {}", curr_state!().pattern_index);
                            // array_push(&self->finished_states, *state);
                            // array_erase(&self->states, (uint32_t)(state - self->states.contents));
                            self.finished_states.push_back(self.states.remove(_j));
                            did_match = true;
                            j += copy_count;
                        }
                    } else {
                        j += 1 + copy_count;
                    }
                } else {
                    j += 1 + copy_count;
                }
            }
        }

        (did_match, node_intersects_range)
    }

    pub fn next_match(&mut self) -> Option<QueryMatch<Cursor::Node>> {
        if self.finished_states.len() == 0 {
            // dbg!();
            if !self.advance(false) {
                return None;
            }
        }

        let mut state = self.finished_states.pop_front().unwrap();
        if state.id == crate::indexed::StateId::MAX {
            state.id = self.next_state_id;
            self.next_state_id.inc();
        };
        let id = state.id;
        let pattern_index = state.pattern_index();
        let captures = self.capture_list_pool.pop(state.capture_list_id);
        return Some(QueryMatch {
            pattern_index,
            captures,
            id,
        });
    }

    fn wont_match(
        pattern: &PatternEntry,
        cursor: &Cursor,
        pat: &crate::query::QueryPattern,
    ) -> bool {
        let Some(needed) = pattern.precomputed() else {
            return false;
        };
        // if !node.could_match(precomp) {
        //     return false;
        // }
        // let Some(precomp) = pat.precomputed() else {
        //     return true;
        // };
        cursor.wont_match(needed)
    }
}

impl<'query, Cursor, Node> QueryCursor<'query, Cursor, Node> {
    /// Set the max depth where queries can start being matched
    /// For example, set it to 0 to only match on the node you start on.
    pub fn set_max_start_depth(&mut self, max: u32) {
        self.max_start_depth = max;
    }
    pub fn _next_match(&mut self) -> Option<QueryMatch<Cursor>> {
        todo!()
    }
}

impl<'query, Cursor: super::Cursor> QueryCursor<'query, Cursor, Cursor::Node> {
    fn should_descend(&self, node_intersects_range: bool) -> bool {
        if node_intersects_range && self.depth < self.max_start_depth {
            if self.cursor.wont_match(self.query.used_precomputed) {
                for i in 0..self.states.len() {
                    let state = &self.states[i];
                    let next_step = &self.query.steps[state.step_index];
                    if !next_step.done() && state.start_depth() + next_step.depth() > self.depth {
                        return true;
                    }
                }
                // not sound to skip an hidden node just under root
                // TODO find a way of making it sound
                if self.depth == 0 {
                    return true;
                }
                log::trace!(
                    "skip subtree. type:{}",
                    self.cursor.current_node().str_symbol()
                );
                return false;
            }
            return true;
        }
        // If there are in-progress matches whose remaining steps occur
        // deeper in the tree, then descend.
        for i in 0..self.states.len() {
            let state = &self.states[i];
            let next_step = &self.query.steps[state.step_index];
            if !next_step.done() && state.start_depth() + next_step.depth() > self.depth {
                return true;
            }
        }
        if self.depth >= self.max_start_depth {
            // dbg!(self.depth, self.max_start_depth);
            return false;
        }
        if self.cursor.wont_match(self.query.used_precomputed) {
            return false;
        }

        // If the current node is hidden, then a non-rooted pattern might match
        // one if its roots inside of this node, and match another of its roots
        // as part of a sibling node, so we may need to descend.
        if !self.on_visible_node {
            // TODO ts_subtree_is_repetition and ts_subtree_symbol are not in the tree_sitter header :/
            // // Descending into a repetition node outside of the range can be
            // // expensive, because these nodes can have many visible children.
            // // Avoid descending into repetition nodes unless we have already
            // // determined that this query can match rootless patterns inside
            // // of this type of repetition node.
            // if self.cursor.is_sutree_repetition() {
            //     return unsafe { &(*self.query).repeat_symbols_with_rootless_patterns }
            //         .search_sorted_by(|s| *s, self.cursor.sutree_symbol())
            //         .is_some();
            // }

            return true;
        }

        return false;
    }

    fn copy_state(&mut self, state_index: &mut usize) -> Option<usize> {
        let state = &self.states[*state_index];
        let capture_list_id = state.capture_list_id;
        let mut copy = state.clone();
        copy.capture_list_id = super::indexed::CaptureListId::MAX;

        self.states.insert(*state_index + 1, copy);
        // If the state has captures, copy its capture list.
        if capture_list_id != super::indexed::CaptureListId::MAX {
            let new_captures = self.prepare_to_capture(*state_index + 1, *state_index as u32)?;
            let old_captures = self.capture_list_pool.get(capture_list_id);
            self.capture_list_pool[new_captures] = old_captures.to_owned();
        }
        return Some(*state_index + 1);
    }

    fn compare_captures(&self, left_state: &State, right_state: &State) -> (bool, bool) {
        let left_captures = self.capture_list_pool.get(left_state.capture_list_id);
        let right_captures = self.capture_list_pool.get(right_state.capture_list_id);
        let mut left_contains_right = true;
        let mut right_contains_left = true;
        let mut i = CaptureId::ZERO;
        let mut j = CaptureId::ZERO;
        loop {
            if left_captures.contains(i) {
                if right_captures.contains(j) {
                    let left = &left_captures[i];
                    let right = &right_captures[j];
                    if left.node == right.node && left.index == right.index {
                        i.inc();
                        j.inc();
                    } else {
                        match left.node.compare(&right.node) {
                            std::cmp::Ordering::Less => {
                                right_contains_left = false;
                                i.inc();
                            }
                            std::cmp::Ordering::Greater => {
                                left_contains_right = false;
                                j.inc();
                            }
                            std::cmp::Ordering::Equal => {
                                right_contains_left = false;
                                left_contains_right = false;
                                i.inc();
                                j.inc();
                            }
                        }
                    }
                } else {
                    right_contains_left = false;
                    break;
                }
            } else {
                if right_captures.contains(j) {
                    left_contains_right = false;
                }
                break;
            }
        }
        (left_contains_right, right_contains_left)
    }

    fn first_in_progress_capture(
        &self,
        root_pattern_guaranteed: &mut bool,
    ) -> Option<(u32, u32, u32)> {
        let mut result = false;
        let mut state_index = u32::MAX;
        let mut byte_offset = u32::MAX;
        let mut pattern_index = u32::MAX;
        for i in 0..self.states.len() {
            let state = &self.states[i];
            if state.dead() {
                continue;
            };

            let captures = self.capture_list_pool.get(state.capture_list_id);
            if state.consumed_capture_count() as usize >= captures.len() {
                continue;
            }

            todo!(
                "code required for matching cartures in order instead of matches or to evict another match because we reached the max number of capture lists"
            );

            // // let node = captures[state.consumed_capture_count as usize].node;
            // // if node.end_byte() <= self.start_byte
            // //     || point_lte(node.end_point(), self.start_point)
            // // {
            // //     state.consumed_capture_count += 1;
            // //     i -= 1;
            // //     continue;
            // // }

            // // let node_start_byte = node.start_byte();
            // if !result
            //     // || node_start_byte < byte_offset
            //     || (
            //         // node_start_byte == byte_offset
            //         // &&
            //         (state.pattern_index as u32) < pattern_index
            //     )
            // {
            //     let step = &unsafe { &(*self.query).steps }[state.step_index as usize];
            //     if *root_pattern_guaranteed {
            //         *root_pattern_guaranteed = step.root_pattern_guaranteed();
            //     } else if step.root_pattern_guaranteed() {
            //         continue;
            //     }

            //     result = true;
            //     state_index = i as u32;
            //     byte_offset = 0; // TODO node_start_byte;
            //     pattern_index = state.pattern_index as u32;
            // }
        }
        result.then_some((state_index, byte_offset, pattern_index))
    }

    fn prepare_to_capture(
        &mut self,
        state_id: usize,
        state_index_to_preserve: u32,
    ) -> Option<CaptureListId> {
        let state = &mut self.states[state_id];
        if state.capture_list_id == super::indexed::CaptureListId::MAX {
            state.capture_list_id = self.capture_list_pool.acquire();

            // If there are no capture lists left in the pool, then terminate whichever
            // state has captured the earliest node in the document, and steal its
            // capture list.
            if state.capture_list_id == super::indexed::CaptureListId::MAX {
                self.did_exceed_match_limit = true;
                if let Some((state_index, byte_offset, pattern_index)) =
                    self.first_in_progress_capture(&mut false)
                {
                    if state_index != state_index_to_preserve {
                        log::trace!(
                            "  abandon state. index:{}, pattern:{}, offset:{}.",
                            state_index,
                            pattern_index,
                            byte_offset,
                        );
                        let other_state = &mut self.states[state_index as usize];
                        let capture_list_id = other_state.capture_list_id;
                        other_state.capture_list_id = super::indexed::CaptureListId::MAX; // TODO handle NONE size stuff...
                        other_state.kill();
                        let list = &mut self.capture_list_pool[capture_list_id];
                        list.clear();
                        let state = &mut self.states[state_id];
                        state.capture_list_id = capture_list_id;
                        return Some(capture_list_id);
                    }
                }
                log::trace!("  ran out of capture lists");
                return None;
            }
        }
        Some(state.capture_list_id)
    }

    fn capture(&mut self, state_id: usize, step_id: super::indexed::StepId, parent: bool) {
        let state = &mut self.states[state_id];
        if state.dead() {
            return;
        };
        let Some(capture_list_id) = self.prepare_to_capture(state_id, u32::MAX) else {
            let state = &mut self.states[state_id];
            state.kill();
            return;
        };
        let state = &self.states[state_id];
        let step = &self.query.steps[step_id];
        for capture_id in step.capture_ids() {
            let node = if parent {
                self.cursor.persist_parent().unwrap()
            } else {
                self.cursor.persist()
            };
            log::trace!(
                "  capture node. type:{}, pattern:{}, capture_id:{}, capture_count:{}",
                node.str_symbol(),
                state.pattern_index,
                capture_id,
                self.capture_list_pool[capture_list_id].len() + 1
            );
            self.capture_list_pool[capture_list_id].push(Capture {
                node,
                index: capture_id,
            });
        }
    }

    fn add_state(&mut self, pattern: &PatternEntry) {
        let step = &self.query.steps[pattern.step_index];
        let start_depth = self.depth - step.depth();

        // Keep the states array in ascending order of start_depth and pattern_index,
        // so that it can be processed more efficiently elsewhere. Usually, there is
        // no work to do here because of two facts:
        // * States with lower start_depth are naturally added first due to the
        //   order in which nodes are visited.
        // * Earlier patterns are naturally added first because of the ordering of the
        //   pattern_map data structure that's used to initiate matches.
        //
        // This loop is only needed in cases where two conditions hold:
        // * A pattern consists of more than one sibling node, so that its states
        //   remain in progress after exiting the node that started the match.
        // * The first node in the pattern matches against multiple nodes at the
        //   same depth.
        //
        // An example of this is the pattern '((comment)* (function))'. If multiple
        // `comment` nodes appear in a row, then we may initiate a new state for this
        // pattern while another state for the same pattern is already in progress.
        // If there are multiple patterns like this in a query, then this loop will
        // need to execute in order to keep the states ordered by pattern_index.
        let mut index = self.states.len();
        while index > 0 {
            let prev_state = &self.states[index - 1];
            if prev_state.start_depth() < start_depth {
                break;
            }
            if prev_state.start_depth() == start_depth {
                // Avoid inserting an unnecessary duplicate state, which would be
                // immediately pruned by the longest-match criteria.
                if prev_state.pattern_index == pattern.pattern_index
                    && prev_state.step_index == pattern.step_index
                {
                    return;
                };
                if prev_state.pattern_index <= pattern.pattern_index {
                    break;
                };
            }
            index -= 1;
        }

        log::trace!(
            "  start state. pattern:{}, step:{}",
            pattern.pattern_index,
            pattern.step_index
        );
        let element = State::new(pattern, start_depth, step.depth());
        self.states.insert(index, element);
    }
}
