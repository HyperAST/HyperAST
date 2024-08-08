
use hyper_ast::store::defaults::NodeIdentifier;

type GithubUrl = Vec<String>;

pub fn compute_ranges(
    stores: &hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    code: NodeIdentifier,
    query: &hyper_ast_tsquery::Query,
) -> Vec<GithubUrl> {
    let pos = hyper_ast::position::StructuralPosition::new(code);
    let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(stores, pos);
    let qcursor = query.matches(cursor);
    let mut result = vec![vec![]; query.enabled_pattern_count()];
    let cid = query.capture_index_for_name("root").expect(r#"you should put a capture named "root" on the pattern you can to capture (can be something else that the root pattern btw)"#);
    for m in qcursor {
        let i = m.pattern_index;
        let i = query.enabled_pattern_index(i).unwrap();
        let mut roots = m.nodes_for_capture_index(cid);
        let root = roots.next().expect("a node captured by @root");
        let position = &root.pos.make_file_line_range(root.stores);
        let value = if position.2 == 0 {
            format!("{}#L{}", position.0, position.1)
        } else {
            let end = position.1 + position.2;
            format!("{}#L{}-#L{}", position.0, position.1, end)
        };
        result[i as usize].push(value);
        assert!(roots.next().is_none());
    }
    result
}
