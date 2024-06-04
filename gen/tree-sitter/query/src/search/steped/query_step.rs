use super::FieldId;

use super::NONE;

use super::PATTERN_DONE_MARKER;

use super::MAX_STEP_CAPTURE_COUNT;

use super::Array;
use super::PatternEntry;

#[repr(C)]
pub struct TSQuery {
    captures: SymbolTable,
    predicate_values: SymbolTable,
    capture_quantifiers: Array<CaptureQuantifiers>,
    pub(super) steps: Array<TSQueryStep>,
    pub(super) pattern_map: Array<PatternEntry>,
    predicate_steps: Array<tree_sitter::ffi::TSQueryPredicateStep>,
    patterns: Array<QueryPattern>,
    step_offsets: Array<StepOffset>,
    pub(super) negated_fields: Array<tree_sitter::ffi::TSFieldId>,
    string_buffer: Array<std::ffi::c_char>,
    repeat_symbols_with_rootless_patterns: Array<tree_sitter::ffi::TSSymbol>,
    pub(super) language: *const tree_sitter::ffi::TSLanguage,
    pub(super) wildcard_root_pattern_count: u16,
}

#[repr(C)]
struct StepOffset {
    byte_offset: u32,
    step_index: u16,
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

impl TSQuery {
    pub(super) fn pattern_map_search(&self, needle: super::Symbol) -> Option<usize> {
        // dbg!(query_step::symbol_name(self, needle.0));
        let mut base_index = self.wildcard_root_pattern_count as usize;
        let mut size = self.pattern_map.len() - base_index;
        // dbg!(needle.to_usize(), base_index, size);
        if size == 0 {
            return None;
        }
        while size > 1 {
            let half_size = size / 2;
            let mid_index = base_index + half_size;
            let mid_symbol =
                self.steps[self.pattern_map[mid_index].step_index as usize].symbol as usize;
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

        let mut symbol =
            self.steps[self.pattern_map[base_index].step_index as usize].symbol as usize;
        // dbg!(symbol);
        // dbg!(query_step::symbol_name(self, symbol as u16));

        if needle.to_usize() > symbol {
            base_index += 1;
            if base_index < self.pattern_map.len() {
                symbol =
                    self.steps[self.pattern_map[base_index].step_index as usize].symbol as usize;
            }
        }

        if needle.to_usize() == symbol {
            // dbg!(base_index);
            Some(base_index)
        } else {
            None
        }
    }

    pub(super) fn step_is_fallible(&self, step_index: u16) -> bool {
        assert!(step_index as usize + 1 < self.steps.len());
        let step = &self.steps[step_index as usize];
        let next_step = &self.steps[step_index as usize + 1];
        return next_step.depth != PATTERN_DONE_MARKER
            && next_step.depth > step.depth
            && !next_step.parent_pattern_guaranteed();
    }

    pub(super) fn field_name(&self, field_id: FieldId) -> &str {
        super::query_step::field_name(self, field_id).unwrap_or("")
    }
    pub(super) fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
    pub(super) fn capture_count(&self) -> usize {
        self.captures.slices.len()
    }
}

#[repr(C)]
pub(crate) struct TSQueryStep {
    pub(crate) symbol: tree_sitter::ffi::TSSymbol,
    pub(crate) supertype_symbol: tree_sitter::ffi::TSSymbol,
    pub(crate) field: tree_sitter::ffi::TSFieldId,
    pub(crate) capture_ids: [u16; MAX_STEP_CAPTURE_COUNT],
    pub(crate) depth: u16,
    pub(crate) alternative_index: u16,
    pub(crate) negated_field_list_id: u16,
    /// bitfield corresponding to the 9 following flags
    /// NOTE cannot use one bit attrs in rust without using macro,
    /// and even then it cannot be accessed like an attribute
    pub(crate) bit_field: u16,
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

impl TSQueryStep {
    pub(crate) fn is_named(&self) -> bool {
        // (for_statement 
        //      init: (expression) @init 
        //      condition: (_) @condition 
        //      update: (_) @update 
        //      body: (_) @body) @stmt @__tsg__full_match
        // query steps:
        //   0: {symbol: for_statement, contains_captures} bitfield: 1000000,
        //   1: {symbol: expression/*, contains_captures, field: init} bitfield: 1000000,
        //   2: {symbol: *, named, contains_captures, field: condition} bitfield: 1000001,
        //   3: {symbol: *, named, contains_captures, field: update} bitfield: 1000001,
        //   4: {symbol: *, named, contains_captures, field: body} bitfield: 1000001,
        //   5: {DONE, root_pattern_guaranteed, parent_pattern_guaranteed} bitfield: 110000000,
        self.bit_field & 0b1 != 0
    }
    pub(crate) fn is_immediate(&self) -> bool {
        self.bit_field & 0b10 != 0
    }
    pub(crate) fn is_last_child(&self) -> bool {
        self.bit_field & 0b100 != 0
    }
    pub(crate) fn is_pass_through(&self) -> bool {
        self.bit_field & 0b1000 != 0
    }
    pub(crate) fn is_dead_end(&self) -> bool {
        self.bit_field & 0b10000 != 0
    }
    pub(crate) fn alternative_is_immediate(&self) -> bool {
        self.bit_field & 0b100000 != 0
    }
    pub(crate) fn contains_captures(&self) -> bool {
        self.bit_field & 0b1000000 != 0
    }
    pub(crate) fn root_pattern_guaranteed(&self) -> bool {
        self.bit_field & 0b10000000 != 0
    }
    pub(crate) fn parent_pattern_guaranteed(&self) -> bool {
        self.bit_field & 0b100000000 != 0
    }
}

pub(crate) fn print_query_step(
    query: &TSQuery,
    step: &TSQueryStep,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    const WILDCARD_SYMBOL: u16 = 0;
    write!(f, "{{")?;
    if step.depth == PATTERN_DONE_MARKER {
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

    if step.field > 0 {
        if let Some(s) = field_name(query, step.field) {
            write!(f, ", field: {}", s)?
        } else {
            write!(f, ", field: {}", step.field)?
        }
    }
    if step.alternative_index != NONE {
        write!(f, ", alternative: {}", step.alternative_index)?;
    }
    write!(f, "}}")?;
    // NOTE C is not always zerowing the 7 unused bits so lets mask them
    write!(f, " bitfield: {:b}", step.bit_field & 0b111111111)
}

pub(crate) fn symbol_name<'a>(
    query: &'a TSQuery,
    symbol: tree_sitter::ffi::TSSymbol,
) -> Option<&'a str> {
    let ptr = unsafe { tree_sitter::ffi::ts_language_symbol_name(query.language, symbol) };
    if !ptr.is_null() {
        Some(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap())
    } else {
        None
    }
}

pub(crate) fn field_name<'a>(query: &'a TSQuery, field: FieldId) -> Option<&'a str> {
    let ptr = unsafe { tree_sitter::ffi::ts_language_field_name_for_id(query.language, field) };
    if !ptr.is_null() {
        Some(unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str().unwrap())
    } else {
        None
    }
}

/// QueryStep is defined in tree_sitter with the bitset syntax,
/// i.e., the size of members is specified, e.g., a bool can take only one bit.
/// It is compiler (and architecture?) dependant so lets be cautious !
#[test]
fn check_querystep_bitset_regresion() {
    let language = tree_sitter_java::language();
    let source = r#"(method_declaration
        .
        (modifiers "static"?@is_static)?
        type: (_) @type
        name: (identifier) @name
        (block)+ @_block
        .) @method"#;
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
    if ptr.is_null() {
        panic!()
    };

    let query: *mut TSQuery = unsafe { std::mem::transmute(ptr) };
    let query = unsafe { query.as_ref().unwrap() };

    eprintln!("{}", query);

    // 0: {symbol: method_declaration, contains_captures} bitfield: 1000000,
    {
        let step = unsafe { query.steps.contents.add(0).as_ref().unwrap() };
        assert_eq!(symbol_name(query, step.symbol), Some("method_declaration"));
        assert!(step.contains_captures());
        assert_eq!(step.bit_field, 0b1000000);
    }
    // 1: {symbol: modifiers, immediate, contains_captures, alternative: 3} bitfield: 1000010,
    {
        let step = unsafe { query.steps.contents.add(1).as_ref().unwrap() };
        assert_eq!(symbol_name(query, step.symbol), Some("modifiers"));
        assert!(step.contains_captures());
        assert!(step.is_immediate());
        assert_eq!(step.bit_field, 0b1000010);
    }
    // 2: {symbol: static, contains_captures, alternative: 3} bitfield: 1000000,
    {
        let step = unsafe { query.steps.contents.add(2).as_ref().unwrap() };
        assert_eq!(symbol_name(query, step.symbol), Some("static"));
        assert!(step.contains_captures());
        assert_eq!(step.bit_field, 0b1000000);
        assert!(step.field == 0);
    }
    // 3: {symbol: *, named, contains_captures, parent_pattern_guaranteed, field: type} bitfield: 101000001,
    {
        let step = unsafe { query.steps.contents.add(3).as_ref().unwrap() };
        assert!(step.symbol == 0);
        assert!(step.is_named());
        assert!(step.contains_captures());
        assert!(step.parent_pattern_guaranteed());
        assert!(step.field > 0);
        assert_eq!(field_name(query, step.field), Some("type"));
        assert_eq!(step.bit_field, 0b101000001);
    }
    // 4: {symbol: identifier, contains_captures, parent_pattern_guaranteed, field: name} bitfield: 101000000,
    {
        let step = unsafe { query.steps.contents.add(4).as_ref().unwrap() };
        assert_eq!(symbol_name(query, step.symbol), Some("identifier"));
        assert!(step.contains_captures());
        assert!(step.parent_pattern_guaranteed());
        assert_eq!(field_name(query, step.field), Some("name"));
        assert_eq!(step.bit_field, 0b101000000);
    }
    // 5: {symbol: block, contains_captures} bitfield: 1000000,
    {
        let step = unsafe { query.steps.contents.add(5).as_ref().unwrap() };
        assert_eq!(symbol_name(query, step.symbol), Some("block"));
        assert!(step.contains_captures());
        assert!(step.is_last_child());
        assert_eq!(step.bit_field, 0b1000100);
    }
    // 6: {pass_through, alternative_is_immediate, root_pattern_guaranteed, parent_pattern_guaranteed, alternative: 5} bitfield: 110111110101000,
    {
        let step = unsafe { query.steps.contents.add(6).as_ref().unwrap() };
        assert!(step.is_pass_through());
        assert!(step.alternative_is_immediate());
        assert!(step.root_pattern_guaranteed());
        assert!(step.parent_pattern_guaranteed());
        // NOTE: the remaining bits are uninitialized and not zeroed
        assert_eq!(step.bit_field & 0b111111111, 0b110101000);
    }
    // 7: {DONE, root_pattern_guaranteed, parent_pattern_guaranteed} bitfield: 110000000,
    {
        let step = unsafe { query.steps.contents.add(7).as_ref().unwrap() };
        assert!(step.depth == PATTERN_DONE_MARKER);
        assert!(step.root_pattern_guaranteed());
        assert!(step.parent_pattern_guaranteed());
        assert_eq!(step.bit_field, 0b110000000);
    }
}
