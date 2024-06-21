use crate::ffi;
const MAX_STEP_CAPTURE_COUNT: usize = 3;

#[repr(C)]
pub(crate) struct CaptureQuantifiers(Vec<u8>);

#[repr(C)]
pub(crate) struct QueryPattern {
    pub(crate) steps: super::utils::Slice,
    pub(crate) predicate_steps: super::utils::Slice,
    pub(crate) start_byte: u32,
    pub(crate) is_non_local: bool,
}

#[repr(C)]
pub(crate) struct TSPatternEntry {
    pub(crate) step_index: u16,
    pub(crate) pattern_index: u16,
    pub(crate) is_rooted: bool,
}

#[repr(C)]
#[derive(Clone)]
pub(crate) struct TSQueryStep {
    pub(crate) symbol: ffi::TSSymbol,
    pub(crate) supertype_symbol: ffi::TSSymbol,
    pub(crate) field: ffi::TSFieldId,
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

impl std::fmt::Debug for TSQueryStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TSQueryStep")
            .field("symbol", &self.symbol)
            .field("supertype_symbol", &self.supertype_symbol)
            .field("field", &self.field)
            .field("capture_ids", &self.capture_ids)
            .field("depth", &self.depth)
            .field("alternative_index", &self.alternative_index)
            .field("negated_field_list_id", &self.negated_field_list_id)
            .field("is_named", &self.is_named())
            .field("is_immediate", &self.is_immediate())
            .field("is_last_child", &self.is_last_child())
            .field("is_pass_through", &self.is_pass_through())
            .field("is_dead_end", &self.is_dead_end())
            .field("alternative_is_immediate", &self.alternative_is_immediate())
            .field("contains_captures", &self.contains_captures())
            .field("root_pattern_guaranteed", &self.root_pattern_guaranteed())
            .field(
                "parent_pattern_guaranteed",
                &self.parent_pattern_guaranteed(),
            )
            .finish()
    }
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

#[repr(C)]
pub(crate) struct TSStepOffset {
    pub(crate) byte_offset: u32,
    pub(crate) step_index: u16,
}

#[repr(C)]
pub struct TSQuery {
    pub(crate) captures: crate::utils::SymbolTable,
    pub(crate) predicate_values: crate::utils::SymbolTable,
    pub(crate) capture_quantifiers: super::utils::Array<CaptureQuantifiers>,
    pub(crate) steps: super::utils::Array<TSQueryStep>,
    pub(crate) pattern_map: super::utils::Array<TSPatternEntry>,
    pub(crate) predicate_steps: super::utils::Array<ffi::TSQueryPredicateStep>,
    pub(crate) patterns: super::utils::Array<QueryPattern>,
    pub(crate) step_offsets: super::utils::Array<TSStepOffset>,
    pub(crate) negated_fields: super::utils::Array<ffi::TSFieldId>,
    pub(crate) string_buffer: super::utils::Array<std::ffi::c_char>,
    pub(crate) repeat_symbols_with_rootless_patterns: super::utils::Array<ffi::TSSymbol>,
    pub(crate) language: *const ffi::TSLanguage,
    pub(crate) wildcard_root_pattern_count: u16,
}
