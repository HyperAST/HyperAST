

pub fn label_for_cursor(text: &[u8], node: &tree_sitter::Node) -> Option<Vec<u8>> {
    let pos = node.start_byte();
    let end = node.end_byte();
    let label = {
        if node.child(0).is_some() {
            None
        } else if node.is_named() {
            let t = &text[pos..end];
            Some(t.to_vec())
        } else {
            None
        }
    };
    label
}