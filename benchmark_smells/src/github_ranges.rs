use hyper_ast::{
    store::defaults::NodeIdentifier,
};

type GithubUrl = String;

pub fn compute_formated_ranges(
    stores: &hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    code: NodeIdentifier,
    query: &hyper_ast_tsquery::Query,
    repo: &str, oid: &str, 
) -> Vec<Vec<GithubUrl>> {
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
        let value = format_pos_as_github_url(repo, oid, position);
        // dbg!(&value);
        result[i as usize].push(value);
        assert!(roots.next().is_none());
    }
    result
}

pub fn format_pos_as_github_url(repo: &str, oid: &str, position: &(String, usize, usize)) -> GithubUrl {
    let value = if position.2 == 0 {
        format!("{}#L{}", position.0, position.1 + 1)
    } else {
        let end = position.1 + position.2;
        // NOTE the `+ 1` is a standard thing with editors starting at line one and not line zero
        format!("{}#L{}-#L{}", position.0, position.1 + 1, end + 1)
    };
    format!("https://github.com/{repo}/blob/{oid}/{value}")
}

pub fn compute_ranges(
    stores: &hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    code: NodeIdentifier,
    query: &hyper_ast_tsquery::Query,
) -> Vec<Vec<(String, usize, usize)>> {
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

        let position = root.pos.make_file_line_range(root.stores);
        result[i as usize].push(position);
        assert!(roots.next().is_none());
    }
    result
}
