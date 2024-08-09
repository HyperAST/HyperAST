use hyper_ast::{
    position::{position_accessors::WithPostOrderPath, row_col, PositionConverter},
    store::defaults::NodeIdentifier, types::{HyperAST, WithStats},
};

type GithubUrl = (usize, Vec<String>);

pub fn compute_ranges(
    stores: &hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    code: NodeIdentifier,
    query: &hyper_ast_tsquery::Query,
) -> Vec<GithubUrl> {
    let pos = hyper_ast::position::StructuralPosition::new(code);
    let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(stores, pos);
    let qcursor = query.matches(cursor);
    let mut result = vec![(0, vec![]); query.enabled_pattern_count()];
    let cid = query.capture_index_for_name("root").expect(r#"you should put a capture named "root" on the pattern you can to capture (can be something else that the root pattern btw)"#);
    for m in qcursor {
        let i = m.pattern_index;
        let i = query.enabled_pattern_index(i).unwrap();
        let mut roots = m.nodes_for_capture_index(cid);
        let root = roots.next().expect("a node captured by @root");
        // for (o, p) in root.pos.iter_offsets_and_parents() {
        //     let t = stores.resolve_type(&p);
        //     dbg!(t);
        //     let n = stores.node_store.resolve(p);
        //     dbg!(n.line_count());
        // }

        let position = &root.pos.make_file_line_range(root.stores);
        // let position: row_col::RowCol<usize> = PositionConverter::new(&root.pos)
        //     .with_stores(root.stores)
        //     .compute_pos_post_order::<_, row_col::RowCol<usize>, _>();
        // let position = (&_position.0, position.row(), 0);
        let value = if position.2 == 0 {
            format!("{}#L{}", position.0, position.1 + 1)
        } else {
            let end = position.1 + position.2;
            // NOTE the `+ 1` is a standard thing with editors starting at line one and not line zero
            format!("{}#L{}-#L{}", position.0, position.1 + 1, end + 1)
        };
        // dbg!(&value);
        result[i as usize].0  += 1;
        result[i as usize].1.push(value);
        assert!(roots.next().is_none());
    }
    result
}
