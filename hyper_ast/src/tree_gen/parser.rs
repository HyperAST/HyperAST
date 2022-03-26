



pub trait Node<'a> {
    fn kind(&self) -> &str;
    fn start_byte(&self) -> usize;
    fn end_byte(&self) -> usize;
    fn child_count(&self) -> usize;
    // fn child<S:Node<'a>>(&self, i: usize) -> Option<S>;
    fn child(&self, i: usize) -> Option<Self> where Self:Sized;
    fn is_named(&self) -> bool;
    
    fn extract_label(&self, text: &[u8]) -> Option<Vec<u8>> {
        let pos = self.start_byte();
        let end = self.end_byte();
        let label = {
            if self.child_count()>=1 {// TODO maybe get node role
                None
            } else if self.is_named() {
                let t = &text[pos..end];
                Some(t.to_vec())
            } else {
                None
            }
        };
        label
    }
}
pub trait TreeCursor<'a,N:Node<'a>> {
    fn node(&self) -> N;
    fn goto_first_child(&mut self) -> bool;
    fn goto_parent(&mut self) -> bool;
    fn goto_next_sibling(&mut self) -> bool;
}

