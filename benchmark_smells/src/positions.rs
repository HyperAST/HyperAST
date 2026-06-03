
use hyperast::store::defaults::NodeIdentifier;

type Positions = Vec<String>;

pub fn output_positions(
    stores: &hyperast::store::SimpleStores<hyperast_vcs_git::TStore>,
    code: NodeIdentifier,
    query: &hyperast_tsquery::Query,
) -> Vec<Positions> {
    let pos = hyperast::position::StructuralPosition::new(code);
    let cursor = hyperast_tsquery::hyperast_cursor::TreeCursor::new(stores, pos);
    let qcursor = query.matches(cursor);
    let mut result = vec![vec![]; query.enabled_pattern_count()];
    let cid = query.capture_index_for_name("root").expect(r#"you should put a capture named "root" on the pattern you can to capture (can be something else that the root pattern btw)"#);
    for m in qcursor {
        let i = m.pattern_index;
        let i = query.enabled_pattern_index(i).unwrap();
        let mut roots = m.nodes_for_capture_index(cid);
        let root = roots.next().expect("a node captured by @root");
        let position = &root.pos.make_position(root.stores);
        let value = position.to_string();
        result[i as usize].push(value);
        assert!(roots.next().is_none());
    }
    result
}