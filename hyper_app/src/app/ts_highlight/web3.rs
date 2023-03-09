use js_sys::Array;
use serde::Deserialize;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_tree_sitter_sg::*;
//{//LossyUtf8,
// QueryCapture, //QueryCursor,
// Query
// QueryMatch};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{iter, mem, ops, str, usize};
use thiserror::Error;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Node<'tree> {
    pub(crate) inner: SyntaxNode,
    pub(crate) phantom: std::marker::PhantomData<&'tree ()>,
}

impl<'tree> From<SyntaxNode> for Node<'tree> {
    fn from(inner: SyntaxNode) -> Self {
        Self {
            inner,
            phantom: Default::default(),
        }
    }
}
struct QueryState;
struct CaptureListPool;

struct QueryCursor {
    query: Option<Query>,
    cursor: Option<TreeCursor>,
    states: Vec<QueryState>,
    finished_states: Vec<QueryState>,
    capture_list_pool: CaptureListPool,
    depth: u32,
    start_byte: u32,
    end_byte: u32,
    start_point: Point,
    end_point: Point,
    next_state_id: u32,
    on_visible_node: bool,
    ascending: bool,
    halted: bool,
    did_exceed_match_limit: bool,
}

impl QueryCursor {
    fn new() -> Self {
        Self {
            did_exceed_match_limit: false,
            ascending: false,
            halted: false,
            states: vec![],
            finished_states: vec![],
            capture_list_pool: CaptureListPool, //capture_list_pool_new(),
            start_byte: 0,
            end_byte: u32::MAX,
            start_point: Point::new(0, 0),
            end_point: Point::new(u32::MAX, u32::MAX),
            query: None,
            cursor: None,
            depth: 0,
            next_state_id: 0,
            on_visible_node: false,
        }
    }

    fn start_position(&self) -> Option<&Point> {
        Some(&self.start_point)
    }
    fn end_position(&self) -> Option<&Point> {
        Some(&self.end_point)
    }

    fn matches<'a, 'tree, T>(
        &mut self,
        query: &'a Query,
        node: Node<'tree>,
        text_provider: T,
    ) -> QueryMatches<'a, 'tree, T> {
        let inner = query.matches(&node.inner, self.start_position(), self.end_position());
        QueryMatches {
            i: 0,
            inner,
            text_provider,
            phantom: Default::default(),
        }
    }
    fn captures<'a, 'tree: 'a, T: 'a>(
        &'a mut self,
        query: &'a Query,
        node: Node<'tree>,
        text_provider: T,
    ) -> QueryCaptures<'a, 'tree, T> {
        let inner = query.captures(&node.inner, self.start_position(), self.end_position());
        QueryCaptures {
            i: 0,
            inner,
            text_provider,
            phantom: Default::default(),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    pub type Query;

    #[wasm_bindgen(method, getter)]
    fn predicates(this: &Query) -> Box<[JsValue]>;

    // Instance Properties

    // -> JsString[]
    #[wasm_bindgen(method, getter, js_name = captureNames)]
    pub fn capture_names(this: &Query) -> Box<[JsValue]>;

    // Instance Methods

    #[wasm_bindgen(method)]
    pub fn delete(this: &Query);

    // -> QueryMatch[]
    #[wasm_bindgen(method)]
    pub fn matches(
        this: &Query,
        node: &SyntaxNode,
        start_position: Option<&Point>,
        end_position: Option<&Point>,
    ) -> Box<[JsValue]>;

    // -> QueryCapture[]
    #[wasm_bindgen(method)]
    pub fn captures(
        this: &Query,
        node: &SyntaxNode,
        start_position: Option<&Point>,
        end_position: Option<&Point>,
    ) -> Box<[JsValue]>;

    // -> PredicateResult[]
    #[wasm_bindgen(method, js_name = predicatesForPattern)]
    pub fn predicates_for_pattern(this: &Query, pattern_index: u32) -> Box<[JsValue]>;
}
impl Query {
    fn pattern_count(&self) -> usize {
        let predicates = js_sys::Reflect::get(self, &"predicates".into()).unwrap();
        let len = self.predicates().len();
        assert!(predicates.is_array());
        assert_eq!(len, Array::from(&predicates).length() as usize);
        self.predicates().len()
    }
    fn start_byte_for_pattern(&self, pattern_index: usize) -> usize {
        eframe::web_sys::console::log_1(&pattern_index.into());
        let predicates = js_sys::Reflect::get(self, &"predicates".into()).unwrap();
        todo!()
    }
    fn property_settings(&self, index: usize) -> &[QueryProperty] {
        todo!()
    }
    fn disable_pattern(&mut self, index: usize) {
        todo!()
    }
    fn property_predicates(&self, index: usize) -> &[(QueryProperty, bool)] {
        todo!()
    }
}
struct QueryProperty {
    key: String,
}

impl From<web_tree_sitter_sg::Query> for Query {
    fn from(value: web_tree_sitter_sg::Query) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

/// Performs syntax highlighting, recognizing a given list of highlight names.
///
/// For the best performance `Highlighter` values should be reused between
/// syntax highlighting calls. A separate highlighter is needed for each thread that
/// is performing highlighting.
pub struct Highlighter {
    parser: Parser,
    cursors: Vec<QueryCursor>,
}

impl Highlighter {
    pub fn new() -> Self {
        Highlighter {
            parser: Parser::new().unwrap(),
            cursors: Vec::new(),
        }
    }

    pub fn parser(&mut self) -> &mut Parser {
        &mut self.parser
    }

    /// Iterate over the highlighted regions for a given slice of source code.
    pub fn highlight<'a>(
        &'a mut self,
        config: &'a HighlightConfiguration,
        source: &'a [u8],
        cancellation_flag: Option<&'a AtomicUsize>,
        mut injection_callback: impl FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a,
    ) -> Result<impl Iterator<Item = Result<HighlightEvent, Error>> + 'a, Error> {
        let layers = HighlightIterLayer::new(
            source,
            self,
            cancellation_flag,
            &mut injection_callback,
            config,
            0,
            vec![R {
                start_byte: 0,
                end_byte: u32::MAX,
                start_point: Point::new(0, 0),
                end_point: Point::new(u32::MAX, u32::MAX),
            }
            .into()],
        )?;
        eframe::web_sys::console::log_1(&layers.len().into());
        assert_ne!(layers.len(), 0);
        let mut result = HighlightIter {
            source,
            byte_offset: 0,
            injection_callback,
            cancellation_flag,
            highlighter: self,
            iter_count: 0,
            layers: layers,
            next_event: None,
            last_highlight_range: None,
        };
        result.sort_layers();
        Ok(result)
    }
}

struct QueryCaptures<'cursor, 'tree, T> {
    i: usize,
    inner: Box<[JsValue]>,
    text_provider: T,
    pub(crate) phantom: std::marker::PhantomData<&'cursor [&'tree QueryCapture]>,
}

impl<'cursor, 'tree, T> Iterator for QueryCaptures<'cursor, 'tree, T> {
    type Item = (QueryMatch, usize); // <'a, 'tree>

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.inner.len() {
            return None;
        }
        let i = self.i;
        let r = &self.inner[i];
        self.i += 1;

        eframe::web_sys::console::log_1(&r);
        assert!(r.is_object());

        // let r = js_sys::Object::try_from(r).unwrap();
        // let name = js_sys::Reflect::get(r, &"name".into()).unwrap();
        // let name = js_sys::JsString::from(name);
        // let node = js_sys::Reflect::get(r, &"node".into()).unwrap();
        // assert!(node.is_object());
        // let node = unsafe { std::mem::transmute(node) };
        // let capture = QueryCapture::new(&name, &node);

        // let pattern = js_sys::Reflect::get(r, &"pattern".into()).unwrap();
        // let pattern = pattern.as_f64().unwrap() as u32;
        // let captures = js_sys::Reflect::get(r, &"captures".into()).unwrap();
        // let captures = js_sys::Array::from(&captures);
        let pattern = 0;
        let captures = Array::of1(r);
        Some((QueryMatch::new(pattern, &captures), i))
    }
}

struct QueryMatches<'cursor, 'tree, T> {
    i: usize,
    inner: Box<[JsValue]>,
    text_provider: T,
    pub(crate) phantom: std::marker::PhantomData<&'cursor [&'tree QueryCapture]>,
}

impl<'cursor, 'tree, T> QueryMatches<'cursor, 'tree, T> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'cursor, 'tree, T> Iterator for QueryMatches<'cursor, 'tree, T> {
    type Item = QueryMatch;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.inner.len() {
            return None;
        }
        let r = &self.inner[self.i];
        self.i += 1;
        // let r = js_sys::Object::try_from(r).unwrap();
        let pattern = js_sys::Reflect::get(r, &"pattern".into()).unwrap();
        let pattern = pattern.as_f64().unwrap() as u32;
        let captures = js_sys::Reflect::get(r, &"captures".into()).unwrap();
        let captures = js_sys::Array::from(&captures);
        Some(QueryMatch::new(pattern, &captures))
    }
}

#[inline]
pub fn parse(
    parser: &mut Parser,
    text: impl AsRef<[u8]>,
    old_tree: Option<&Tree>,
    ranges: &[Range],
) -> Result<Option<Tree>, ParserError> {
    let text = text.as_ref();
    let text = unsafe { std::str::from_utf8_unchecked(text) };
    let text = &text.into();
    let array = ranges.iter().map(JsValue::from).collect();
    let options = ParseOptions::new(Some(&array));
    parser
        .parse_with_string(text, old_tree, Some(&options))
        .map(|ok| ok.map(Into::into))
        .map_err(Into::into)
}

impl<'a> HighlightIterLayer<'a> {
    /// Create a new 'layer' of highlighting for this document.
    ///
    /// In the even that the new layer contains "combined injections" (injections where multiple
    /// disjoint ranges are parsed as one syntax tree), these will be eagerly processed and
    /// added to the returned vector.
    fn new<F: FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a>(
        source: &'a [u8],
        highlighter: &mut Highlighter,
        cancellation_flag: Option<&'a AtomicUsize>,
        injection_callback: &mut F,
        mut config: &'a HighlightConfiguration,
        mut depth: usize,
        mut ranges: Vec<Range>,
    ) -> Result<Vec<Self>, Error> {
        let mut result = Vec::with_capacity(1);
        let mut queue = Vec::new();
        loop {
            // let opt = ParseOptions::new();
            // let o = highlighter.parser.;
            // let p = highlighter.parser.parse_with_string(text, old_tree, Some(&options));
            // if highlighter.parser.set_included_ranges(&ranges).is_ok() {

            highlighter
                .parser
                .set_language(Some(&config.language))
                .map_err(|_| Error::InvalidLanguage)?;
            // unsafe { highlighter.parser.set_cancellation_flag(cancellation_flag) };
            let tree = parse(&mut highlighter.parser, source, None, &ranges)
                .unwrap()
                .ok_or(Error::Cancelled)?;
            eframe::web_sys::console::log_1(&"aaaa".into());
            // unsafe { highlighter.parser.set_cancellation_flag(None) };
            let mut cursor = highlighter.cursors.pop().unwrap_or(QueryCursor::new());
            // Process combined injections.
            if let Some(combined_injections_query) = &config.combined_injections_query {
                eframe::web_sys::console::log_1(&"bbbb".into());
                let mut injections_by_pattern_index =
                    vec![(None, Vec::new(), false); combined_injections_query.pattern_count()];
                let matches =
                    cursor.matches(&combined_injections_query, tree.root_node().into(), source);
                eframe::web_sys::console::log_1(&"bbbb2".into());
                eframe::web_sys::console::log_1(&matches.len().into());
                for mat in matches {
                    let entry = &mut injections_by_pattern_index[mat.pattern() as usize];
                    let (language_name, content_node, include_children) =
                        injection_for_match(config, combined_injections_query, &mat, source);
                    if language_name.is_some() {
                        entry.0 = language_name;
                    }
                    if let Some(content_node) = content_node {
                        entry.1.push(content_node);
                    }
                    entry.2 = include_children;
                }
                for (lang_name, content_nodes, includes_children) in injections_by_pattern_index {
                    if let (Some(lang_name), false) = (lang_name, content_nodes.is_empty()) {
                        if let Some(next_config) = (injection_callback)(&lang_name) {
                            let ranges =
                                Self::intersect_ranges(&ranges, &content_nodes, includes_children);
                            if !ranges.is_empty() {
                                queue.push((next_config, depth + 1, ranges));
                            }
                        }
                    }
                }
            }
            // The `captures` iterator borrows the `Tree` and the `QueryCursor`, which
            // prevents them from being moved. But both of these values are really just
            // pointers, so it's actually ok to move them.
            let tree_ref = unsafe { mem::transmute::<_, &'static Tree>(&tree) };
            let cursor_ref = unsafe { mem::transmute::<_, &'static mut QueryCursor>(&mut cursor) };

            let captures = cursor_ref
                .captures(&config.query, tree_ref.root_node().into(), source)
                .peekable();

            result.push(HighlightIterLayer {
                highlight_end_stack: Vec::new(),
                scope_stack: vec![LocalScope {
                    inherits: false,
                    range: 0..usize::MAX,
                    local_defs: Vec::new(),
                }],
                cursor,
                depth,
                _tree: tree,
                captures,
                config,
                ranges,
            });

            // }
            if queue.is_empty() {
                break;
            } else {
                let (next_config, next_depth, next_ranges) = queue.remove(0);
                config = next_config;
                depth = next_depth;
                ranges = next_ranges;
            }
        }
        Ok(result)
    }

    // Compute the ranges that should be included when parsing an injection.
    // This takes into account three things:
    // * `parent_ranges` - The ranges must all fall within the *current* layer's ranges.
    // * `nodes` - Every injection takes place within a set of nodes. The injection ranges
    //   are the ranges of those nodes.
    // * `includes_children` - For some injections, the content nodes' children should be
    //   excluded from the nested document, so that only the content nodes' *own* content
    //   is reparsed. For other injections, the content nodes' entire ranges should be
    //   reparsed, including the ranges of their children.
    fn intersect_ranges(
        parent_ranges: &[Range],
        nodes: &[Node<'_>],
        includes_children: bool,
    ) -> Vec<Range> {
        todo!()
    }

    // First, sort scope boundaries by their byte offset in the document. At a
    // given position, emit scope endings before scope beginnings. Finally, emit
    // scope boundaries from deeper layers first.
    fn sort_key(&mut self) -> Option<(usize, bool, isize)> {
        let depth = -(self.depth as isize);
        let next_start = self
            .captures
            .peek()
            .map(|(m, i)| to_query_capture(&m.captures()[*i]).node().start_index() as usize); //_byte());
        let next_end = self.highlight_end_stack.last().cloned();
        match (next_start, next_end) {
            (Some(start), Some(end)) => {
                if start < end {
                    Some((start, true, depth))
                } else {
                    Some((end, false, depth))
                }
            }
            (Some(i), None) => Some((i, true, depth)),
            (None, Some(j)) => Some((j, false, depth)),
            _ => None,
        }
    }
}

fn to_query_capture(r: &JsValue) -> QueryCapture {
    let r = js_sys::Object::try_from(r).unwrap();
    let name = js_sys::Reflect::get(r, &"name".into()).unwrap();
    let name = js_sys::JsString::from(name);
    let node = js_sys::Reflect::get(r, &"node".into()).unwrap();
    assert!(node.is_object());
    let node = unsafe { std::mem::transmute(node) };
    QueryCapture::new(&name, &node)
}

struct HighlightIterLayer<'a> {
    _tree: Tree,
    cursor: QueryCursor,
    captures: iter::Peekable<QueryCaptures<'a, 'a, &'a [u8]>>,
    config: &'a HighlightConfiguration,
    highlight_end_stack: Vec<usize>,
    scope_stack: Vec<LocalScope<'a>>,
    ranges: Vec<Range>,
    depth: usize,
}

#[derive(Debug)]
struct LocalDef<'a> {
    name: &'a str,
    value_range: ops::Range<usize>,
    highlight: Option<Highlight>,
}

#[derive(Debug)]
struct LocalScope<'a> {
    inherits: bool,
    range: ops::Range<usize>,
    local_defs: Vec<LocalDef<'a>>,
}

// struct HighlightIter<'a, F>
// where
//     F: FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a,
// {
//     source: &'a [u8],
//     byte_offset: usize,
//     highlighter: &'a mut Highlighter,
//     injection_callback: F,
//     cancellation_flag: Option<&'a AtomicUsize>,
//     layers: Vec<HighlightIterLayer<'a>>,
//     iter_count: usize,
//     next_event: Option<HighlightEvent>,
//     last_highlight_range: Option<(usize, usize, usize)>,
// }

// impl<'a, F> Iterator for HighlightIter<'a, F>
// where
//     F: FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a,
// {
//     type Item = Result<HighlightEvent, Error>;

//     fn next(&mut self) -> Option<Self::Item> {
//        todo!()
//     }
// }

struct HighlightIter<'a, F>
where
    F: FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a,
{
    source: &'a [u8],
    byte_offset: usize,
    highlighter: &'a mut Highlighter,
    injection_callback: F,
    cancellation_flag: Option<&'a AtomicUsize>,
    layers: Vec<HighlightIterLayer<'a>>,
    iter_count: usize,
    next_event: Option<HighlightEvent>,
    last_highlight_range: Option<(usize, usize, usize)>,
}

impl<'a, F> HighlightIter<'a, F>
where
    F: FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a,
{
    fn emit_event(
        &mut self,
        offset: usize,
        event: Option<HighlightEvent>,
    ) -> Option<Result<HighlightEvent, Error>> {
        let result;
        if self.byte_offset < offset {
            result = Some(Ok(HighlightEvent::Source {
                start: self.byte_offset,
                end: offset,
            }));
            self.byte_offset = offset;
            self.next_event = event;
        } else {
            result = event.map(Ok);
        }
        self.sort_layers();
        result
    }

    fn sort_layers(&mut self) {
        while !self.layers.is_empty() {
            if let Some(sort_key) = self.layers[0].sort_key() {
                let mut i = 0;
                while i + 1 < self.layers.len() {
                    if let Some(next_offset) = self.layers[i + 1].sort_key() {
                        if next_offset < sort_key {
                            i += 1;
                            continue;
                        }
                    }
                    break;
                }
                if i > 0 {
                    self.layers[0..(i + 1)].rotate_left(1);
                }
                break;
            } else {
                let layer = self.layers.remove(0);
                self.highlighter.cursors.push(layer.cursor);
            }
        }
    }
}

impl<'a, F> Iterator for HighlightIter<'a, F>
where
    F: FnMut(&str) -> Option<&'a HighlightConfiguration> + 'a,
{
    type Item = Result<HighlightEvent, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        'main: loop {
            // If we've already determined the next highlight boundary, just return it.
            if let Some(e) = self.next_event.take() {
                return Some(Ok(e));
            }

            // Periodically check for cancellation, returning `Cancelled` error if the
            // cancellation flag was flipped.
            if let Some(cancellation_flag) = self.cancellation_flag {
                self.iter_count += 1;
                if self.iter_count >= CANCELLATION_CHECK_INTERVAL {
                    self.iter_count = 0;
                    if cancellation_flag.load(Ordering::Relaxed) != 0 {
                        return Some(Err(Error::Cancelled));
                    }
                }
            }

            // If none of the layers have any more highlight boundaries, terminate.
            if self.layers.is_empty() {
                return if self.byte_offset < self.source.len() {
                    let result = Some(Ok(HighlightEvent::Source {
                        start: self.byte_offset,
                        end: self.source.len(),
                    }));
                    self.byte_offset = self.source.len();
                    result
                } else {
                    None
                };
            }
            // Get the next capture from whichever layer has the earliest highlight boundary.
            let range;
            let layer = &mut self.layers[0];
            if let Some((next_match, capture_index)) = layer.captures.peek() {
                let next_capture = &next_match.captures()[*capture_index];
                let next_capture = to_query_capture(next_capture);
                let node = next_capture.node();
                range = node.start_index() as usize..node.end_index() as usize;

                // If any previous highlight ends before this node starts, then before
                // processing this capture, emit the source code up until the end of the
                // previous highlight, and an end event for that highlight.
                if let Some(end_byte) = layer.highlight_end_stack.last().cloned() {
                    if end_byte <= range.start {
                        layer.highlight_end_stack.pop();
                        return self.emit_event(end_byte, Some(HighlightEvent::HighlightEnd));
                    }
                }
            }
            // If there are no more captures, then emit any remaining highlight end events.
            // And if there are none of those, then just advance to the end of the document.
            else if let Some(end_byte) = layer.highlight_end_stack.last().cloned() {
                layer.highlight_end_stack.pop();
                return self.emit_event(end_byte, Some(HighlightEvent::HighlightEnd));
            } else {
                return self.emit_event(self.source.len(), None);
            };

            let (mut match_, capture_index) = layer.captures.next().unwrap();
            let mut capture = to_query_capture(&match_.captures()[capture_index]);

            // If this capture represents an injection, then process the injection.
            if (match_.pattern() as usize) < layer.config.locals_pattern_index {
                todo!()
            }

            // Remove from the local scope stack any local scopes that have already ended.
            while range.start > layer.scope_stack.last().unwrap().range.end {
                layer.scope_stack.pop();
            }

            // If this capture is for tracking local variables, then process the
            // local variable info.
            // let mut reference_highlight = None;
            // let mut definition_highlight = None;
            while (match_.pattern() as usize) < layer.config.highlights_pattern_index {
                // Continue processing any additional matches for the same node.
                if let Some((next_match, next_capture_index)) = layer.captures.peek() {
                    let next_capture = to_query_capture(&next_match.captures()[*next_capture_index]);
                    if next_capture.node() == capture.node() {
                        capture = next_capture;
                        match_ = layer.captures.next().unwrap().0;
                        continue;
                    }
                }

                self.sort_layers();
                continue 'main;
            }

            // Otherwise, this capture must represent a highlight.
            // If this exact range has already been highlighted by an earlier pattern, or by
            // a different layer, then skip over this one.
            if let Some((last_start, last_end, last_depth)) = self.last_highlight_range {
                if range.start == last_start && range.end == last_end && layer.depth < last_depth {
                    self.sort_layers();
                    continue 'main;
                }
            }

            // // If the current node was found to be a local variable, then skip over any
            // // highlighting patterns that are disabled for local variables.
            // if definition_highlight.is_some() || reference_highlight.is_some() {
            //     while layer.config.non_local_variable_patterns[match_.pattern() as usize] {
            //         match_.remove();
            //         if let Some((next_match, next_capture_index)) = layer.captures.peek() {
            //             let next_capture = &next_match.captures()[*next_capture_index];
            //             if next_capture.node == capture.node {
            //                 capture = next_capture;
            //                 match_ = layer.captures.next().unwrap().0;
            //                 continue;
            //             }
            //         }

            //         self.sort_layers();
            //         continue 'main;
            //     }
            // }

            // Once a highlighting pattern is found for the current node, skip over
            // any later highlighting patterns that also match this node. Captures
            // for a given node are ordered by pattern index, so these subsequent
            // captures are guaranteed to be for highlighting, not injections or
            // local variables.
            while let Some((next_match, next_capture_index)) = layer.captures.peek() {
                let next_capture = to_query_capture(&next_match.captures()[*next_capture_index]);
                if next_capture.node() == capture.node() {
                    layer.captures.next();
                } else {
                    break;
                }
            }

            eframe::web_sys::console::log_1(&capture);
            // let current_highlight = layer.config.highlight_indices[capture.index as usize];
            let current_highlight = layer.config.highlight_indices[0];

            // // If this node represents a local definition, then store the current
            // // highlight value on the local scope entry representing this node.
            // if let Some(definition_highlight) = definition_highlight {
            //     *definition_highlight = current_highlight;
            // }

            // Emit a scope start event and push the node's end position to the stack.
            if let Some(highlight) = current_highlight {
                self.last_highlight_range = Some((range.start, range.end, layer.depth));
                layer.highlight_end_stack.push(range.end);
                return self
                    .emit_event(range.start, Some(HighlightEvent::HighlightStart(highlight)));
            }

            self.sort_layers();
        }
    }
}
struct R {
    pub start_byte: u32,
    pub end_byte: u32,
    pub start_point: Point,
    pub end_point: Point,
}

impl From<R> for Range {
    fn from(value: R) -> Self {
        Range::new(
            &value.start_point,
            &value.end_point,
            value.start_byte,
            value.end_byte,
        )
    }
}

fn injection_for_match<'a>(
    config: &HighlightConfiguration,
    query: &'a Query,
    query_match: &QueryMatch,
    source: &'a [u8],
) -> (Option<String>, Option<Node<'a>>, bool) {
    let content_capture_index = config.injection_content_capture_index;
    let language_capture_index = config.injection_language_capture_index;

    let mut language_name = None;
    let mut content_node = None;
    for capture in query_match.captures().into_iter() {
        let capture: QueryCapture = (*capture).clone().try_into().unwrap();
        language_name = capture.name().as_string();
        content_node = Some(capture.node().into());
        // let index = Some(capture.index);
        // if index == language_capture_index {
        //     language_name = capture.node.utf8_text(source).ok();
        // } else if index == content_capture_index {
        //     content_node = Some(capture.node.into());
        // }
    }

    // let mut include_children = false;
    // let property_settings = if cfg!(wasm) {
    //     todo!()
    // } else {
    //     query.property_settings(query_match.pattern_index)
    // };
    // for prop in property_settings {
    //     match prop.key.as_ref() {
    //         // In addition to specifying the language name via the text of a
    //         // captured node, it can also be hard-coded via a `#set!` predicate
    //         // that sets the injection.language key.
    //         "injection.language" => {
    //             if language_name.is_none() {
    //                 language_name = prop.value.as_ref().map(|s| s.as_ref())
    //             }
    //         }

    //         // By default, injections do not include the *children* of an
    //         // `injection.content` node - only the ranges that belong to the
    //         // node itself. This can be changed using a `#set!` predicate that
    //         // sets the `injection.include-children` key.
    //         "injection.include-children" => include_children = true,
    //         _ => {}
    //     }
    // }
    let include_children = false;
    (language_name, content_node, include_children)
}

fn shrink_and_clear<T>(vec: &mut Vec<T>, capacity: usize) {
    if vec.len() > capacity {
        vec.truncate(capacity);
        vec.shrink_to_fit();
    }
    vec.clear();
}

pub fn html_escape(c: u8) -> Option<&'static [u8]> {
    match c as char {
        '>' => Some(b"&gt;"),
        '<' => Some(b"&lt;"),
        '&' => Some(b"&amp;"),
        '\'' => Some(b"&#39;"),
        '"' => Some(b"&quot;"),
        _ => None,
    }
}

const CANCELLATION_CHECK_INTERVAL: usize = 100;
const BUFFER_HTML_RESERVE_CAPACITY: usize = 10 * 1024;
const BUFFER_LINES_RESERVE_CAPACITY: usize = 1000;

/// Indicates which highlight should be applied to a region of source code.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Highlight(pub usize);

/// Represents the reason why syntax highlighting failed.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("Cancelled")]
    Cancelled,
    #[error("Invalid language")]
    InvalidLanguage,
    #[error("Unknown error")]
    Unknown,
}

/// Represents a single step in rendering a syntax-highlighted document.
#[derive(Copy, Clone, Debug)]
pub enum HighlightEvent {
    Source { start: usize, end: usize },
    HighlightStart(Highlight),
    HighlightEnd,
}

/// Contains the data needed to highlight code written in a particular language.
///
/// This struct is immutable and can be shared between threads.
pub struct HighlightConfiguration {
    pub language: Language,
    pub query: Query,
    combined_injections_query: Option<Query>,
    locals_pattern_index: usize,
    highlights_pattern_index: usize,
    highlight_indices: Vec<Option<Highlight>>,
    non_local_variable_patterns: Vec<bool>,
    injection_content_capture_index: Option<u32>,
    injection_language_capture_index: Option<u32>,
    local_scope_capture_index: Option<u32>,
    local_def_capture_index: Option<u32>,
    local_def_value_capture_index: Option<u32>,
    local_ref_capture_index: Option<u32>,
}

impl HighlightConfiguration {
    /// Creates a `HighlightConfiguration` for a given `Language` and set of highlighting
    /// queries.
    ///
    /// # Parameters
    ///
    /// * `language`  - The Tree-sitter `Language` that should be used for parsing.
    /// * `highlights_query` - A string containing tree patterns for syntax highlighting. This
    ///   should be non-empty, otherwise no syntax highlights will be added.
    /// * `injections_query` -  A string containing tree patterns for injecting other languages
    ///   into the document. This can be empty if no injections are desired.
    /// * `locals_query` - A string containing tree patterns for tracking local variable
    ///   definitions and references. This can be empty if local variable tracking is not needed.
    ///
    /// Returns a `HighlightConfiguration` that can then be used with the `highlight` method.
    pub fn new(
        language: Language,
        highlights_query: &str,
        injection_query: &str,
        locals_query: &str,
    ) -> Result<Self, QueryError> {
        // Concatenate the query strings, keeping track of the start offset of each section.
        let mut query_source = String::new();
        query_source.push_str(injection_query);
        let locals_query_offset = query_source.len();
        query_source.push_str(locals_query);
        let highlights_query_offset = query_source.len();
        query_source.push_str(highlights_query);

        eframe::web_sys::console::log_1(&"aa1".into());

        // Construct a single query by concatenating the three query strings, but record the
        // range of pattern indices that belong to each individual string.
        // let mut query = Query::new(&language, &query_source)?;
        let mut query: Query = language.query(&query_source.into())?.into();
        let mut locals_pattern_index = 0;
        let mut highlights_pattern_index = 0;
        eframe::web_sys::console::log_1(&"aa2".into());
        if injection_query.is_empty() && locals_query.is_empty() {
            // for i in 0..(query.pattern_count()) {
            // }
        } else {
            for i in 0..(query.pattern_count()) {
                let pattern_offset = query.start_byte_for_pattern(i);
                if pattern_offset < highlights_query_offset {
                    if pattern_offset < highlights_query_offset {
                        highlights_pattern_index += 1;
                    }
                    if pattern_offset < locals_query_offset {
                        locals_pattern_index += 1;
                    }
                }
            }
        }
        eframe::web_sys::console::log_1(&"aa3".into());

        // Construct a separate query just for dealing with the 'combined injections'.
        // Disable the combined injection patterns in the main query.
        let mut combined_injections_query: Query = language.query(&injection_query.into())?.into();
        let mut has_combined_queries = false;
        eframe::web_sys::console::log_1(&"aa4".into());
        for pattern_index in 0..locals_pattern_index {
            let settings = query.property_settings(pattern_index);
            if settings.iter().any(|s| &*s.key == "injection.combined") {
                has_combined_queries = true;
                query.disable_pattern(pattern_index);
            } else {
                combined_injections_query.disable_pattern(pattern_index);
            }
        }
        eframe::web_sys::console::log_1(&"aa5".into());
        let combined_injections_query = if has_combined_queries {
            Some(combined_injections_query)
        } else {
            None
        };

        // Find all of the highlighting patterns that are disabled for nodes that
        // have been identified as local variables.
        let non_local_variable_patterns = (0..query.pattern_count())
            .map(|i| {
                // query
                //     .property_predicates(i)
                //     .iter()
                //     .any(|(prop, positive)| !*positive && prop.key.as_str() == "local")
                true
            })
            .collect();

        eframe::web_sys::console::log_1(&"aa6".into());

        let capture_names = query.capture_names();
        // Store the numeric ids for all of the special captures.
        let mut injection_content_capture_index = None;
        let mut injection_language_capture_index = None;
        let mut local_def_capture_index = None;
        let mut local_def_value_capture_index = None;
        let mut local_ref_capture_index = None;
        let mut local_scope_capture_index = None;
        for (i, name) in capture_names.iter().enumerate() {
            let i = Some(i as u32);
            assert!(name.is_string());
            let name: String = name.as_string().unwrap();
            match name.as_str() {
                "injection.content" => injection_content_capture_index = i,
                "injection.language" => injection_language_capture_index = i,
                "local.definition" => local_def_capture_index = i,
                "local.definition-value" => local_def_value_capture_index = i,
                "local.reference" => local_ref_capture_index = i,
                "local.scope" => local_scope_capture_index = i,
                x => {
                    eframe::web_sys::console::log_1(&x.into());
                }
            }
        }
        let highlight_indices = vec![None; capture_names.len()];
        Ok(HighlightConfiguration {
            language,
            query,
            combined_injections_query,
            locals_pattern_index,
            highlights_pattern_index,
            highlight_indices,
            non_local_variable_patterns,
            injection_content_capture_index,
            injection_language_capture_index,
            local_def_capture_index,
            local_def_value_capture_index,
            local_ref_capture_index,
            local_scope_capture_index,
        })
    }

    /// Set the list of recognized highlight names.
    ///
    /// Tree-sitter syntax-highlighting queries specify highlights in the form of dot-separated
    /// highlight names like `punctuation.bracket` and `function.method.builtin`. Consumers of
    /// these queries can choose to recognize highlights with different levels of specificity.
    /// For example, the string `function.builtin` will match against `function.method.builtin`
    /// and `function.builtin.constructor`, but will not match `function.method`.
    ///
    /// When highlighting, results are returned as `Highlight` values, which contain the index
    /// of the matched highlight this list of highlight names.
    pub fn configure(&mut self, recognized_names: &[impl AsRef<str>]) {
        let mut capture_parts:Vec<String> = Vec::new();
        self.highlight_indices.clear();
        let capture_names = self.query.capture_names();
        self.highlight_indices
            .extend(capture_names.iter().map(move |capture_name| {
                capture_parts.clear();
                capture_parts.extend(capture_name.as_string().unwrap().split('.').map(|x|x.to_string()));

                let mut best_index = None;
                let mut best_match_len = 0;
                for (i, recognized_name) in recognized_names.into_iter().enumerate() {
                    let mut len = 0;
                    let mut matches = true;
                    for part in recognized_name.as_ref().split('.') {
                        len += 1;
                        if !capture_parts.contains(&part.to_string()) {
                            matches = false;
                            break;
                        }
                    }
                    if matches && len > best_match_len {
                        best_index = Some(i);
                        best_match_len = len;
                    }
                }
                best_index.map(Highlight)
            }));
    }
}
