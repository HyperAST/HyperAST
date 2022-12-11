#[derive(Debug)]
pub struct FullNode<Global, Local> {
    pub global: Global,
    pub local: Local,
}

// pub struct FullNode {
//     compressible_node: NodeIdentifier,
//     depth: usize,
//     position: usize,
//     height: u32,
//     size: u32,
//     hashs: SyntaxNodeHashs<u32>,
// }

impl<Global, Local> FullNode<Global, Local> {
    pub fn local(&self) -> &Local {
        &self.local
    }
}
