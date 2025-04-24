//! This query matcher started as a translation from tree_sitter (query.c).
//! TODOs
//! - handle range constraint
//! - use custom indexes
//! - rework status' consumption after a bit of profiling
//! - handle matching captures in order

// static const TSQueryError PARENT_DONE = -1;

const PATTERN_DONE_MARKER: u16 = u16::MAX;
const NONE: u16 = u16::MAX;
const WILDCARD_SYMBOL: Symbol = Symbol::WILDCARD_SYMBOL;

// #define MAX_STEP_CAPTURE_COUNT 3
const MAX_STEP_CAPTURE_COUNT: usize = 3;
// #define MAX_NEGATED_FIELD_COUNT 8
// #define MAX_STATE_PREDECESSOR_COUNT 256

// TODO use indexes on typed collections, it will for me to remove some casts and will help to normalize/generify indexes.
// it will also make it easier to maintain and change stuff later.
struct StateId(usize);
struct StepId(usize);
struct CaptureListId(usize);
struct PatternId(usize);

pub use hyperast_tsquery::Symbol;

pub struct Query {
    q: *mut crate::search::steped::TSQuery,
    capture_names: Vec<&'static str>,
    capture_quantifiers_vec: Vec<Vec<tree_sitter::CaptureQuantifier>>,
    text_predicates: pred::TextPredicateCaptures,
    property_predicates_vec: Vec<()>,
    property_settings_vec: Vec<()>,
    general_predicates_vec: Vec<()>,
}

mod pred {
    use super::query_cursor::TextPredicateCapture;

    pub type TextPredsBuilder<C = u32> = PredsBuilder<TextPredicateCapture<C>>;

    pub struct PredsBuilder<P> {
        curr: Option<Vec<P>>,
        acc: Vec<Box<[P]>>,
    }

    impl<P> PredsBuilder<P> {
        pub fn with_patt_count(pat_count: usize) -> Self {
            Self {
                curr: None,
                acc: Vec::with_capacity(pat_count),
            }
        }
        pub fn prep(&mut self) {
            if let Some(curr) = self.curr.take() {
                self.acc.push(curr.into());
            }
            self.curr = Some(vec![])
        }
        pub fn push(&mut self, value: P) {
            self.curr.as_mut().unwrap().push(value)
        }
        pub fn build(mut self) -> Predicates<P> {
            if let Some(curr) = self.curr.take() {
                self.acc.push(curr.into());
            }
            Predicates(self.acc.into())
        }
    }
    pub struct Predicates<P>(Box<[Box<[P]>]>);
    pub type TextPredicateCaptures<C = u32> = Predicates<TextPredicateCapture<C>>;

    impl<P> Predicates<P> {
        pub fn preds_for_patern_id<'a>(&'a self, id: usize) -> impl Iterator<Item = &'a P> {
            self.0[id].iter()
        }
    }
}

mod pred_opt {
    use crate::search::steped::query_cursor::TextPredicateCapture;

    struct TextPredsBuilder<C = u32> {
        offsets: Vec<usize>,
        variants: Vec<u8>,
        captures: Vec<C>,
    }

    impl TextPredsBuilder {
        fn with_patt_count(pat_count: usize) -> Self {
            Self {
                offsets: Vec::with_capacity(pat_count),
                variants: Default::default(),
                captures: Default::default(),
            }
        }
    }

    struct TextPredicateCaptures {
        offsets: Vec<usize>,
        variants: Vec<u8>,
        captures: Vec<u32>,
        // either capture, text_offset, regex_offset, or text_set offset
        other: Vec<u32>,
        text: Vec<u8>,
        // regex: Vec<Regex>,
        // is_positive: bitvec::vec::BitVec,
        // match_all_nodes: bitvec::vec::BitVec,
    }

    impl TextPredicateCaptures {
        fn preds_for_patern_id(&self, id: usize) -> impl Iterator<Item = TextPredicateCapture> {
            struct It {}
            impl Iterator for It {
                type Item = TextPredicateCapture;

                fn next(&mut self) -> Option<Self::Item> {
                    todo!()
                }
            }
            let variants = &self.variants[self.offsets[id]..self.offsets[id + 1]];
            let captures = &self.captures[self.offsets[id]..self.offsets[id + 1]];

            It {}
        }
    }
}

impl Drop for Query {
    fn drop(&mut self) {
        unsafe { tree_sitter::ffi::ts_query_delete(std::mem::transmute(self.q)) }
    }
}
impl Debug for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.q.fmt(f)
    }
}

pub struct MatchIt<'query, Cursor, Node>(QueryCursor<'query, Cursor, Node>);
impl<'query, Cursor: self::Cursor> Iterator for MatchIt<'query, Cursor, Cursor::Node>
where
    <Cursor::Status as Status>::IdF: Into<u16> + From<u16>,
{
    type Item = query_cursor::QueryMatch<Cursor::Node>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let result = self.0.next_match()?;
            //
            if result.satisfies_text_predicates(
                self.0.text_provider(),
                self.0
                    .query
                    .text_predicates_for_pattern_id(result.pattern_index),
            ) {
                dbg!();
                return Some(result);
            }
        }
    }
}

impl Query {
    pub fn new(
        source: &str,
        language: tree_sitter::Language,
    ) -> Result<Self, tree_sitter::QueryError> {
        let ptr = Self::init_tsquery(source, language)?;
        let query: *mut TSQuery = unsafe { std::mem::transmute(ptr) };
        let ptr = {
            struct TSQueryDrop(*mut ffi::TSQuery);
            impl Drop for TSQueryDrop {
                fn drop(&mut self) {
                    unsafe { ffi::ts_query_delete(self.0) }
                }
            }
            TSQueryDrop(ptr)
        };

        let string_count = TSQuery::string_count(query) as u32;
        let capture_count = TSQuery::capture_count(query);
        let pattern_count = TSQuery::pattern_count(query);

        let mut capture_names = Vec::with_capacity(capture_count as usize);
        let mut capture_quantifiers_vec = Vec::with_capacity(pattern_count as usize);
        let mut text_predicates_vec = pred::TextPredsBuilder::with_patt_count(pattern_count);
        let mut property_predicates_vec = Vec::with_capacity(pattern_count);
        let mut property_settings_vec = Vec::with_capacity(pattern_count);
        let mut general_predicates_vec = Vec::with_capacity(pattern_count);

        // Build a vector of strings to store the capture names.
        for i in 0..capture_count {
            let name = TSQuery::capture_name(query, i as u32);
            capture_names.push(name);
        }
        // Build a vector to store capture quantifiers.
        for i in 0..pattern_count {
            let quantifiers = TSQuery::quantifiers_at_pattern(query, i);
            capture_quantifiers_vec.push(quantifiers);
        }

        use tree_sitter::ffi;

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
            // let mut property_predicates = Vec::new();
            // let mut property_settings = Vec::new();
            // let mut general_predicates = Vec::new();
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
                        text_predicates_vec.push(if p[2].type_ == TYPE_CAPTURE {
                            TextPredicateCapture::EqCapture(TextPred {
                                left: p[1].value_id,
                                right: p[2].value_id,
                                is_positive,
                                match_all_nodes,
                            })
                        } else {
                            TextPredicateCapture::EqString(TextPred {
                                left: p[1].value_id,
                                right: string_values[p[2].value_id as usize].to_string().into(),
                                is_positive,
                                match_all_nodes,
                            })
                        });
                    }
                    _ => {
                        return Err(predicate_error(
                            row,
                            format!("predicate {} not handled", operator_name),
                        ));
                    }
                }
            }
            // property_predicates_vec.push(property_predicates.into());
            // property_settings_vec.push(property_settings.into());
            // general_predicates_vec.push(general_predicates.into());
        }

        let text_predicates = text_predicates_vec.build();
        log::trace!("{}", &unsafe { query.as_ref() }.unwrap());
        let query = Query {
            q: query,
            capture_names,
            capture_quantifiers_vec,
            text_predicates,
            general_predicates_vec,
            property_predicates_vec,
            property_settings_vec,
        };
        std::mem::forget(ptr);
        Ok(query)
    }

    pub fn init_tsquery(
        source: &str,
        language: tree_sitter::Language,
    ) -> Result<*mut tree_sitter::ffi::TSQuery, tree_sitter::QueryError> {
        let mut error_offset = 0u32;
        let mut error_type: tree_sitter::ffi::TSQueryError = 0;
        let bytes = source.as_bytes();
        // Compile the query.
        let ptr = unsafe {
            tree_sitter::ffi::ts_query_new(
                language.into_raw(),
                bytes.as_ptr().cast::<std::ffi::c_char>(),
                bytes.len() as u32,
                std::ptr::addr_of_mut!(error_offset),
                std::ptr::addr_of_mut!(error_type),
            )
        };

        // On failure, build an error based on the error code and offset.
        if ptr.is_null() {
            use tree_sitter::QueryError;
            use tree_sitter::QueryErrorKind;
            use tree_sitter::ffi;
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

    pub fn pattern_count(&self) -> usize {
        TSQuery::pattern_count(self.q as *const TSQuery)
    }

    pub fn matches<'query, Cursor: self::Cursor>(
        &'query self,
        cursor: Cursor,
    ) -> MatchIt<'query, Cursor, Cursor::Node> {
        let qcursor = QueryCursor::<Cursor, _> {
            halted: false,
            ascending: false,
            states: vec![],
            capture_list_pool: crate::search::steped::CaptureListPool::default(),
            finished_states: Default::default(),
            max_start_depth: u32::MAX,
            did_exceed_match_limit: false,
            depth: 0,
            on_visible_node: true,
            query: self,
            cursor,
            next_state_id: 0,
        };
        MatchIt(qcursor)
    }

    /// Match all patterns that starts on cursor current node
    pub fn matches_immediate<'query, Cursor: self::Cursor>(
        &'query self,
        cursor: Cursor,
    ) -> MatchIt<'query, Cursor, Cursor::Node> {
        let mut qcursor = QueryCursor::<Cursor, _> {
            halted: false,
            ascending: false,
            states: vec![],
            capture_list_pool: crate::search::steped::CaptureListPool::default(),
            finished_states: Default::default(),
            max_start_depth: u32::MAX,
            did_exceed_match_limit: false,
            depth: 0,
            on_visible_node: true,
            query: self,
            cursor,
            next_state_id: 0,
        };
        // can only match patterns starting on provided node
        qcursor.set_max_start_depth(0);
        MatchIt(qcursor)
    }

    pub fn capture_index_for_name(&self, name: &str) -> Option<u32> {
        self.capture_names
            .iter()
            .position(|x| *x == name)
            .map(|i| i as u32)
    }

    pub fn capture_quantifiers(
        &self,
        index: usize,
    ) -> impl std::ops::Index<usize, Output = tree_sitter::CaptureQuantifier> {
        self.capture_quantifiers_vec[index].clone()
    }

    pub fn capture_name(&self, i: u32) -> &str {
        self.capture_names[i as usize]
    }

    fn text_predicates_for_pattern_id<'a>(
        &'a self,
        pattern_index: usize,
    ) -> impl Iterator<Item = &'a TextPredicateCapture> {
        self.text_predicates.preds_for_patern_id(pattern_index)
    }
}

#[must_use]
const fn predicate_error(row: usize, message: String) -> tree_sitter::QueryError {
    tree_sitter::QueryError {
        kind: tree_sitter::QueryErrorKind::Predicate,
        row,
        column: 0,
        offset: 0,
        message,
    }
}

pub struct QueryCursor<'query, Cursor, Node> {
    halted: bool,
    ascending: bool,
    on_visible_node: bool,
    cursor: Cursor,
    query: &'query Query,
    states: Vec<State>,
    depth: u32,
    max_start_depth: u32,
    capture_list_pool: CaptureListPool<Node>,
    finished_states: VecDeque<State>,
    next_state_id: u32,
    // only triggers when there is no more capture list available
    // not triggered by reaching max_start_depth
    did_exceed_match_limit: bool,
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
    fn acquire(&mut self) -> u32 {
        // First see if any already allocated capture list is currently unused.
        if self.free_capture_list_count > 0 {
            for i in 0..self.list.len() {
                if self.list[i].len() == 0 {
                    self.list[i].clear();
                    self.free_capture_list_count -= 1;
                    return i as u32;
                }
            }
        }

        // Otherwise allocate and initialize a new capture list, as long as that
        // doesn't put us over the requested maximum.
        let i = self.list.len();
        if i >= self.max_capture_list_count as usize {
            return u32::MAX;
        }
        self.list.push(vec![]);
        return i as u32;
    }
}

#[derive(Clone)]
pub struct Capture<Node, I = u32> {
    pub node: Node,
    pub index: I,
}

#[derive(Clone)]
#[repr(C)]
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

#[repr(C)]
struct PatternEntry {
    step_index: u16,
    pattern_index: u16,
    is_rooted: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum TreeCursorStep {
    TreeCursorStepNone,
    TreeCursorStepHidden,
    TreeCursorStepVisible,
}

pub trait Cursor {
    type Node: Node;

    fn goto_next_sibling_internal(&mut self) -> TreeCursorStep;

    fn goto_first_child_internal(&mut self) -> TreeCursorStep;

    fn goto_parent(&mut self) -> bool;
    fn current_node(&self) -> Self::Node;

    fn parent_node(&self) -> Option<Self::Node>;

    type Status: Status<IdF = <Self::Node as Node>::IdF>;

    fn current_status(&self) -> Self::Status;

    // fn is_subtree_repetition(&self) -> bool {
    //     unimplemented!("related to query analysis, don't know how to handle that for now")
    // }
    // fn subtree_symbol(&self) -> tree_sitter::ffi::TSSymbol {
    //     unimplemented!("related to query analysis, don't know how to handle that for now")
    // }

    fn text_provider(&self) -> <Self::Node as TextLending<'_>>::TP;
}

pub struct TSTreeCursor<'a> {
    text: &'a [u8],
    cursor: tree_sitter::TreeCursor<'a>,
}

impl<'a> TSTreeCursor<'a> {
    pub fn new(text: &'a [u8], cursor: tree_sitter::TreeCursor<'a>) -> Self {
        Self { text, cursor }
    }
}

impl<'a> Cursor for TSTreeCursor<'a> {
    type Node = tree_sitter::Node<'a>;

    fn goto_next_sibling_internal(&mut self) -> TreeCursorStep {
        unsafe extern "C" {
            pub fn ts_tree_cursor_goto_next_sibling_internal(
                self_: *mut tree_sitter::ffi::TSTreeCursor,
            ) -> TreeCursorStep;
        }
        unsafe {
            let s: *mut tree_sitter::ffi::TSTreeCursor = std::mem::transmute(&mut self.cursor);
            ts_tree_cursor_goto_next_sibling_internal(s)
        }
    }

    fn goto_first_child_internal(&mut self) -> TreeCursorStep {
        unsafe extern "C" {
            pub fn ts_tree_cursor_goto_first_child_internal(
                self_: *mut tree_sitter::ffi::TSTreeCursor,
            ) -> TreeCursorStep;
        }
        unsafe {
            let s: *mut tree_sitter::ffi::TSTreeCursor = std::mem::transmute(&mut self.cursor);
            ts_tree_cursor_goto_first_child_internal(s)
        }
    }

    fn goto_parent(&mut self) -> bool {
        self.cursor.goto_parent()
    }

    fn current_node(&self) -> Self::Node {
        self.cursor.node()
    }

    fn parent_node(&self) -> Option<Self::Node> {
        unsafe extern "C" {
            pub fn ts_tree_cursor_parent_node(
                self_: *const tree_sitter::ffi::TSTreeCursor,
            ) -> tree_sitter::ffi::TSNode;
        }
        unsafe {
            let s: *const tree_sitter::ffi::TSTreeCursor = std::mem::transmute(&self.cursor);
            let n = ts_tree_cursor_parent_node(s);
            if tree_sitter::ffi::ts_node_is_null(n) {
                return None;
            }
            let n: tree_sitter::Node = std::mem::transmute(n);
            Some(n)
        }
    }

    type Status = TSStatus;

    #[inline]
    fn current_status(&self) -> TSStatus {
        unsafe extern "C" {
            pub fn ts_tree_cursor_current_status(
                self_: *const tree_sitter::ffi::TSTreeCursor,
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
            let s: *const tree_sitter::ffi::TSTreeCursor = std::mem::transmute(&self.cursor);
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
            TSStatus {
                has_later_siblings,
                has_later_named_siblings,
                can_have_later_siblings_with_this_field,
                field_id,
                supertypes,
            }
        }
    }

    fn text_provider(&self) -> <Self::Node as TextLending<'_>>::TP {
        self.text
    }
}

pub trait Status {
    type IdF;
    fn has_later_siblings(&self) -> bool;
    fn has_later_named_siblings(&self) -> bool;
    fn can_have_later_siblings_with_this_field(&self) -> bool;
    fn field_id(&self) -> Self::IdF;
    fn has_supertypes(&self) -> bool;
    fn contains_supertype(&self, sym: Symbol) -> bool;
}

pub struct TSStatus {
    pub has_later_siblings: bool,
    pub has_later_named_siblings: bool,
    pub can_have_later_siblings_with_this_field: bool,
    pub field_id: tree_sitter::ffi::TSFieldId,
    pub supertypes: Vec<Symbol>,
}

impl Status for TSStatus {
    type IdF = tree_sitter::ffi::TSFieldId;

    fn has_later_siblings(&self) -> bool {
        self.has_later_siblings
    }

    fn has_later_named_siblings(&self) -> bool {
        self.has_later_named_siblings
    }

    fn can_have_later_siblings_with_this_field(&self) -> bool {
        self.can_have_later_siblings_with_this_field
    }

    fn field_id(&self) -> Self::IdF {
        self.field_id
    }

    fn has_supertypes(&self) -> bool {
        !self.supertypes.is_empty()
    }

    fn contains_supertype(&self, sym: Symbol) -> bool {
        self.supertypes.contains(&sym)
    }
}

use hyperast_tsquery::BiCow;
use hyperast_tsquery::Node;
use hyperast_tsquery::TextLending;

// mod analyze;

#[cfg(test)]
mod convert;

pub mod hyperast;

pub mod query_cursor;

impl<'query, Cursor: self::Cursor> QueryCursor<'query, Cursor, Cursor::Node> {
    /// Set the max depth where queries can start being matched
    /// For example, set it to 0 to only match on the node you start on.
    pub fn set_max_start_depth(&mut self, max: u32) {
        self.max_start_depth = max;
    }

    fn should_descend(&self, node_intersects_range: bool) -> bool {
        if node_intersects_range && self.depth < self.max_start_depth {
            return true;
        }

        // If there are in-progress matches whose remaining steps occur
        // deeper in the tree, then descend.
        for i in 0..self.states.len() {
            let state = &self.states[i];
            let next_step = &unsafe { &(*self.query.q).steps }[state.step_index as usize];
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

    fn copy_state(&mut self, state_index: &mut usize) -> Option<usize> {
        let state = &self.states[*state_index];
        let capture_list_id = state.capture_list_id;
        let mut copy = state.clone();
        copy.capture_list_id = u32::MAX;

        self.states.insert(*state_index + 1, copy);
        // dbg!(capture_list_id);
        // If the state has captures, copy its capture list.
        if capture_list_id != u32::MAX {
            let new_captures = self.prepare_to_capture(*state_index + 1, *state_index as u32)?;
            let old_captures = self.capture_list_pool.get(capture_list_id);
            self.capture_list_pool.list[new_captures] = old_captures.to_vec();
        }
        return Some(*state_index + 1);
    }

    fn compare_captures(&self, left_state: &State, right_state: &State) -> (bool, bool) {
        let left_captures = self.capture_list_pool.get(left_state.capture_list_id);
        let right_captures = self.capture_list_pool.get(right_state.capture_list_id);
        let mut left_contains_right = true;
        let mut right_contains_left = true;
        let mut i = 0;
        let mut j = 0;
        loop {
            if i < left_captures.len() {
                if j < right_captures.len() {
                    let left = &left_captures[i];
                    let right = &right_captures[j];
                    if left.node.equal(&right.node) && left.index == right.index {
                        i += 1;
                        j += 1;
                    } else {
                        match left.node.compare(&right.node) {
                            std::cmp::Ordering::Less => {
                                right_contains_left = false;
                                i += 1;
                            }
                            std::cmp::Ordering::Greater => {
                                left_contains_right = false;
                                j += 1;
                            }
                            std::cmp::Ordering::Equal => {
                                right_contains_left = false;
                                left_contains_right = false;
                                i += 1;
                                j += 1;
                            }
                        }
                    }
                } else {
                    right_contains_left = false;
                    break;
                }
            } else {
                if j < right_captures.len() {
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
            if state.dead {
                continue;
            };

            let captures = self.capture_list_pool.get(state.capture_list_id);
            if state.consumed_capture_count as usize >= captures.len() {
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
    ) -> Option<usize> {
        let state = &mut self.states[state_id];
        if state.capture_list_id == u32::MAX {
            state.capture_list_id = self.capture_list_pool.acquire();

            // If there are no capture lists left in the pool, then terminate whichever
            // state has captured the earliest node in the document, and steal its
            // capture list.
            if state.capture_list_id == u32::MAX {
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
                        other_state.capture_list_id = u32::MAX; // TODO handle NONE size stuff...
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
        let step = &unsafe { &(*self.query.q).steps }[step_id];
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
        let step = &unsafe { &(*self.query.q).steps }[pattern.step_index as usize];
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
            capture_list_id: u32::MAX,
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

    fn text_provider(&self) -> <Cursor::Node as TextLending<'_>>::TP {
        self.cursor.text_provider()
    }
}

use query_cursor::{TextPred, TextPredicateCapture};
pub use query_step::TSQuery;

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

impl std::fmt::Display for TSQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "query steps:\n")?;
        let steps = &self.steps;
        for i in 0..steps.size {
            let step = unsafe { steps.contents.add(i as usize).as_ref().unwrap() };
            write!(f, "  {:>2}: ", i)?;
            print_query_step(self, step, f)?;
            write!(f, ",\n")?;
        }
        Ok(())
    }
}
mod query_step;
use std::collections::VecDeque;
use std::fmt::Debug;

pub(crate) use query_step::print_query_step;
