//! Experimental! WIP!
//! For now lets use the analysis done by tree_sitter
//! cons of implementing the analysis myself: 
//! - takes time
//! - more compatible with tree_sitter
//! - only need to read the analysis
//! - difficult to access even deeper treesitter apis like the lookhead
//! pros of implementing the analysis myself: 
//! - fragile to read internal of tree_sitter
//! - can add more optimizations
//! - can use rust idioms
//! - avoid unsafety due to ffi
//! 
//! At least for now implementing the analysis myself is put on hold.
//! It might still be straitforward and realtively easy to add optimizations by further processing the provided query graph it is needed.
//! Before impl the analysis, I need some profiling of query performances on the hyperast (also useful for eval.).

use super::*;
const DEBUG_ANALYZE_QUERY: bool = true;
// #define MAX_ANALYSIS_STATE_DEPTH 8
const MAX_ANALYSIS_STATE_DEPTH: u16 = 8;
// #define MAX_ANALYSIS_ITERATION_COUNT 256
const MAX_ANALYSIS_ITERATION_COUNT: usize = 256;

type AnalysisSubgraphArray = SortedVec<AnalysisSubgraph>;


// struct Query {
//     pattern_map: Vec<PatternEntry>,
//     steps: Vec<QueryStep>,
//     negated_fields: Vec<Vec<FieldId>>,
//     wildcard_root_pattern_count: usize,
//     language: Language,
//     step_offsets: SortedVec<StepOffset>,
// }

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

struct Field {
    id: usize,
}


struct QueryAnalysis {
    final_step_indices: Vec<u16>,
    finished_parent_symbols: Vec<Symbol>,
    did_abort: bool,
    states: AnalysisStateSet,
    next_states: AnalysisStateSet,
    deeper_states: AnalysisStateSet,
    state_pool: AnalysisStateSet,
}
impl QueryAnalysis {
    fn new() -> Self {
        Self {
            final_step_indices: todo!(),
            finished_parent_symbols: todo!(),
            did_abort: todo!(),
            states: todo!(),
            next_states: todo!(),
            deeper_states: todo!(),
            state_pool: todo!(),
        }
    }
}

struct AnalysisStateSet(Vec<AnalysisState>);

impl AnalysisStateSet {
    fn len(&self) -> usize {
        todo!()
    }

    fn pop(&mut self) -> AnalysisState {
        todo!()
    }

    fn insert_sorted(&self, state_pool: &AnalysisStateSet, state: &AnalysisState) {
        todo!()
    }

    fn push(&self, state_pool: &AnalysisStateSet, j: &AnalysisState) {
        todo!()
    }

    fn clear(&mut self, state_pool: &AnalysisStateSet) {
        todo!()
    }
}
impl std::ops::Index<usize> for AnalysisStateSet {
    type Output = AnalysisState;

    fn index(&self, index: usize) -> &Self::Output {
        todo!()
    }
}
impl std::ops::IndexMut<usize> for AnalysisStateSet {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        todo!()
    }
}

/*
 * AnalysisState - The state needed for walking the parse table when analyzing
 * a query pattern, to determine at which steps the pattern might fail to match.
 */
struct AnalysisStateEntry {
    parse_state: StateId,
    parent_symbol: Symbol,
    child_index: u16,
    field_id: FieldId,
    done: bool,
}

type StateId = u16;

struct AnalysisState {
    depth: u16,
    step_index: u16,
    root_symbol: Symbol,
    // stack: Vec<()>,
    stack: [AnalysisStateEntry; MAX_ANALYSIS_STATE_DEPTH as usize],
}
impl AnalysisState {
    fn top(&self) -> AnalysisStateEntry {
        todo!()
    }

    fn compare_position(&self, state: AnalysisState) -> isize {
        todo!()
    }

    fn has_supertype(&self, supertype_symbol: Symbol) -> bool {
        todo!()
    }
}

struct AnalysisSubgraphNode {
    state: StateId,
    child_index: u8,
    production_id: u16,
    done: bool,
}

impl AnalysisSubgraphNode {
    fn compare(&self, other: &AnalysisSubgraphNode) -> i32 {
        if self.state < other.state {
            -1
        } else if self.state > other.state {
            1
        } else if self.child_index < other.child_index {
            -1
        } else if self.child_index > other.child_index {
            1
        } else if self.done < other.done {
            -1
        } else if self.done > other.done {
            1
        } else if self.production_id < other.production_id {
            -1
        } else if self.production_id > other.production_id {
            1
        } else {
            0
        }
    }
}

struct AnalysisSubgraph {
    symbol: Symbol,
    start_states: Vec<StateId>,
    nodes: SortedVec<AnalysisSubgraphNode>,
}

impl AnalysisSubgraph {
    fn new(symbol: Symbol) -> Self {
        Self {
            symbol,
            start_states: Default::default(),
            nodes: Default::default(),
        }
    }
}

mod aaa {
    use tree_sitter::ffi::TSLanguage;
    #[repr(C)]
    #[derive(Clone, Copy, PartialEq)]
    pub(super) enum TSParseActionType {
        TSParseActionTypeShift,
        TSParseActionTypeReduce,
        TSParseActionTypeAccept,
        TSParseActionTypeRecover,
    }

    impl TSParseActionType {
        pub fn is_shift(&self) -> bool {
            self == &TSParseActionType::TSParseActionTypeShift
        }
        pub fn is_reduce(&self) -> bool {
            self == &TSParseActionType::TSParseActionTypeReduce
        }
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct Shift {
        r#type: TSParseActionType,
        pub state: tree_sitter::ffi::TSStateId,
        pub extra: bool,
        repetition: bool,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Reduce {
        r#type: TSParseActionType,
        pub child_count: u8,
        pub symbol: tree_sitter::ffi::TSSymbol,
        dynamic_precedence: i16,
        pub production_id: u16,
    }
    #[repr(C)]
    pub(super) union TSParseAction {
        pub shift: Shift,
        pub reduce: Reduce,
        pub r#type: TSParseActionType,
    }

    #[repr(C)]
    pub(super) struct LookaheadIterator {
        pub(super) language: *const TSLanguage,
        pub(super) data: *const u16,
        pub(super) group_end: *const u16,
        pub(super) state: tree_sitter::ffi::TSStateId,
        pub(super) table_value: u16,
        pub(super) section_index: u16,
        pub(super) group_count: u16,
        pub(super) is_small_state: bool,

        pub(super) actions: *const TSParseAction,
        pub(super) symbol: tree_sitter::ffi::TSSymbol,
        pub(super) next_state: tree_sitter::ffi::TSStateId,
        pub(super) action_count: u16,
    }
}

impl Query {
    // Walk the subgraph for this non-terminal, tracking all of the possible
    // sequences of progress within the pattern.
    fn perform_analysis(
        &mut self,
        subgraphs: &AnalysisSubgraphArray,
        analysis: &mut QueryAnalysis,
    ) {
        let mut recursion_depth_limit = 0;
        let mut prev_final_step_count = 0;
        analysis.final_step_indices.clear();
        analysis.finished_parent_symbols.clear();

        for iteration in 0.. {
            if iteration == MAX_ANALYSIS_ITERATION_COUNT {
                analysis.did_abort = true;
                break;
            }

            if DEBUG_ANALYZE_QUERY {
                eprint!("Iteration: {}. Final step indices:", iteration);
                for s in &analysis.final_step_indices {
                    eprint!(" {:4}", s);
                }
                eprint!("\n");
                for j in 0..analysis.states.len() {
                    let state = &analysis.states[j];
                    eprint!("  {:3}: step: {}, stack: [", j, state.step_index);
                    for k in 0..state.depth {
                        let k = k as usize;
                        eprint!(
                            " {{{}, child: {}, state: {:4}",
                            self.language.symbol_name(state.stack[k].parent_symbol),
                            state.stack[k].child_index,
                            state.stack[k].parse_state
                        );
                        if state.stack[k].field_id > 0 {
                            eprint!(
                                ", field: {}",
                                self.language.field_name(state.stack[k].field_id)
                            )
                        }
                        if state.stack[k].done {
                            eprint!(", DONE")
                        }
                        eprint!("}}");
                    }
                    eprint!(" ]\n");
                }
            }
            // If no further progress can be made within the current recursion depth limit, then
            // bump the depth limit by one, and continue to process the states the exceeded the
            // limit. But only allow this if progress has been made since the last time the depth
            // limit was increased.
            if analysis.states.len() == 0 {
                if analysis.deeper_states.len() > 0
                    && analysis.final_step_indices.len() > prev_final_step_count
                {
                    if DEBUG_ANALYZE_QUERY {
                        eprint!(
                            "Increase recursion depth limit to {}\n",
                            recursion_depth_limit + 1,
                        );
                    }

                    prev_final_step_count = analysis.final_step_indices.len();
                    recursion_depth_limit += 1;
                    std::mem::swap(&mut analysis.states, &mut analysis.deeper_states);
                    continue;
                }

                break;
            }

            analysis.next_states.clear(&analysis.state_pool);
            let mut j = 0;
            while j < analysis.states.len() {
                let state = &mut analysis.states[j];

                // For efficiency, it's important to avoid processing the same analysis state more
                // than once. To achieve this, keep the states in order of ascending position within
                // their hypothetical syntax trees. In each iteration of this loop, start by advancing
                // the states that have made the least progress. Avoid advancing states that have already
                // made more progress.
                if analysis.next_states.len() > 0 {
                    let comparison = state.compare_position(analysis.next_states.pop());
                    if comparison == 0 {
                        analysis
                            .next_states
                            .insert_sorted(&analysis.state_pool, state);
                        continue;
                    } else if comparison > 0 {
                        if DEBUG_ANALYZE_QUERY {
                            eprint!("Terminate iteration at state {}\n", j);
                        }
                        while j < analysis.states.len() {
                            analysis
                                .next_states
                                .push(&analysis.state_pool, &analysis.states[j]);
                            j += 1;
                        }
                        break;
                    }
                }

                // const TSStateId
                let parse_state = state.top().parse_state;
                // const TSSymbol
                let parent_symbol = state.top().parent_symbol;
                // const TSFieldId
                let parent_field_id = state.top().field_id;
                // const unsigned
                let child_index = state.top().child_index;
                // const QueryStep
                let step = &self.steps[state.step_index as usize];

                let Some(subgraph_index) =
                    subgraphs.search_sorted_by(|x| x.symbol, parent_symbol.to_usize())
                else {
                    continue;
                };
                let subgraph = &subgraphs[subgraph_index];

                // Follow every possible path in the parse table, but only visit states that
                // are part of the subgraph for the current symbol.
                let mut _lookahead_iterator = self.language.lookaheads(parse_state);
                while let Some(sym) = _lookahead_iterator.next() {
                    let sym = Symbol::from(sym);
                    let __lookahead_iterator = _lookahead_iterator.into_raw();
                    let lookahead_iterator: *mut aaa::LookaheadIterator =
                        unsafe { std::mem::transmute(__lookahead_iterator) };
                    let action_count = unsafe { (*lookahead_iterator).action_count };
                    let next_state = unsafe { (*lookahead_iterator).next_state };

                    let mut successor = AnalysisSubgraphNode {
                        state: parse_state,
                        child_index: child_index as u8,
                        production_id: Default::default(),
                        done: Default::default(),
                    };
                    if action_count > 0 {
                        let action =
                            unsafe { (*lookahead_iterator).actions.add(action_count as usize - 1) };
                        if (unsafe { (*action).r#type })
                            == aaa::TSParseActionType::TSParseActionTypeShift
                        {
                            if !unsafe { (*action).shift.extra } {
                                successor.state = unsafe { (*action).shift.state };
                                successor.child_index += 1;
                            }
                        } else {
                            _lookahead_iterator = unsafe {
                                tree_sitter::LookaheadIterator::from_raw(__lookahead_iterator)
                            };
                            continue;
                        }
                    } else if next_state != 0 {
                        successor.state = next_state;
                        successor.child_index += 1;
                    } else {
                        _lookahead_iterator = unsafe {
                            tree_sitter::LookaheadIterator::from_raw(__lookahead_iterator)
                        };
                        continue;
                    }
                    _lookahead_iterator =
                        unsafe { tree_sitter::LookaheadIterator::from_raw(__lookahead_iterator) };

                    // unsigned node_index;
                    // array_search_sorted_with(
                    //   &subgraph->nodes,
                    //   analysis_subgraph_node__compare, &successor,
                    //   &node_index, &exists
                    // );
                    let mut node_index = subgraph
                        .nodes
                        .search_sorted_with(AnalysisSubgraphNode::compare, &mut successor)
                        .unwrap();
                    while node_index < subgraph.nodes.len() {
                        let node = &subgraph.nodes[node_index]; // TODO compare with sem. of node_index++
                        node_index += 1;
                        if node.state != successor.state
                            || node.child_index != successor.child_index
                        {
                            break;
                        };

                        // Use the subgraph to determine what alias and field will eventually be applied
                        // to this child node.
                        let alias = self.language.alias_at(node.production_id, child_index);
                        let visible_symbol = alias
                            // self.language.symbol_metadata[sym].visible
                            //       ? self.language.public_symbol_map[sym]
                            //       : 0;
                            .unwrap_or_else(|| {
                                if self.language.symbol_metadata(sym).visible {
                                    self.language.public_symbol_map(sym)
                                } else {
                                    Symbol::NONE // TODO check if is 0
                                }
                            });
                        let field_id = parent_field_id;
                        if field_id != 0 {
                            //     const TSFieldMapEntry *field_map, *field_map_end;
                            //     ts_language_field_map(self->language, node->production_id, &field_map, &field_map_end);
                            //     for (; field_map != field_map_end; field_map++) {
                            //       if (!field_map->inherited && field_map->child_index == child_index) {
                            //         field_id = field_map->field_id;
                            //         break;
                            //       }
                            //     }
                        }

                        // Create a new state that has advanced past this hypothetical subtree.
                        let next_state = &mut *state;
                        let mut next_state_top = &mut next_state.top();
                        next_state_top.child_index = successor.child_index as u16;
                        next_state_top.parse_state = successor.state;
                        if node.done {
                            next_state_top.done = true
                        }

                        // Determine if this hypothetical child node would match the current step
                        // of the query pattern.
                        let mut does_match = false;
                        if visible_symbol != Symbol::NONE {
                            does_match = true;
                            if step.symbol == WILDCARD_SYMBOL {
                                if step.is_named
                                    && !self.language.symbol_metadata(visible_symbol).named
                                {
                                    does_match = false
                                };
                            } else if step.symbol != visible_symbol {
                                does_match = false;
                            }
                            if step.field > 0 && step.field != field_id {
                                does_match = false;
                            }
                            if step.supertype_symbol != Symbol::NONE
                                && !state.has_supertype(step.supertype_symbol)
                            {
                                does_match = false
                            };
                        }
                        // If this child is hidden, then descend into it and walk through its children.
                        // If the top entry of the stack is at the end of its rule, then that entry can
                        // be replaced. Otherwise, push a new entry onto the stack.
                        else if self.language.is_hidden(sym) {
                            if !next_state_top.done {
                                if next_state.depth + 1 >= MAX_ANALYSIS_STATE_DEPTH {
                                    //         #ifdef DEBUG_ANALYZE_QUERY
                                    //           printf("Exceeded depth limit for state %u\n", j);
                                    //         #endif

                                    analysis.did_abort = true;
                                    continue;
                                }

                                next_state.depth += 1;
                                next_state_top = &mut next_state.top();
                            }

                            //     *next_state_top = (AnalysisStateEntry) {
                            //       .parse_state = parse_state,
                            //       .parent_symbol = sym,
                            //       .child_index = 0,
                            //       .field_id = field_id,
                            //       .done = false,
                            //     };

                            //     if (analysis_state__recursion_depth(&next_state) > recursion_depth_limit) {
                            //       analysis_state_set__insert_sorted(
                            //         &analysis->deeper_states,
                            //         &analysis->state_pool,
                            //         &next_state
                            //       );
                            //       continue;
                            //     }
                        }

                        // Pop from the stack when this state reached the end of its current syntax node.
                        while (next_state.depth > 0 && next_state_top.done) {
                            next_state.depth -= 1;
                            next_state_top = &mut next_state.top();
                        }

                        // If this hypothetical child did match the current step of the query pattern,
                        // then advance to the next step at the current depth. This involves skipping
                        // over any descendant steps of the current child.
                        let mut next_step = step;
                        if does_match {
                            loop {
                                next_state.step_index += 1;
                                next_step = &self.steps[next_state.step_index as usize];
                                if next_step.depth == PATTERN_DONE_MARKER
                                    || next_step.depth <= step.depth
                                {
                                    break;
                                };
                            }
                        } else if successor.state == parse_state {
                            continue;
                        }

                        loop {
                            // Skip pass-through states. Although these states have alternatives, they are only
                            // used to implement repetitions, and query analysis does not need to process
                            // repetitions in order to determine whether steps are possible and definite.
                            if next_step.is_pass_through {
                                next_state.step_index += 1;
                                // next_step += 1; // TODO
                                continue;
                            }

                            // If the pattern is finished or hypothetical parent node is complete, then
                            // record that matching can terminate at this step of the pattern. Otherwise,
                            // add this state to the list of states to process on the next iteration.
                            if !next_step.is_dead_end {
                                let did_finish_pattern =
                                    self.steps[next_state.step_index as usize].depth != step.depth;
                                if did_finish_pattern {
                                    //         array_insert_sorted_by(&analysis.finished_parent_symbols, , state.root_symbol);
                                } else if next_state.depth == 0 {
                                    //         array_insert_sorted_by(&analysis.final_step_indices, , next_state.step_index);
                                } else {
                                    //         analysis_state_set__insert_sorted(&analysis.next_states, &analysis.state_pool, &next_state);
                                }
                            }

                            // If the state has advanced to a step with an alternative step, then add another state
                            // at that alternative step. This process is simpler than the process of actually matching a
                            // pattern during query execution, because for the purposes of query analysis, there is no
                            // need to process repetitions.
                            if does_match
                                && next_step.alternative_index != NONE
                                && next_step.alternative_index > next_state.step_index
                            {
                                next_state.step_index = next_step.alternative_index;
                                next_step = &self.steps[next_state.step_index as usize];
                            } else {
                                break;
                            }
                        }
                    }
                }
            }

            std::mem::swap(&mut analysis.states, &mut analysis.next_states);
        }
    }

    fn analyze_patterns(&mut self, error_offset: &mut usize) -> bool {
        let non_rooted_pattern_start_steps: Vec<u16> = vec![];
        // for (unsigned i = 0; i < self->pattern_map.size; i++) {
        //   PatternEntry *pattern = &self->pattern_map.contents[i];
        //   if (!pattern->is_rooted) {
        //     QueryStep *step = &self->steps.contents[pattern->step_index];
        //     if (step->symbol != WILDCARD_SYMBOL) {
        //       array_push(&non_rooted_pattern_start_steps, i);
        //     }
        //   }
        // }

        // Walk forward through all of the steps in the query, computing some
        // basic information about each step. Mark all of the steps that contain
        // captures, and record the indices of all of the steps that have child steps.
        let mut parent_step_indices: Vec<u32> = vec![];
        for i in 0..self.steps.len() {
            let step = &mut self.steps[i];
            if step.depth == PATTERN_DONE_MARKER {
                step.parent_pattern_guaranteed = true;
                step.root_pattern_guaranteed = true;
                continue;
            }

            let mut has_children = false;
            let is_wildcard = step.symbol == WILDCARD_SYMBOL;
            step.contains_captures = step.capture_ids[0] != NONE;
            for j in (i + 1)..self.steps.len() {
                let next_step = &mut self.steps[j];
                if next_step.depth == PATTERN_DONE_MARKER || next_step.depth <= step.depth {
                    break;
                }
                if next_step.capture_ids[0] != NONE {
                    step.contains_captures = true;
                }
                if !is_wildcard {
                    next_step.root_pattern_guaranteed = true;
                    next_step.parent_pattern_guaranteed = true;
                }
                has_children = true;
            }

            if has_children && !is_wildcard {
                parent_step_indices.push(i.try_into().unwrap());
            }
        }

        // For every parent symbol in the query, initialize an 'analysis subgraph'.
        // This subgraph lists all of the states in the parse table that are directly
        // involved in building subtrees for this symbol.
        //
        // In addition to the parent symbols in the query, construct subgraphs for all
        // of the hidden symbols in the grammar, because these might occur within
        // one of the parent nodes, such that their children appear to belong to the
        // parent.
        let mut subgraphs = AnalysisSubgraphArray::default();
        for i in 0..parent_step_indices.len() {
            let parent_step_index = parent_step_indices[i];
            let parent_symbol = self.steps[parent_step_index as usize].symbol;
            let subgraph = AnalysisSubgraph::new(parent_symbol);
            subgraphs.insert_sorted_by(|s| s.symbol, subgraph);
        }
        for sym in self.language.hidden_symbols() {
            // (TSSymbol sym = (uint16_t)self->language->token_count; sym < (uint16_t)self->language->symbol_count; sym++)
            if !self.language.symbol_metadata(sym).visible {
                let subgraph = AnalysisSubgraph::new(sym);
                subgraphs.insert_sorted_by(|s| s.symbol, subgraph);
            }
        }

        // Scan the parse table to find the data needed to populate these subgraphs.
        // Collect three things during this scan:
        //   1) All of the parse states where one of these symbols can start.
        //   2) All of the parse states where one of these symbols can end, along
        //      with information about the node that would be created.
        //   3) A list of predecessor states for each state.
        let mut predecessor_map = self.language.state_predecessor_map_new();
        // StatePredecessorMap predecessor_map = state_predecessor_map_new(self->language);
        // for (TSStateId state = 1; state < (uint16_t)self->language->state_count; state++)
        for state in self.language.states() {
            //   unsigned subgraph_index, exists;
            let mut _lookahead_iterator = self.language.lookaheads(state);
            while let Some(sym) = _lookahead_iterator.next() {
                let sym = Symbol::from(sym);
                let __lookahead_iterator = _lookahead_iterator.into_raw();
                let lookahead_iterator: *mut aaa::LookaheadIterator =
                    unsafe { std::mem::transmute(__lookahead_iterator) };
                let action_count = unsafe { (*lookahead_iterator).action_count };
                let next_state = unsafe { (*lookahead_iterator).next_state };
                if action_count > 0 {
                    for i in 0..action_count {
                        let action = unsafe { (*lookahead_iterator).actions.add(i as usize) };
                        let action = unsafe { action.as_ref().unwrap() };
                        if action.r#type.is_reduce() {
                            let aliases = self.language.alias_for_symbol(action.reduce.symbol);
                            for symbol in aliases {
                                if let Some(subgraph_index) =
                                    subgraphs.search_sorted_by(|s| s.symbol, symbol.to_usize())
                                {
                                    let subgraph = &mut subgraphs[subgraph_index];
                                    if subgraph.nodes.len() == 0
                                        || subgraph.nodes.back().state != state
                                    {
                                        subgraph.nodes.push(AnalysisSubgraphNode {
                                            state,
                                            child_index: action.reduce.production_id as u8,
                                            production_id: action.reduce.child_count as u16,
                                            done: true,
                                        })
                                    }
                                }
                            }
                        } else if action.r#type.is_shift() && !action.shift.extra {
                            let next_state = action.shift.state;
                            //   state_predecessor_map_add(&predecessor_map, next_state, state);
                        }
                    }
                } else if next_state != 0 {
                    if next_state != state {
                        // state_predecessor_map_add(&predecessor_map, lookahead_iterator.next_state, state);
                    }
                    if self.language.state_is_primary(state) {
                        //         const TSSymbol *aliases, *aliases_end;
                        //         ts_language_aliases_for_symbol(
                        //           self->language,
                        //           lookahead_iterator.symbol,
                        //           &aliases,
                        //           &aliases_end
                        //         );
                        //         for (const TSSymbol *symbol = aliases; symbol < aliases_end; symbol++) {
                        //           array_search_sorted_by(
                        //             &subgraphs,
                        //             .symbol,
                        //             *symbol,
                        //             &subgraph_index,
                        //             &exists
                        //           );
                        //           if (exists) {
                        //             AnalysisSubgraph *subgraph = &subgraphs.contents[subgraph_index];
                        //             if (
                        //               subgraph->start_states.size == 0 ||
                        //               *array_back(&subgraph->start_states) != state
                        //             )
                        //             array_push(&subgraph->start_states, state);
                        //           }
                        //         }
                    }
                }
                _lookahead_iterator =
                    unsafe { tree_sitter::LookaheadIterator::from_raw(__lookahead_iterator) };
            }
        }

        // For each subgraph, compute the preceding states by walking backward
        // from the end states using the predecessor map.
        // Array(AnalysisSubgraphNode) next_nodes = array_new();
        // for (unsigned i = 0; i < subgraphs.size; i++)
        {
            //   AnalysisSubgraph *subgraph = &subgraphs.contents[i];
            //   if (subgraph->nodes.size == 0) {
            //     array_delete(&subgraph->start_states);
            //     array_erase(&subgraphs, i);
            //     i--;
            //     continue;
            //   }
            //   array_assign(&next_nodes, &subgraph->nodes);
            //   while (next_nodes.size > 0) {
            //     AnalysisSubgraphNode node = array_pop(&next_nodes);
            //     if (node.child_index > 1) {
            //       unsigned predecessor_count;
            //       const TSStateId *predecessors = state_predecessor_map_get(
            //         &predecessor_map,
            //         node.state,
            //         &predecessor_count
            //       );
            //       for (unsigned j = 0; j < predecessor_count; j++) {
            //         AnalysisSubgraphNode predecessor_node = {
            //           .state = predecessors[j],
            //           .child_index = node.child_index - 1,
            //           .production_id = node.production_id,
            //           .done = false,
            //         };
            //         unsigned index, exists;
            //         array_search_sorted_with(
            //           &subgraph->nodes, analysis_subgraph_node__compare, &predecessor_node,
            //           &index, &exists
            //         );
            //         if (!exists) {
            //           array_insert(&subgraph->nodes, index, predecessor_node);
            //           array_push(&next_nodes, predecessor_node);
            //         }
            //       }
            //     }
            //   }
        }

        if DEBUG_ANALYZE_QUERY {
            //   printf("\nSubgraphs:\n");
            //   for (unsigned i = 0; i < subgraphs.size; i++) {
            //     AnalysisSubgraph *subgraph = &subgraphs.contents[i];
            //     printf("  %u, %s:\n", subgraph->symbol, ts_language_symbol_name(self->language, subgraph->symbol));
            //     for (unsigned j = 0; j < subgraph->start_states.size; j++) {
            //       printf(
            //         "    {state: %u}\n",
            //         subgraph->start_states.contents[j]
            //       );
            //     }
            //     for (unsigned j = 0; j < subgraph->nodes.size; j++) {
            //       AnalysisSubgraphNode *node = &subgraph->nodes.contents[j];
            //       printf(
            //         "    {state: %u, child_index: %u, production_id: %u, done: %d}\n",
            //         node->state, node->child_index, node->production_id, node->done
            //       );
            //     }
            //     printf("\n");
            //   }
        }

        // For each non-terminal pattern, determine if the pattern can successfully match,
        // and identify all of the possible children within the pattern where matching could fail.
        let mut all_patterns_are_valid = true;
        let mut analysis = QueryAnalysis::new();
        for i in 0..parent_step_indices.len() {
            let parent_step_index = parent_step_indices[i];
            let parent_depth = self.steps[parent_step_index as usize].depth;
            let parent_symbol = self.steps[parent_step_index as usize].symbol;
            if parent_symbol == Symbol::ERROR {
                continue;
            }

            // Find the subgraph that corresponds to this pattern's root symbol. If the pattern's
            // root symbol is a terminal, then return an error.
            if let Some(subgraph_index) =
                subgraphs.search_sorted_by(|s| s.symbol, parent_symbol.to_usize())
            {
                let first_child_step_index = parent_step_index + 1;
                //     uint32_t j, child_exists;
                //     array_search_sorted_by(&self->step_offsets, .step_index, first_child_step_index, &j, &child_exists);
                //     assert(child_exists);
                let j = self
                    .step_offsets
                    .search_sorted_by(|s| s.step_index, first_child_step_index as usize)
                    .unwrap();
                //     *error_offset = self->step_offsets.contents[j].byte_offset;
                all_patterns_are_valid = false;
                break;
            }

            // Initialize an analysis state at every parse state in the table where
            // this parent symbol can occur.
            //   AnalysisSubgraph *subgraph = &subgraphs.contents[subgraph_index];
            //   analysis_state_set__clear(&analysis.states, &analysis.state_pool);
            //   analysis_state_set__clear(&analysis.deeper_states, &analysis.state_pool);
            //   for (unsigned j = 0; j < subgraph->start_states.size; j++)
            {
                //     TSStateId parse_state = subgraph->start_states.contents[j];
                //     analysis_state_set__push(&analysis.states, &analysis.state_pool, &((AnalysisState) {
                //       .step_index = parent_step_index + 1,
                //       .stack = {
                //         [0] = {
                //           .parse_state = parse_state,
                //           .parent_symbol = parent_symbol,
                //           .child_index = 0,
                //           .field_id = 0,
                //           .done = false,
                //         },
                //       },
                //       .depth = 1,
                //       .root_symbol = parent_symbol,
                //     }));
            }

            if DEBUG_ANALYZE_QUERY {
                //     printf(
                //       "\nWalk states for %s:\n",
                //       ts_language_symbol_name(self->language, analysis.states.contents[0]->stack[0].parent_symbol)
                //     );
            }

            //   analysis.did_abort = false;
            //   ts_query__perform_analysis(self, &subgraphs, &analysis);

            // If this pattern could not be fully analyzed, then every step should
            // be considered fallible.
            if analysis.did_abort {
                //     for (unsigned j = parent_step_index + 1; j < self->steps.size; j++)
                {
                    //       QueryStep *step = &self->steps.contents[j];
                    //       if (
                    //         step->depth <= parent_depth ||
                    //         step->depth == PATTERN_DONE_MARKER
                    //       ) break;
                    //       if (!step->is_dead_end) {
                    //         step->parent_pattern_guaranteed = false;
                    //         step->root_pattern_guaranteed = false;
                    //       }
                }
                continue;
            }

            //   // If this pattern cannot match, store the pattern index so that it can be
            //   // returned to the caller.
            //   if (analysis.finished_parent_symbols.size == 0) {
            //     assert(analysis.final_step_indices.size > 0);
            //     uint16_t impossible_step_index = *array_back(&analysis.final_step_indices);
            //     uint32_t j, impossible_exists;
            //     array_search_sorted_by(&self->step_offsets, .step_index, impossible_step_index, &j, &impossible_exists);
            //     if (j >= self->step_offsets.size) j = self->step_offsets.size - 1;
            //     *error_offset = self->step_offsets.contents[j].byte_offset;
            //     all_patterns_are_valid = false;
            //     break;
            //   }

            //   // Mark as fallible any step where a match terminated.
            //   // Later, this property will be propagated to all of the step's predecessors.
            //   for (unsigned j = 0; j < analysis.final_step_indices.size; j++) {
            //     uint32_t final_step_index = analysis.final_step_indices.contents[j];
            //     QueryStep *step = &self->steps.contents[final_step_index];
            //     if (
            //       step->depth != PATTERN_DONE_MARKER &&
            //       step->depth > parent_depth &&
            //       !step->is_dead_end
            //     ) {
            //       step->parent_pattern_guaranteed = false;
            //       step->root_pattern_guaranteed = false;
            //     }
            //   }
            // }

            // Mark as indefinite any step with captures that are used in predicates.
            // Array(uint16_t) predicate_capture_ids = array_new();
            // for (unsigned i = 0; i < self->patterns.size; i++) {
            //   QueryPattern *pattern = &self->patterns.contents[i];

            //   // Gather all of the captures that are used in predicates for this pattern.
            //   array_clear(&predicate_capture_ids);
            //   for (
            //     unsigned start = pattern->predicate_steps.offset,
            //     end = start + pattern->predicate_steps.length,
            //     j = start; j < end; j++
            //   ) {
            //     TSQueryPredicateStep *step = &self->predicate_steps.contents[j];
            //     if (step->type == TSQueryPredicateStepTypeCapture) {
            //       uint16_t value_id = step->value_id;
            //       array_insert_sorted_by(&predicate_capture_ids, , value_id);
            //     }
            //   }

            //   // Find all of the steps that have these captures.
            //   for (
            //     unsigned start = pattern->steps.offset,
            //     end = start + pattern->steps.length,
            //     j = start; j < end; j++
            //   ) {
            //     QueryStep *step = &self->steps.contents[j];
            //     for (unsigned k = 0; k < MAX_STEP_CAPTURE_COUNT; k++) {
            //       uint16_t capture_id = step->capture_ids[k];
            //       if (capture_id == NONE) break;
            //       unsigned index, exists;
            //       array_search_sorted_by(&predicate_capture_ids, , capture_id, &index, &exists);
            //       if (exists) {
            //         step->root_pattern_guaranteed = false;
            //         break;
            //       }
            //     }
            //   }
        }

        // Propagate fallibility. If a pattern is fallible at a given step, then it is
        // fallible at all of its preceding steps.
        // bool done = self->steps.size == 0;
        // while (!done) {
        //   done = true;
        //   for (unsigned i = self->steps.size - 1; i > 0; i--) {
        //     QueryStep *step = &self->steps.contents[i];
        //     if (step->depth == PATTERN_DONE_MARKER) continue;

        //     // Determine if this step is definite or has definite alternatives.
        //     bool parent_pattern_guaranteed = false;
        //     for (;;) {
        //       if (step->root_pattern_guaranteed) {
        //         parent_pattern_guaranteed = true;
        //         break;
        //       }
        //       if (step->alternative_index == NONE || step->alternative_index < i) {
        //         break;
        //       }
        //       step = &self->steps.contents[step->alternative_index];
        //     }

        //     // If not, mark its predecessor as indefinite.
        //     if (!parent_pattern_guaranteed) {
        //       QueryStep *prev_step = &self->steps.contents[i - 1];
        //       if (
        //         !prev_step->is_dead_end &&
        //         prev_step->depth != PATTERN_DONE_MARKER &&
        //         prev_step->root_pattern_guaranteed
        //       ) {
        //         prev_step->root_pattern_guaranteed = false;
        //         done = false;
        //       }
        //     }
        //   }
        // }

        if DEBUG_ANALYZE_QUERY {
            //   printf("Steps:\n");
            //   for (unsigned i = 0; i < self->steps.size; i++) {
            //     QueryStep *step = &self->steps.contents[i];
            //     if (step->depth == PATTERN_DONE_MARKER) {
            //       printf("  %u: DONE\n", i);
            //     } else {
            //       printf(
            //         "  %u: {symbol: %s, field: %s, depth: %u, parent_pattern_guaranteed: %d, root_pattern_guaranteed: %d}\n",
            //         i,
            //         (step->symbol == WILDCARD_SYMBOL)
            //           ? "ANY"
            //           : ts_language_symbol_name(self->language, step->symbol),
            //         (step->field ? ts_language_field_name_for_id(self->language, step->field) : "-"),
            //         step->depth,
            //         step->parent_pattern_guaranteed,
            //         step->root_pattern_guaranteed
            //       );
            //     }
            //   }
        }

        // Determine which repetition symbols in this language have the possibility
        // of matching non-rooted patterns in this query. These repetition symbols
        // prevent certain optimizations with range restrictions.
        analysis.did_abort = false;
        // for (uint32_t i = 0; i < non_rooted_pattern_start_steps.size; i++) {
        //   uint16_t pattern_entry_index = non_rooted_pattern_start_steps.contents[i];
        //   PatternEntry *pattern_entry = &self->pattern_map.contents[pattern_entry_index];

        //   analysis_state_set__clear(&analysis.states, &analysis.state_pool);
        //   analysis_state_set__clear(&analysis.deeper_states, &analysis.state_pool);
        //   for (unsigned j = 0; j < subgraphs.size; j++) {
        //     AnalysisSubgraph *subgraph = &subgraphs.contents[j];
        //     TSSymbolMetadata metadata = ts_language_symbol_metadata(self->language, subgraph->symbol);
        //     if (metadata.visible || metadata.named) continue;

        //     for (uint32_t k = 0; k < subgraph->start_states.size; k++) {
        //       TSStateId parse_state = subgraph->start_states.contents[k];
        //       analysis_state_set__push(&analysis.states, &analysis.state_pool, &((AnalysisState) {
        //         .step_index = pattern_entry->step_index,
        //         .stack = {
        //           [0] = {
        //             .parse_state = parse_state,
        //             .parent_symbol = subgraph->symbol,
        //             .child_index = 0,
        //             .field_id = 0,
        //             .done = false,
        //           },
        //         },
        //         .root_symbol = subgraph->symbol,
        //         .depth = 1,
        //       }));
        //     }
        //   }

        if DEBUG_ANALYZE_QUERY {
            //     printf("\nWalk states for rootless pattern step %u:\n", pattern_entry->step_index);
        }

        //   ts_query__perform_analysis(
        //     self,
        //     &subgraphs,
        //     &analysis
        //   );

        //   if (analysis.finished_parent_symbols.size > 0) {
        //     self->patterns.contents[pattern_entry->pattern_index].is_non_local = true;
        //   }

        //   for (unsigned k = 0; k < analysis.finished_parent_symbols.size; k++) {
        //     TSSymbol symbol = analysis.finished_parent_symbols.contents[k];
        //     array_insert_sorted_by(&self->repeat_symbols_with_rootless_patterns, , symbol);
        //   }
        // }

        if DEBUG_ANALYZE_QUERY {
            //   if (self->repeat_symbols_with_rootless_patterns.size > 0) {
            //     printf("\nRepetition symbols with rootless patterns:\n");
            //     printf("aborted analysis: %d\n", analysis.did_abort);
            //     for (unsigned i = 0; i < self->repeat_symbols_with_rootless_patterns.size; i++) {
            //       TSSymbol symbol = self->repeat_symbols_with_rootless_patterns.contents[i];
            //       printf("  %u, %s\n", symbol, ts_language_symbol_name(self->language, symbol));
            //     }
            //     printf("\n");
            //   }
        }

        // Cleanup
        // for (unsigned i = 0; i < subgraphs.size; i++) {
        //   array_delete(&subgraphs.contents[i].start_states);
        //   array_delete(&subgraphs.contents[i].nodes);
        // }
        // array_delete(&subgraphs);
        // query_analysis__delete(&analysis);
        // array_delete(&next_nodes);
        // array_delete(&non_rooted_pattern_start_steps);
        // array_delete(&parent_step_indices);
        // array_delete(&predicate_capture_ids);
        // state_predecessor_map_delete(&predecessor_map);

        return all_patterns_are_valid;
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
