// static const TSQueryError PARENT_DONE = -1;
// static const uint16_t PATTERN_DONE_MARKER = UINT16_MAX;
const PATTERN_DONE_MARKER: u16 = u16::MAX; // TODO
                                           // static const uint16_t NONE = UINT16_MAX;
const NONE: u16 = u16::MAX; // TODO
                            // static const TSSymbol WILDCARD_SYMBOL = 0;
const WILDCARD_SYMBOL: Symbol = Symbol(0); // TODO

// #define MAX_STEP_CAPTURE_COUNT 3
const MAX_STEP_CAPTURE_COUNT: usize = 3;
// #define MAX_NEGATED_FIELD_COUNT 8
// #define MAX_STATE_PREDECESSOR_COUNT 256
// #define MAX_ANALYSIS_STATE_DEPTH 8
const MAX_ANALYSIS_STATE_DEPTH: u16 = 8;
// #define MAX_ANALYSIS_ITERATION_COUNT 256
const MAX_ANALYSIS_ITERATION_COUNT: usize = 256;

struct StateId(usize);
struct StepId(usize);
struct CaptureListId(usize);
struct PatternId(usize);

struct Query(std::ptr::NonNull<TSQuery>);

impl Drop for Query {
    fn drop(&mut self) {
        unsafe { tree_sitter::ffi::ts_query_delete(std::mem::transmute(self.0.as_ptr())) }
    }
}

struct QueryCursor<Cursor, Node> {
    halted: bool,
    ascending: bool,
    on_visible_node: bool,
    cursor: Cursor,
    query: *const TSQuery,
    states: Vec<State>,
    depth: u32,
    max_start_depth: u32,
    capture_list_pool: CaptureListPool<Node>,
    finished_states: VecDeque<State>,
    next_state_id: u32,
    did_exceed_match_limit: bool,
}
impl<Cursor, Node> Drop for QueryCursor<Cursor, Node> {
    fn drop(&mut self) {
        unsafe { tree_sitter::ffi::ts_query_delete(std::mem::transmute(self.query)) }
    }
}

struct CaptureListPool<Node> {
    list: Vec<Vec<Capture<Node>>>,
    // The maximum number of capture lists that we are allowed to allocate. We
    // never allow `list` to allocate more entries than this, dropping pending
    // matches if needed to stay under the limit.
    max_capture_list_count: u32,
    // The number of capture lists allocated in `list` that are not currently in
    // use. We reuse those existing-but-unused capture lists before trying to
    // allocate any new ones. We use an invalid value (UINT32_MAX) for a capture
    // list's length to indicate that it's not in use.
    free_capture_list_count: u32,
}
impl<Node> Default for CaptureListPool<Node> {
    fn default() -> Self {
        Self {
            list: Default::default(),
            max_capture_list_count: u32::MAX,
            free_capture_list_count: Default::default(),
        }
    }
}
impl<Node> CaptureListPool<Node> {
    fn release(&mut self, id: u32) {
        if id as usize >= self.list.len() {
            return;
        }
        self.list[id as usize].clear();
        self.free_capture_list_count += 1;
    }
    fn get(&self, id: u32) -> &[Capture<Node>] {
        if id as usize >= self.list.len() {
            return &[];
        };
        return &self.list[id as usize];
    }
    fn pop(&mut self, id: u32) -> Vec<Capture<Node>> {
        if id as usize >= self.list.len() {
            return vec![];
        };
        let r = std::mem::take(&mut self.list[id as usize]);
        self.free_capture_list_count += 1;
        return r;
    }
    fn acquire(&mut self) -> u16 {
        // First see if any already allocated capture list is currently unused.
        if self.free_capture_list_count > 0 {
          for i in 0..self.list.len() {
            if self.list[i].len() == 0 {
                self.list[i].clear();
                self.free_capture_list_count -= 1;
                return i as u16;
            }
          }
        }
      
        // Otherwise allocate and initialize a new capture list, as long as that
        // doesn't put us over the requested maximum.
        let i = self.list.len();
        if i >= self.max_capture_list_count as usize {
          return NONE;
        }
        self.list.push(vec![]);
        // CaptureList list;
        // array_init(&list);
        // array_push(&self.list, list);
        return i as u16;
    }
}

struct Capture<Node> {
    node: Node,
    index: u32,
}

struct State {
    id: u32,
    capture_list_id: u32,
    start_depth: u16,
    step_index: u16,
    pattern_index: u16,
    consumed_capture_count: u16,
    seeking_immediate_match: bool,
    has_in_progress_alternatives: bool,
    dead: bool,
    needs_parent: bool,
}

// struct Query {
//     pattern_map: Vec<PatternEntry>,
//     steps: Vec<QueryStep>,
//     negated_fields: Vec<Vec<FieldId>>,
//     wildcard_root_pattern_count: usize,
//     language: Language,
//     step_offsets: SortedVec<StepOffset>,
// }

struct StepOffset {
    byte_offset: u32,
    step_index: u16,
}
struct Language {
    token_count: usize,
    symbol_count: usize,
}

impl Language {
    fn field_name(&self, i: FieldId) -> &'static str {
        todo!()
    }
    fn symbol_name(&self, s: Symbol) -> &'static str {
        todo!()
    }

    fn lookaheads(&self, parse_state: u16) -> tree_sitter::LookaheadIterator {
        todo!()
    }

    fn alias_at(&self, production_id: u16, child_index: u16) -> Option<Symbol> {
        todo!()
    }

    fn symbol_metadata(&self, sym: Symbol) -> &SymbolMetadata {
        todo!()
    }

    fn public_symbol_map(&self, sym: Symbol) -> Symbol {
        todo!()
    }

    fn is_hidden(&self, sym: Symbol) -> bool {
        // sym >= self.language.token_count
        todo!()
    }

    fn hidden_symbols(&self) -> impl Iterator<Item = Symbol> {
        // TSSymbol sym = (uint16_t)self->language->token_count; sym < (uint16_t)self->language->symbol_count; sym++
        (self.token_count..self.symbol_count).map(|x| {
            assert!(x < u16::MAX as usize);
            Symbol::from(x as u16)
        })
    }

    fn states(&self) -> impl Iterator<Item = tree_sitter::ffi::TSStateId> {
        // TSStateId state = 1; state < (uint16_t)self->language->state_count; state++
        todo!();
        vec![].into_iter()
    }

    fn alias_for_symbol(&self, symbol: u16) -> impl Iterator<Item = Symbol> {
        todo!();
        vec![].into_iter()
    }

    fn state_is_primary(&self, state: u16) -> bool {
        todo!()
    }

    fn state_predecessor_map_new(&self) -> StatePredecessorMap {
        todo!()
    }
}

struct StatePredecessorMap;

struct SymbolMetadata {
    visible: bool,
    named: bool,
}

struct PatternEntry {
    step_index: u16,
    pattern_index: u16,
    is_rooted: bool,
}

struct QueryStep {
    supertype_symbol: Symbol,
    symbol: Symbol,
    is_named: bool,
    is_immediate: bool,
    is_last_child: bool,
    is_dead_end: bool,
    is_pass_through: bool,
    parent_pattern_guaranteed: bool,
    root_pattern_guaranteed: bool,
    alternative_is_immediate: bool,
    contains_captures: bool,
    field: FieldId,
    capture_ids: [u16; MAX_STEP_CAPTURE_COUNT],
    depth: u16,
    alternative_index: u16,
    negated_field_list_id: u16,
}
impl TSQuery {
    fn pattern_map_search(&self, needle: Symbol) -> Option<usize> {
        dbg!(query_step::symbol_name(self, needle.0));
        let mut base_index = self.wildcard_root_pattern_count as usize;
        let mut size = self.pattern_map.len() - base_index;
        dbg!(needle.to_usize(), base_index, size);
        if size == 0 {
            return Some(base_index);
        }
        while size > 1 {
            let half_size = size / 2;
            let mid_index = base_index + half_size;
            let mid_symbol =
                self.steps[self.pattern_map[mid_index].step_index as usize].symbol as usize;
            dbg!(mid_symbol);
            dbg!(query_step::symbol_name(self, mid_symbol as u16));
            if needle.to_usize() > mid_symbol {
                base_index = mid_index
            };
            size -= half_size;
        }
        dbg!(base_index, size);
        dbg!(
            self.pattern_map[base_index].step_index,
            self.pattern_map[base_index].pattern_index
        );

        let mut symbol =
            self.steps[self.pattern_map[base_index].step_index as usize].symbol as usize;
        dbg!(symbol);
        dbg!(query_step::symbol_name(self, symbol as u16));

        if needle.to_usize() > symbol {
            base_index += 1;
            if base_index < self.pattern_map.len() {
                symbol =
                    self.steps[self.pattern_map[base_index].step_index as usize].symbol as usize;
            }
        }

        if needle.to_usize() == symbol {
            dbg!(base_index);
            Some(base_index)
        } else {
            None
        }
    }

    fn step_is_fallible(&self, step_index: u16) -> bool {
        todo!()
    }

    fn field_name(&self, field_id: FieldId) -> &str {
        query_step::field_name(self, field_id).unwrap_or("")
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
enum TreeCursorStep {
    TreeCursorStepNone,
    TreeCursorStepHidden,
    TreeCursorStepVisible,
}

trait Cursor {
    type Node: Node;
    fn goto_next_sibling_internal(&mut self) -> TreeCursorStep;

    fn goto_first_child_internal(&mut self) -> TreeCursorStep;

    fn goto_parent(&mut self) -> bool;
    fn current_node(&self) -> Self::Node;

    fn parent_node(&self) -> Option<Self::Node>;

    fn current_status(&self) -> Status;
    fn is_sutree_repetition(&self) -> bool;
    fn sutree_symbol(&self) -> tree_sitter::ffi::TSSymbol;
}

impl<'a> Cursor for tree_sitter::TreeCursor<'a> {
    type Node = tree_sitter::Node<'a>;

    fn goto_next_sibling_internal(&mut self) -> TreeCursorStep {
        extern "C" {
            pub fn ts_tree_cursor_goto_next_sibling_internal(
                self_: *mut tree_sitter::ffi::TSTreeCursor,
            ) -> TreeCursorStep;
        }
        unsafe {
            let s: *mut tree_sitter::ffi::TSTreeCursor = std::mem::transmute(self);
            ts_tree_cursor_goto_next_sibling_internal(s)
        }
    }

    fn goto_first_child_internal(&mut self) -> TreeCursorStep {
        extern "C" {
            pub fn ts_tree_cursor_goto_first_child_internal(
                self_: *mut tree_sitter::ffi::TSTreeCursor,
            ) -> TreeCursorStep;
        }
        unsafe {
            let s: *mut tree_sitter::ffi::TSTreeCursor = std::mem::transmute(self);
            ts_tree_cursor_goto_first_child_internal(s)
        }
    }

    fn goto_parent(&mut self) -> bool {
        self.goto_parent()
    }

    fn current_node(&self) -> Self::Node {
        self.node()
    }

    fn parent_node(&self) -> Option<Self::Node> {
        extern "C" {
            pub fn ts_tree_cursor_parent_node(
                self_: *mut tree_sitter::ffi::TSTreeCursor,
            ) -> tree_sitter::ffi::TSNode;
        }
        unsafe {
            let s: *mut tree_sitter::ffi::TSTreeCursor = std::mem::transmute(self);
            let n = ts_tree_cursor_parent_node(s);
            if tree_sitter::ffi::ts_node_is_null(n) {
                return None;
            }
            let n: tree_sitter::Node = std::mem::transmute(n);
            Some(n)
        }
    }
    #[inline]
    fn current_status(&self) -> Status {
        extern "C" {
            pub fn ts_tree_cursor_current_status(
                self_: *mut tree_sitter::ffi::TSTreeCursor,
                field_id: *mut tree_sitter::ffi::TSFieldId,
                has_later_siblings: *mut bool,
                has_later_named_siblings: *mut bool,
                can_have_later_siblings_with_this_field: *mut bool,
                supertypes: *mut tree_sitter::ffi::TSSymbol,
                // unsigned *
                supertype_count: *mut std::os::raw::c_uint,
            ) -> TreeCursorStep;
        }
        unsafe {
            let s: *mut tree_sitter::ffi::TSTreeCursor = std::mem::transmute(self);
            let mut field_id: tree_sitter::ffi::TSFieldId = 0;
            let mut has_later_siblings: bool = false;
            let mut has_later_named_siblings: bool = false;
            let mut can_have_later_siblings_with_this_field: bool = false;
            let mut supertype_count: u32 = 8;
            // TODO mem perf: might not be efficient, I am surious about perfs impacts of this,
            // if ffi fct is inlined maybe the allocation can be optimized out,
            // but I believe it to be inprobable.
            // It would probably be possible to opacify Status and provide just the required meth to uses
            // NOTE in query cursor supertypes is used as a set, where it is asked if its empty and if it contains symbols
            let mut supertypes =
                Vec::<tree_sitter::ffi::TSSymbol>::with_capacity(supertype_count as usize);
            {
                let supertypes = supertypes.as_mut_ptr();
                ts_tree_cursor_current_status(
                    s,
                    std::ptr::addr_of_mut!(field_id),
                    std::ptr::addr_of_mut!(has_later_siblings),
                    std::ptr::addr_of_mut!(has_later_named_siblings),
                    std::ptr::addr_of_mut!(can_have_later_siblings_with_this_field),
                    supertypes,
                    std::ptr::addr_of_mut!(supertype_count),
                );
            }
            supertypes.set_len(supertype_count as usize);
            let supertypes = supertypes.into_iter().map(Into::into).collect();
            Status {
                has_later_siblings,
                has_later_named_siblings,
                can_have_later_siblings_with_this_field,
                field_id,
                supertypes,
            }
        }
    }
    fn is_sutree_repetition(&self) -> bool {
        unimplemented!("missing fct from tree_sitter header")
        // #[repr(C)]
        // pub struct Subtree {
        //     _unused: [u8; 0],
        // }
        // extern "C" {
        //     pub fn ts_tree_cursor_current_subtree(
        //         self_: *const tree_sitter::ffi::TSTreeCursor,
        //     ) -> Subtree;
        //     pub fn ts_subtree_is_repetition(self_: Subtree) -> u32;
        // }
        // let s: *mut tree_sitter::ffi::TSTreeCursor = unsafe { std::mem::transmute(self) };
        // let s = unsafe { ts_tree_cursor_current_subtree(s) };
        // unsafe { ts_subtree_is_repetition(s) != 0 }
    }
    fn sutree_symbol(&self) -> tree_sitter::ffi::TSSymbol {
        unimplemented!("missing fct from tree_sitter header")
        // #[repr(C)]
        // pub struct Subtree {
        //     _unused: [u8; 0],
        // }
        // extern "C" {
        //     pub fn ts_tree_cursor_current_subtree(
        //         self_: *const tree_sitter::ffi::TSTreeCursor,
        //     ) -> Subtree;
        //     pub fn ts_subtree_symbol(self_: Subtree) -> tree_sitter::ffi::TSSymbol;
        // }
        // let s: *mut tree_sitter::ffi::TSTreeCursor = unsafe { std::mem::transmute(self) };
        // let s = unsafe { ts_tree_cursor_current_subtree(s) };
        // unsafe { ts_subtree_symbol(s) }
    }
}

struct Status {
    has_later_siblings: bool,
    has_later_named_siblings: bool,
    can_have_later_siblings_with_this_field: bool,
    field_id: FieldId,
    supertypes: Vec<Symbol>,
}

type FieldId = u16;

trait Node: Clone {
    fn symbol(&self) -> Symbol;

    fn is_named(&self) -> bool;
    fn str_symbol(&self) -> &str;

    fn start_point(&self) -> tree_sitter::Point;

    fn child_by_field_id(&self, negated_field_id: FieldId) -> Self;

    fn id(&self) -> usize;
}

impl<'a> Node for tree_sitter::Node<'a> {
    fn symbol(&self) -> Symbol {
        self.kind_id().into()
    }

    fn is_named(&self) -> bool {
        self.is_named()
    }

    fn str_symbol(&self) -> &str {
        self.kind()
    }

    fn start_point(&self) -> tree_sitter::Point {
        self.start_position()
    }

    fn child_by_field_id(&self, field_id: FieldId) -> Self {
        self.child_by_field_id(field_id).unwrap()
    }

    fn id(&self) -> usize {
        self.id()
    }
}

struct Field {
    id: usize,
}

#[derive(PartialEq, Eq, Clone, Copy)]
struct Symbol(u16);

impl Symbol {
    const ERROR: Symbol = Symbol(u16::MAX - 1);
    const NONE: Symbol = Symbol(u16::MAX);
    const END: Symbol = Symbol(0);

    fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

impl From<u16> for Symbol {
    fn from(value: u16) -> Self {
        Symbol(value)
    }
}

// mod analyze;

mod convert;

mod query_cursor;

impl<Cursor: self::Cursor> QueryCursor<Cursor, Cursor::Node> {
    fn should_descend(&self, node_intersects_range: bool) -> bool {
        if node_intersects_range && self.depth < self.max_start_depth {
            return true;
        }

        // If there are in-progress matches whose remaining steps occur
        // deeper in the tree, then descend.
        for i in 0..self.states.len() {
            let state = &self.states[i];
            let next_step = &unsafe { &(*self.query).steps }[state.step_index as usize];
            if next_step.depth != PATTERN_DONE_MARKER
                && state.start_depth as u32 + next_step.depth as u32 > self.depth
            {
                return true;
            }
        }

        if self.depth >= self.max_start_depth {
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

    fn copy_state(&self, state_index: &mut usize) -> Option<usize> {
        todo!()
    }

    fn compare_captures(&self, state: &State, other_state: &State) -> (bool, bool) {
        todo!()
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
            if state.dead {
                continue;
            };

            let captures = self.capture_list_pool.get(state.capture_list_id);
            if state.consumed_capture_count as usize >= captures.len() {
                continue;
            }

            todo!();

            // let node = captures[state.consumed_capture_count as usize].node;
            // if node.end_byte() <= self.start_byte
            //     || point_lte(node.end_point(), self.start_point)
            // {
            //     state.consumed_capture_count += 1;
            //     i -= 1;
            //     continue;
            // }

            // let node_start_byte = node.start_byte();
            // if !result
            //     || node_start_byte < byte_offset
            //     || (node_start_byte == byte_offset && (state.pattern_index as u32) < pattern_index)
            // {
            //     let step = &unsafe { &(*self.query).steps }[state.step_index as usize];
            //     if *root_pattern_guaranteed {
            //         *root_pattern_guaranteed = step.root_pattern_guaranteed();
            //     } else if step.root_pattern_guaranteed() {
            //         continue;
            //     }

            //     result = true;
            //     state_index = i as u32;
            //     byte_offset = node_start_byte;
            //     pattern_index = state.pattern_index as u32;
            // }
        }
        result.then_some((state_index, byte_offset, pattern_index))
    }

    fn prepare_to_capture(
        &mut self,
        state_id: usize,
        state_index_to_preserve: u32,
    ) -> Option<usize> {
        let state = &mut self.states[state_id];
        if state.capture_list_id == NONE as u32 {
            state.capture_list_id = self.capture_list_pool.acquire() as u32;

            // If there are no capture lists left in the pool, then terminate whichever
            // state has captured the earliest node in the document, and steal its
            // capture list.
            if state.capture_list_id == NONE as u32 {
                self.did_exceed_match_limit = true;
                // uint32_t state_index, byte_offset, pattern_index;
                if let Some((state_index, byte_offset, pattern_index)) = self
                    .first_in_progress_capture(
                        &mut false, // &state_index,
                                   // &byte_offset,
                                   // &pattern_index,
                                   // NULL
                    )
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
                        other_state.capture_list_id = NONE as u32;
                        other_state.dead = true;
                        let list = &mut self.capture_list_pool.list[capture_list_id as usize];
                        list.clear();
                        let state = &mut self.states[state_id];
                        state.capture_list_id = capture_list_id;
                        return Some(capture_list_id as usize);
                    }
                }
                log::trace!("  ran out of capture lists");
                return None;
            }
        }
        Some(state.capture_list_id as usize)
    }

    fn capture(&mut self, state_id: usize, step_id: usize, node: &Cursor::Node) {
        let state = &mut self.states[state_id];
        if state.dead {
            return;
        };
        let Some(capture_list_id) = self.prepare_to_capture(state_id, u32::MAX) else {
            let state = &mut self.states[state_id];
            state.dead = true;
            return;
        };
        let state = &self.states[state_id];
        let step = &unsafe { &(*self.query).steps }[step_id];
        for j in 0..MAX_STEP_CAPTURE_COUNT {
            let capture_id = step.capture_ids[j];
            if step.capture_ids[j] == NONE {
                break;
            };
            self.capture_list_pool.list[capture_list_id].push(Capture {
                node: node.clone(),
                index: capture_id as u32,
            });
            log::trace!(
                "  capture node. type:{}, pattern:{}, capture_id:{}, capture_count:{}",
                node.str_symbol(),
                state.pattern_index,
                capture_id,
                self.capture_list_pool.list[capture_list_id].len()
            );
        }
    }

    fn add_state(&mut self, pattern: &PatternEntry) {
        let step = &unsafe { &(*self.query).steps }[pattern.step_index as usize];
        let start_depth = self.depth as usize - step.depth as usize;

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
            if (prev_state.start_depth as usize) < start_depth {
                break;
            }
            if prev_state.start_depth as usize == start_depth {
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
        let element = State {
            //     .id = UINT32_MAX,
            //     .capture_list_id = NONE,
            //     .step_index = pattern.step_index,
            //     .pattern_index = pattern.pattern_index,
            //     .start_depth = start_depth,
            //     .consumed_capture_count = 0,
            //     .seeking_immediate_match = true,
            //     .has_in_progress_alternatives = false,
            //     .needs_parent = step.depth == 1,
            //     .dead = false,
            id: u32::MAX,
            capture_list_id: NONE as u32,
            step_index: pattern.step_index,
            pattern_index: pattern.pattern_index,
            start_depth: start_depth as u16,
            consumed_capture_count: 0,
            seeking_immediate_match: true,
            has_in_progress_alternatives: false,
            needs_parent: step.depth == 1,
            dead: false,
        };
        self.states.insert(index, element);
    }
}

struct SortedVec<T, I = usize>(Vec<T>, std::marker::PhantomData<I>);

impl<T, I: TryFrom<usize>> SortedVec<T, I> {
    fn search_sorted_by<C: Ord, F: Fn(&T) -> C>(&self, f: F, needle: C) -> Option<I> {
        self.0
            .binary_search_by_key(&needle, f)
            .ok()?
            .try_into()
            .ok()
    }

    fn search_sorted_with<F: Fn(&T, &T) -> i32>(&self, compare: F, needle: T) -> Option<I> {
        // self.0.binary_search_by(&needle, f).ok()?.try_into().ok()
        todo!()
    }

    fn insert_sorted_by<C, F: Fn(T) -> C>(&self, f: F, x: T) {
        todo!()
    }

    fn back(&self) -> T {
        todo!()
    }

    fn push(&self, x: T) {
        todo!()
    }
}
impl<T> SortedVec<T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T, I> Default for SortedVec<T, I> {
    fn default() -> Self {
        Self(Default::default(), Default::default())
    }
}

// TODO temporary, later use custom made indexes and impl on individual structs
impl<T> std::ops::Index<usize> for SortedVec<T, usize> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
impl<T> std::ops::IndexMut<usize> for SortedVec<T, usize> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[repr(C)]
pub struct TSQuery {
    captures: SymbolTable,
    predicate_values: SymbolTable,
    capture_quantifiers: Array<CaptureQuantifiers>,
    steps: Array<query_step::TSQueryStep>,
    pattern_map: Array<PatternEntry>,
    predicate_steps: Array<tree_sitter::ffi::TSQueryPredicateStep>,
    patterns: Array<QueryPattern>,
    step_offsets: Array<StepOffset>,
    negated_fields: Array<tree_sitter::ffi::TSFieldId>,
    string_buffer: Array<std::ffi::c_char>,
    repeat_symbols_with_rootless_patterns: Array<tree_sitter::ffi::TSSymbol>,
    language: *const tree_sitter::ffi::TSLanguage,
    wildcard_root_pattern_count: u16,
}
#[repr(C)]
struct CaptureQuantifiers;
#[repr(C)]
struct QueryPattern;
#[repr(C)]
struct SymbolTable {
    characters: Array<std::ffi::c_char>,
    slices: Array<Slice>,
}
#[repr(C)]
struct Slice {
    offset: u32,
    length: u32,
}
#[repr(C)]
struct Array<T> {
    contents: *mut T,
    size: u32,
    capacity: u32,
}

impl<T> Array<T> {
    fn len(&self) -> usize {
        self.size as usize
    }

    fn search_sorted_by<C: Ord, F: Fn(&T) -> C>(&self, f: F, needle: C) -> Option<usize> {
        unsafe { std::slice::from_raw_parts(self.contents, self.size as usize) }
            .binary_search_by_key(&needle, f)
            .ok()?
            .try_into()
            .ok()
    }
}

impl<T, I: std::slice::SliceIndex<[T]>> std::ops::Index<I> for Array<T> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        // assert!(index < self.size as usize);
        // unsafe { self.contents.add(index).as_ref().unwrap() }
        let contents = unsafe { std::slice::from_raw_parts(self.contents, self.size as usize) };
        std::ops::Index::index(contents, index)
    }
}
impl<T> std::ops::IndexMut<usize> for Array<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.contents.add(index).as_mut().unwrap() }
    }
}

pub fn print_query(query: &TSQuery) {
    eprint!("query steps:\n");
    let steps = &query.steps;
    for i in 0..steps.size {
        let step = unsafe { steps.contents.add(i as usize).as_ref().unwrap() };
        eprint!("  {}: ", i);
        print_query_step(query, step);
        eprint!(",\n");
    }
}
mod query_step;
use std::collections::VecDeque;

pub(crate) use query_step::print_query_step;
pub(crate) use query_step::TSQueryStep;
use tree_sitter::ffi::TSSymbol;
