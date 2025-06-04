// began from tree-sitter 0.22.5
// TODO integrate that to my fork of tree-sitter,
// it will be cleaner and then I can access private stuff easily.
use crate::search::steped::Status;

use super::{
    Node, QueryCursor, Symbol, TreeCursorStep, NONE, PATTERN_DONE_MARKER, WILDCARD_SYMBOL,
};

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
                            self.depth -= 1;
                            self.on_visible_node = false;
                        }
                        self.ascending = false;
                    }
                    TreeCursorStep::TreeCursorStepNone => {
                        if self.cursor.goto_parent() {
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
                    match self.cursor.goto_first_child_internal() {
                        TreeCursorStep::TreeCursorStepVisible => {
                            self.depth += 1;
                            self.on_visible_node = true;
                            continue;
                        }
                        TreeCursorStep::TreeCursorStepHidden => {
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
                "leave node. depth:{}, type:{}",
                self.depth,
                self.cursor.current_node().str_symbol()
            );
            let steps = unsafe { &(*self.query.q).steps };
            // After leaving a node, remove any states that cannot make further progress.
            self.states = std::mem::take(&mut self.states)
                .into_iter()
                .filter_map(|state| {
                    let step = &steps[state.step_index as usize];

                    // If a state completed its pattern inside of this node, but was deferred from finishing
                    // in order to search for longer matches, mark it as finished.
                    if step.depth == PATTERN_DONE_MARKER
                        && (state.start_depth as u32 > self.depth || self.depth == 0)
                    {
                        log::trace!("  finish pattern {}", state.pattern_index);
                        self.finished_states.push_back(state);
                        did_match = true;
                        None
                    }
                    // If a state needed to match something within this node, then remove that state
                    // as it has failed to match.
                    else if step.depth != PATTERN_DONE_MARKER
                        && state.start_depth as u32 + step.depth as u32 > self.depth
                    {
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
        let query = unsafe { self.query.q.as_ref().unwrap() };
        let mut did_match = false;
        // Get the properties of the current node.
        let node = self.cursor.current_node();
        let parent_node = self.cursor.parent_node();
        let node_intersects_range;
        let parent_intersects_range;
        {
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

            let node_is_error = symbol == Symbol::ERROR;
            let parent_is_error = parent_node.map_or(false, |s| s.symbol() == Symbol::ERROR);

            // Add new states for any patterns whose root node is a wildcard.
            if !node_is_error {
                for i in 0..query.wildcard_root_pattern_count {
                    let pattern = &query.pattern_map[i as usize];

                    // If this node matches the first step of the pattern, then add a new
                    // state at the start of this pattern.
                    let step = &query.steps[pattern.step_index as usize];
                    let start_depth = self.depth - step.depth as u32;
                    let mut should_add = if pattern.is_rooted {
                        node_intersects_range
                    } else {
                        parent_intersects_range && !parent_is_error
                    };
                    should_add &= (!step.field > 0 || status.field_id().into() == step.field)
                        && (Symbol::from(step.supertype_symbol) == Symbol::NONE
                            || status.has_supertypes())
                        && (start_depth <= self.max_start_depth);
                    if should_add {
                        self.add_state(pattern);
                    }
                }
            }

            // Add new states for any patterns whose root node matches this node.
            if let Some(mut i) = query.pattern_map_search(symbol) {
                let mut pattern = &query.pattern_map[i];

                let mut step = &query.steps[pattern.step_index as usize];
                let start_depth = self.depth - step.depth as u32;
                loop {
                    // If this node matches the first step of the pattern, then add a new
                    // state at the start of this pattern.
                    if pattern.is_rooted {
                        if node_intersects_range
                            && (!step.field > 0 || status.field_id().into() == step.field)
                            && (start_depth <= self.max_start_depth)
                        {
                            self.add_state(pattern);
                        }
                    } else if (parent_intersects_range && !parent_is_error)
                        && (!step.field > 0 || status.field_id().into() == step.field)
                        && (start_depth <= self.max_start_depth)
                    {
                        self.add_state(pattern);
                    }

                    // Advance to the next pattern whose root node matches this node.
                    i += 1;
                    if i == query.pattern_map.len() {
                        break;
                    };
                    pattern = &query.pattern_map[i];
                    step = &query.steps[pattern.step_index as usize];
                    if Symbol::from(step.symbol) != symbol {
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
                        query.steps[state!($i).step_index as usize]
                    };
                    (@step) => {
                        query.steps[state!().step_index as usize]
                    };
                }
                state!().has_in_progress_alternatives = false;
                copy_count = 0;

                // Check that the node matches all of the criteria for the next
                // step of the pattern.
                if state!().start_depth as u32 + state!(@step).depth as u32 != self.depth {
                    j += 1 + copy_count;
                    continue;
                }

                // Determine if this node matches this step of the pattern, and also
                // if this node can have later siblings that match this step of the
                // pattern.
                let mut node_does_match;
                if Symbol::from(state!(@step).symbol) == WILDCARD_SYMBOL {
                    node_does_match = !node_is_error && (is_named || !state!(@step).is_named());
                } else {
                    node_does_match = symbol == Symbol::from(state!(@step).symbol);
                }
                let mut later_sibling_can_match = status.has_later_siblings();
                if (state!(@step).is_immediate() && is_named) || state!().seeking_immediate_match {
                    later_sibling_can_match = false;
                }
                if state!(@step).is_last_child() && status.has_later_named_siblings() {
                    node_does_match = false;
                }
                let ss = state!(@step).supertype_symbol;
                if Symbol::from(ss) != Symbol::END {
                    self.cursor.current_status();
                    let has_supertype =
                        status.contains_supertype(state!(@step).supertype_symbol.into());
                    if !has_supertype {
                        node_does_match = false
                    };
                }
                if state!(@step).field > 0 {
                    if state!(@step).field == status.field_id().into() {
                        if !status.can_have_later_siblings_with_this_field() {
                            later_sibling_can_match = false;
                        }
                    } else {
                        node_does_match = false;
                    }
                }

                if state!(@step).negated_field_list_id > 0 {
                    let negated_field_ids =
                        &query.negated_fields[state!(@step).negated_field_list_id as usize..];
                    for negated_field_id in negated_field_ids {
                        // if node.child_by_field_id(*negated_field_id).is_some() { // .id() > 0 // TODO make a more specialized accessor -> better opt and simple to impl
                        if node.has_child_with_field_id((*negated_field_id).into()) {
                            // .id() > 0 // TODO make a more specialized accessor -> better opt and simple to impl
                            node_does_match = false;
                            break;
                        }
                    }
                }

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
                if state!().needs_parent {
                    if let Some(parent) = self.cursor.parent_node() {
                        state!().needs_parent = false;
                        let mut skipped_wildcard = state!(@index);
                        while skipped_wildcard > 0 {
                            skipped_wildcard -= 1;
                            if state!(@step skipped_wildcard).is_dead_end()
                                || state!(@step skipped_wildcard).is_pass_through()
                                || state!(@step skipped_wildcard).depth > 0
                            {
                                continue;
                            }
                            if state!(@step skipped_wildcard).capture_ids[0] != NONE {
                                log::trace!("  capture wildcard parent");
                                self.capture(state!(@index), skipped_wildcard, &parent);
                            }
                            break;
                        }
                    } else {
                        log::trace!("  missing parent node");
                        state!().dead = true;
                    }
                }

                // If the current node is captured in this pattern, add it to the capture list.
                if state!(@step).capture_ids[0] != NONE {
                    self.capture(state!(@index), state!().step_index as usize, &node);
                }

                if state!().dead {
                    self.states.remove(j);
                    j += copy_count;
                    continue;
                }

                // Advance this state to the next step of its pattern.
                state!().step_index += 1;
                state!().seeking_immediate_match = false;
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
                    // dbg!(unsafe { &(*self.query).steps }.len());
                    // dbg!(_j, _k);
                    // dbg!(state!(_k).step_index);
                    // dbg!(state!(@step).alternative_index);
                    // dbg!(state!(@step _k).alternative_index);
                    if state!(@step _k).alternative_index != NONE {
                        // A "dead-end" step exists only to add a non-sequential jump into the step sequence,
                        // via its alternative index. When a state reaches a dead-end step, it jumps straight
                        // to the step's alternative.
                        if state!(@step _k).is_dead_end() {
                            state!(_k).step_index = state!(@step _k).alternative_index;
                            continue;
                        }

                        // A "pass-through" step exists only to add a branch into the step sequence,
                        // via its alternative_index. When a state reaches a pass-through step, it splits
                        // in order to process the alternative step, and then it advances to the next step.
                        let pt = state!(@step _k).is_pass_through();
                        if pt {
                            // dbg!();
                            state!(_k).step_index += 1;
                        }

                        if let Some(_copy) = self.copy_state(&mut _k) {
                            log::trace!(
                                "  split state for branch. pattern:{}, from_step:{}, to_step:{}, immediate:{}, capture_count: {}",
                                state!(_copy).pattern_index,
                                state!(_copy).step_index,
                                state!(@step).alternative_index,
                                state!(@step).alternative_is_immediate(),
                                self.capture_list_pool.get(state!(_copy).capture_list_id).len()
                            );
                            end_index += 1;
                            copy_count += 1;
                            // dbg!(state!(_k).step_index, _k, _copy);
                            // dbg!(state!(@step _k).alternative_index);
                            state!(_copy).step_index = query.steps[_s as usize].alternative_index;
                            if query.steps[_s as usize].alternative_is_immediate() {
                                state!(_copy).seeking_immediate_match = true;
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
                if curr_state!().dead {
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
                        other_state!().has_in_progress_alternatives = true;
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
                        curr_state!().has_in_progress_alternatives = true;
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
                        self.capture_list_pool.get(curr_state!().capture_list_id).len()
                    );
                    let next_step = &query.steps[curr_state!().step_index as usize];
                    if next_step.depth == PATTERN_DONE_MARKER {
                        if curr_state!().has_in_progress_alternatives {
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
            dbg!();
            if !self.advance(false) {
                return None;
            }
        }

        let mut state = self.finished_states.pop_front().unwrap();
        if state.id == u32::MAX {
            state.id = self.next_state_id;
            self.next_state_id += 1;
        };
        let id = state.id;
        let pattern_index = state.pattern_index as usize;
        let captures = self.capture_list_pool.pop(state.capture_list_id);
        return Some(QueryMatch {
            pattern_index,
            captures,
            id,
        });
    }
}

pub struct QueryMatch<Node, P = usize, I = u32> {
    pub pattern_index: P,
    pub captures: Vec<super::Capture<Node, I>>,
    // id of state
    id: u32,
}

#[derive(Debug)]
/// The first item is the capture index
/// The next is capture specific, depending on what item is expected
/// The first bool is if the capture is positive
/// The last item is a bool signifying whether or not it's meant to match
/// any or all captures
pub enum TextPredicateCapture<N = u32, T = Box<str>> {
    EqString(TextPred<N, T>),
    EqCapture(TextPred<N, N>),
    // MatchString(N, regex::bytes::Regex, bool, bool),
    // AnyString(N, Box<[T]>, bool),
}

#[derive(Debug)]
pub struct TextPred<L, R> {
    pub left: L,
    pub right: R,
    pub is_positive: bool,
    pub match_all_nodes: bool,
}

impl<Node: super::Node, P, I: std::cmp::PartialEq> QueryMatch<Node, P, I> {
    pub(crate) fn satisfies_text_predicates<'a, 'b, T: 'a + AsRef<str>>(
        &self,
        text_provider: <Node as super::TextLending<'_>>::TP,
        mut text_predicates: impl Iterator<Item = &'a TextPredicateCapture<I, T>>,
    ) -> bool
    where
        I: 'a + Copy,
    {
        text_predicates.all(|predicate| match predicate {
            TextPredicateCapture::EqCapture(TextPred {
                left,
                right,
                is_positive,
                match_all_nodes,
            }) => {
                // WARN sligntly different sem as we compare nodes structurally and not textually
                // bad for comparing the name of a type ref with the name of a variable ref
                let mut nodes_1 = self.nodes_for_capture_index(*left);
                let mut nodes_2 = self.nodes_for_capture_index(*right);
                while let (Some(node1), Some(node2)) = (nodes_1.next(), nodes_2.next()) {
                    let comp = node1.equal(node2);
                    if comp != *is_positive && *match_all_nodes {
                        return false;
                    }
                    if comp == *is_positive && !*match_all_nodes {
                        return true;
                    }
                }
                nodes_1.next().is_none() && nodes_2.next().is_none()
            }
            TextPredicateCapture::EqString(TextPred {
                left,
                right,
                is_positive,
                match_all_nodes,
            }) => {
                let nodes = self.nodes_for_capture_index(*left);
                let s = right.as_ref().as_bytes();
                for node in nodes {
                    let comp = node.text_equal(text_provider, s.iter().copied());
                    if comp != *is_positive && *match_all_nodes {
                        return false;
                    }
                    if comp == *is_positive && !*match_all_nodes {
                        return true;
                    }
                }
                true
            }
        })
    }

    pub fn nodes_for_capture_index<'a>(&'a self, index: I) -> impl Iterator<Item = &'a Node> {
        self.captures
            .iter()
            .filter(move |x| x.index == index)
            .map(|x| &x.node)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct QueryProperty {
    pub key: Box<str>,
    pub value: Option<Box<str>>,
    pub capture_id: Option<usize>,
}
