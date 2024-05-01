pub trait Node<'a> {
    fn kind(&self) -> &str;
    fn start_byte(&self) -> usize;
    fn end_byte(&self) -> usize;
    fn child_count(&self) -> usize;
    // fn child<S:Node<'a>>(&self, i: usize) -> Option<S>;
    fn child(&self, i: usize) -> Option<Self>
    where
        Self: Sized;
    fn is_named(&self) -> bool;

    fn extract_label(&self, text: &[u8]) -> Option<Vec<u8>> {
        let pos = self.start_byte();
        let end = self.end_byte();
        if self.has_label() {
            Some(text[pos..end].to_vec())
        } else {
            None
        }
    }
    fn has_label(&self) -> bool {
        if self.child_count() >= 1 {
            // TODO maybe get node role
            false
        } else if self.is_named() {
            true
        } else {
            false
        }
    }
}

pub trait NodeWithU16TypeId<'a>: Node<'a> {
    fn kind_id(&self) -> u16;
}

#[derive(PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
}

pub trait TreeCursor<'a, N: Node<'a>> {
    fn node(&self) -> N;
    fn role(&self) -> Option<std::num::NonZeroU16>;
    fn goto_parent(&mut self) -> bool;

    /// try to goto first child and return if it is visible
    /// NOTE should be overridden to process hidden nodes
    fn goto_first_child_extended(&mut self) -> Option<Visibility> {
        if self.goto_first_child() {
            Some(Visibility::Visible)
        } else {
            None
        }
    }
    /// try to goto next sibling and return if it is visible
    /// NOTE should be overridden to process hidden nodes
    fn goto_next_sibling_extended(&mut self) -> Option<Visibility> {
        if self.goto_next_sibling() {
            Some(Visibility::Visible)
        } else {
            None
        }
    }
    /// try to goto first child
    fn goto_first_child(&mut self) -> bool;

    /// try to goto next sibling
    fn goto_next_sibling(&mut self) -> bool;
}
