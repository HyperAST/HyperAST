use hyperast::{position::position_accessors::SolvedPosition, store::defaults::NodeIdentifier};

type GithubUrl = String;

pub fn compute_formated_ranges(
    stores: &hyperast::store::SimpleStores<hyperast_vcs_git::TStore>,
    code: NodeIdentifier,
    query: &hyperast_tsquery::Query,
    repo: &str,
    oid: &str,
) -> Vec<Vec<GithubUrl>> {
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

        let position = &root.pos.make_file_line_range(root.stores);
        let value = format_pos_as_github_url(repo, oid, position);
        // dbg!(&value);
        result[i as usize].push(value);
        assert!(roots.next().is_none());
    }
    result
}

pub trait Pos {
    fn file(&self) -> &str;
    fn line_start(&self) -> usize;
    fn line_count(&self) -> usize;
}

impl Pos for &(String, usize, usize) {
    fn file(&self) -> &str {
        &self.0
    }

    fn line_start(&self) -> usize {
        self.1
    }

    fn line_count(&self) -> usize {
        self.2
    }
}

impl Pos for &PositionWithContext {
    fn file(&self) -> &str {
        &self.file
    }

    fn line_start(&self) -> usize {
        self.start
    }

    fn line_count(&self) -> usize {
        self.start
    }
}

pub type CommitId = str;

pub fn format_pos_as_github_url(repo: &str, oid: &CommitId, position: impl Pos) -> GithubUrl {
    let value = if position.line_count() == 0 {
        format!("{}#L{}", position.file(), position.line_start() + 1)
    } else {
        let end = position.line_start() + position.line_count();
        // NOTE the `+ 1` is a standard thing with editors starting at line one and not line zero
        format!(
            "{}#L{}-#L{}",
            position.file(),
            position.line_start() + 1,
            end + 1
        )
    };
    format!("https://github.com/{repo}/blob/{oid}/{value}")
}

pub fn format_pos_as_github_diff_url(repo: &str, oid: &CommitId, position: impl Pos) -> GithubUrl {
    use sha2::{Digest, Sha256};
    let data = position.file().as_bytes();
    let hash = Sha256::digest(data);

    // use base16ct::{Base16, Encoding};
    let mut buffer = vec![0;200];
    // let base16_hash = Base16::encode(&hash, &mut buffer).unwrap();
    // eprintln!("Base16-encoded hash: {}", base16_hash);

    let hash = base16ct::lower::encode_str(&hash, &mut buffer).unwrap();
    println!("Hex-encoded hash: {}", hash);

    // let hex_hash = base16ct::lower::encode_string(&hash);
    // println!("Hex-encoded hash: {}", hex_hash);

    let value = if position.line_count() == 0 {
        format!("L{}", position.line_start() + 1)
    } else {
        let end = position.line_start() + position.line_count();
        // NOTE the `+ 1` is a standard thing with editors starting at line one and not line zero
        format!(
            "L{}-L{}",
            position.line_start() + 1,
            end + 1
        )
    };
    let f = position.file();
    format!("{f} https://github.com/{repo}/commit/{oid}#diff-{hash}{value}")
}

pub fn compute_ranges(
    stores: &hyperast::store::SimpleStores<hyperast_vcs_git::TStore>,
    code: NodeIdentifier,
    query: &hyperast_tsquery::Query,
) -> Vec<Vec<(String, usize, usize)>> {
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

        let position = root.pos.make_file_line_range(root.stores);
        result[i as usize].push(position);
        assert!(roots.next().is_none());
    }
    result
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PositionWithContext {
    pub file: String,
    pub start: usize,
    pub end: usize,
    pub id: NodeIdentifier,
    pub pos: hyperast::position::StructuralPosition<NodeIdentifier, u16>, // pub test_method: Option<NodeIdentifier>,
                                                                           // pub test_class: Option<NodeIdentifier>,
                                                                           // pub blob: NodeIdentifier,
}

pub fn compute_postions_with_context(
    stores: &hyperast::store::SimpleStores<hyperast_vcs_git::TStore>,
    code: NodeIdentifier,
    query: &hyperast_tsquery::Query,
) -> Vec<Vec<PositionWithContext>> {
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

        let position = root.pos.make_file_line_range(root.stores);
        result[i as usize].push(PositionWithContext {
            file: position.0,
            start: position.1,
            end: position.2,
            id: root.pos.node(),
            pos: root.pos.clone(),
            // test_method: None,
            // test_class: None,
            // blob: todo!(),
        });
        assert!(roots.next().is_none());
    }
    result
}
