use super::print_query;
use super::FieldId;

use super::NONE;

use super::PATTERN_DONE_MARKER;

use super::TSQuery;

use super::MAX_STEP_CAPTURE_COUNT;

#[repr(C)]
pub(crate) struct TSQueryStep {
    pub(crate) symbol: tree_sitter::ffi::TSSymbol,
    pub(crate) supertype_symbol: tree_sitter::ffi::TSSymbol,
    pub(crate) field: tree_sitter::ffi::TSFieldId,
    pub(crate) capture_ids: [u16; MAX_STEP_CAPTURE_COUNT],
    pub(crate) depth: u16,
    pub(crate) alternative_index: u16,
    pub(crate) negated_field_list_id: u16,
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
        // (block
        //     .
        //     (statement) @left
        //     .
        //     (statement) @right
        //     .
        //   )
        //
        // 0: {symbol: block} 1000000 named, contains_captures
        // 1: {symbol: *}1000010 named, immediate, contains_captures
        // 2: {symbol: *}1000110 named, last_child,immediate, contains_captures
        // 3: {DONE}110000000 named, dead_end, contains_captures
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



pub(crate) fn print_query_step(query: &TSQuery, step: &TSQueryStep) {
    const WILDCARD_SYMBOL: u16 = 0;
    eprint!("{{");
    if step.depth == PATTERN_DONE_MARKER {
        eprint!("DONE");
    } else if step.is_dead_end() {
        eprint!("dead_end");
    } else if step.is_pass_through() {
        eprint!("pass_through");
    } else if step.symbol != WILDCARD_SYMBOL {
        if let Some(s) = symbol_name(query, step.symbol) {
            eprint!("symbol: {}", s)
        } else {
            eprint!("symbol: {}", step.symbol)
        }
    } else {
        eprint!("symbol: *");
    }
    if step.is_named() {
        eprint!(", named");
    }
    if step.is_immediate() {
        eprint!(", immediate");
    }
    if step.is_last_child() {
        eprint!(", last_child");
    }
    if step.alternative_is_immediate() {
        eprint!(", alternative_is_immediate");
    }
    if step.contains_captures() {
        eprint!(", contains_captures");
    }
    if step.root_pattern_guaranteed() {
        eprint!(", root_pattern_guaranteed");
    }
    if step.parent_pattern_guaranteed() {
        eprint!(", parent_pattern_guaranteed");
    }

    if step.field > 0 {
        if let Some(s) = field_name(query, step.field) {
            eprint!("symbol: {}", s)
        } else {
            eprint!("symbol: {}", step.symbol)
        }
    }
    if step.alternative_index != NONE {
        eprint!(", alternative: {}", step.alternative_index);
    }
    eprint!("}}");
    eprint!(" bitfield: {:b}", step.bit_field);
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
        (block)+ @_block) @method"#;
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

    print_query(query);

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
        assert_eq!(step.bit_field, 0b1000000);
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
