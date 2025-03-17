use crate::{ffi, CNLending};

use super::{indexed::Symbol, Cursor, Status, TreeCursorStep};

pub struct TreeCursor<'a> {
    text: &'a [u8],
    cursor: tree_sitter::TreeCursor<'a>,
}

impl<'a> TreeCursor<'a> {
    pub fn new(text: &'a [u8], cursor: tree_sitter::TreeCursor<'a>) -> Self {
        Self { text, cursor }
    }
}

impl<'a> crate::WithField for TreeCursor<'a> {
    type IdF = ffi::TSFieldId;
}

impl<'a, 'b> CNLending<'b> for TreeCursor<'a> {
    type NR = tree_sitter::Node<'b>;
}

impl<'a> Cursor for TreeCursor<'a> {
    type Node = tree_sitter::Node<'a>;
    // type NodeRef<'b> = tree_sitter::Node<'a> where Self: 'b;

    fn goto_next_sibling_internal(&mut self) -> TreeCursorStep {
        extern "C" {
            pub fn ts_tree_cursor_goto_next_sibling_internal(
                self_: *mut ffi::TSTreeCursor,
            ) -> TreeCursorStep;
        }
        unsafe {
            let s: *mut ffi::TSTreeCursor = std::mem::transmute(&mut self.cursor);
            ts_tree_cursor_goto_next_sibling_internal(s)
        }
    }

    fn goto_first_child_internal(&mut self) -> TreeCursorStep {
        extern "C" {
            pub fn ts_tree_cursor_goto_first_child_internal(
                self_: *mut ffi::TSTreeCursor,
            ) -> TreeCursorStep;
        }
        unsafe {
            let s: *mut ffi::TSTreeCursor = std::mem::transmute(&mut self.cursor);
            ts_tree_cursor_goto_first_child_internal(s)
        }
    }

    fn goto_parent(&mut self) -> bool {
        self.cursor.goto_parent()
    }

    fn current_node(&self) -> <Self as CNLending<'_>>::NR {
        self.cursor.node()
    }

    fn parent_is_error(&self) -> bool {
        extern "C" {
            pub fn ts_tree_cursor_parent_node(self_: *const ffi::TSTreeCursor) -> ffi::TSNode;
        }
        unsafe {
            let s: *const ffi::TSTreeCursor = std::mem::transmute(&self.cursor);
            let n = ts_tree_cursor_parent_node(s);
            if ffi::ts_node_is_null(n) {
                return false;
            }
            let n: tree_sitter::Node = std::mem::transmute(n);
            n.is_error()
        }
    }

    fn has_parent(&self) -> bool {
        self.cursor.node().parent().is_some()
    }

    fn persist(&mut self) -> Self::Node {
        self.cursor.node()
    }

    fn persist_parent(&mut self) -> Option<Self::Node> {
        self.cursor.node().parent()
    }

    type Status = TSStatus;

    #[inline]
    fn current_status(&self) -> TSStatus {
        extern "C" {
            pub fn ts_tree_cursor_current_status(
                self_: *const ffi::TSTreeCursor,
                field_id: *mut ffi::TSFieldId,
                has_later_siblings: *mut bool,
                has_later_named_siblings: *mut bool,
                can_have_later_siblings_with_this_field: *mut bool,
                supertypes: *mut ffi::TSSymbol,
                // unsigned *
                supertype_count: *mut std::os::raw::c_uint,
            ) -> TreeCursorStep;
        }
        unsafe {
            let s: *const ffi::TSTreeCursor = std::mem::transmute(&self.cursor);
            let mut field_id: ffi::TSFieldId = 0;
            let mut has_later_siblings: bool = false;
            let mut has_later_named_siblings: bool = false;
            let mut can_have_later_siblings_with_this_field: bool = false;
            let mut supertype_count: u32 = 8;
            // TODO mem perf: might not be efficient, I am surious about perfs impacts of this,
            // if ffi fct is inlined maybe the allocation can be optimized out,
            // but I believe it to be inprobable.
            // It would probably be possible to opacify Status and provide just the required meth to uses
            // NOTE in query cursor supertypes is used as a set, where it is asked if its empty and if it contains symbols
            let mut supertypes = Vec::<ffi::TSSymbol>::with_capacity(supertype_count as usize);
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

    fn text_provider(&self) -> <Self::Node as super::TextLending<'_>>::TP {
        self.text
    }
}

pub struct TSStatus {
    pub has_later_siblings: bool,
    pub has_later_named_siblings: bool,
    pub can_have_later_siblings_with_this_field: bool,
    pub field_id: ffi::TSFieldId,
    pub supertypes: Vec<Symbol>,
}

impl Status for TSStatus {
    type IdF = ffi::TSFieldId;

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

impl<'a, 'b> super::TextLending<'a> for tree_sitter::Node<'b> {
    type TP = &'a [u8];
}

impl<'a> super::Node for tree_sitter::Node<'a> {
    type IdF = ffi::TSFieldId;
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

    fn has_child_with_field_id(&self, field_id: ffi::TSFieldId) -> bool {
        self.child_by_field_id(field_id).is_some()
    }

    fn equal(&self, other: &Self) -> bool {
        self.id() == other.id()
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering::*;
        let left = self;
        let right = other;
        if !left.equal(right) {
            let left_start = left.start_byte();
            let right_start = right.start_byte();
            if left_start < right_start {
                return Less;
            } else if left_start > right_start {
                return Greater;
            }
            let left_node_count = left.end_byte();
            let right_node_count = right.end_byte();
            if left_node_count > right_node_count {
                return Less;
            } else if left_node_count < right_node_count {
                return Greater;
            }
        }
        Equal
    }

    fn text<'s, 'l>(&'s self, text_provider: <Self as super::TextLending<'l>>::TP) -> super::BB<'s, 'l, str> {
        // self.utf8_text(text_provider).unwrap().into()
        let r = std::str::from_utf8(&text_provider[self.start_byte()..self.end_byte()])
            .unwrap();
        super::BB::B(r)
    }

    // fn id(&self) -> usize {
    //     self.id()
    // }
}
