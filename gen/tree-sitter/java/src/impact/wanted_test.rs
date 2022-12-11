#[test]
fn test() {

    // per commit (from head ? from first (empty) ?)
    // - parse changed files in post-order
    //    index to later decide
    //          (there is a tradeoff btw same parent hierarchy (decompressed nodes) and subtree reuse, for caches)
    //          use path for indexing
    //    if subtree reach a certain threshold
    //      - compress + index
    //          same ast used in same commit = clone (state)
    //          same ast used in different commits
    //            without a insert = clone (state)
    //            with an insert = duplication (change)
    //              TODO cache path + version if compressed ast is an insertion (not mapped to same parent (or none) in previous version)
    //          a clone can later be considered a move if original is later removed
    //      - compute other things:
    //        - bloom filter of remaining refs
    //        - partial type resolutions to remove:
    //          - resolved refs
    //          - no longer visible decls
    //              when to stop bubble up impact to parent with name resolutions
    //        - decide if knowing that it does not contain (reference, or complete reference) is ok,
    //            ie. try to keep a constant/low number of needed refs for each node
    //            shared nodes (not subtree) could benefit from sharing ext references between versions
    //
}

#[cfg(test)]
#[allow(unused)]
mod try_typed_store {
    use hyper_ast::types::Type;

    pub enum Element {
        Block(Box<Block>),
        Statement {},
    }

    pub struct Block {
        kind: Type,
        elements: [Element],
    }
}
