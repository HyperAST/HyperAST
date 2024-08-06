use std::time::Instant;

use hyper_ast::store::defaults::NodeIdentifier;


pub fn count_matches(
    stores: &hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    code: NodeIdentifier,
    query: &hyper_ast_tsquery::Query,
) -> Vec<usize> {
    let pos = hyper_ast::position::StructuralPosition::new(code);
    let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(stores, pos);
    let qcursor = query.matches(cursor);
    let now = Instant::now();
    let mut result = vec![0; query.enabled_pattern_count()];
    for m in qcursor {
        let i = m.pattern_index;
        let i = query.enabled_pattern_index(i).unwrap();
        result[i as usize] += 1;
    }
    let compute_time = now.elapsed().as_secs_f64();
    result
}
