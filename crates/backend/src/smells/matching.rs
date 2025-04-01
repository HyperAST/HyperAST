use super::SearchResult;

use hyperast::store::defaults::NodeIdentifier;

pub(crate) fn matches_default<'a>(
    with_spaces_stores: &hyperast::store::SimpleStores<hyperast_vcs_git::TStore>,
    tr: NodeIdentifier,
    queries: impl Iterator<Item = &'a str>,
) -> Result<Vec<usize>, String> {
    let mut len = 0;
    let collect = queries
        .map(|x| {
            len += 1;
            format!("{}\n\n", x)
        })
        .collect::<String>();
    let qqq = hyperast_tsquery::Query::new(&collect, hyperast_gen_ts_java::language())
        .map_err(|e| e.to_string())?;
    if qqq.enabled_pattern_count() != len {
        dbg!(qqq.enabled_pattern_count(), len);
        let mut count = 0;
        let mut prev_b = 0;
        let a = collect.split("@_root");
        let mut b = qqq
            .get_each_pat_start_byte()
            .into_iter()
            .skip(1)
            .chain(vec![collect.len()].into_iter());
        for (i, a) in a.enumerate() {
            count += 1;
            let a = a.split("@_root").next().unwrap().trim();
            let Some(b) = b.next() else {
                eprintln!("++++{}++++\n{}\n=====++++========", i, a);
                break;
            };
            let bb = collect[prev_b..b].split("@_root").next().unwrap().trim();
            prev_b = b;
            if a == bb {
                continue;
            }
            eprintln!(
                "------{}-------\n{}\n=============\n{}\n-------------",
                i, a, bb
            );
        }
        for (i, b) in b.enumerate() {
            let bb = collect[prev_b..b].split("@_root").next().unwrap().trim();
            prev_b = b;
            eprintln!("////{}/////\n{}\n=====/////=======", i, bb);
        }
        dbg!(count);
        return Err("different number of patterns".to_string());
    }
    let qcursor = qqq.matches(hyperast_tsquery::hyperast_opt::TreeCursor::new(
        with_spaces_stores,
        hyperast::position::structural_pos::CursorWithPersistance::new(tr),
    ));
    let mut res = vec![0; len];
    for m in qcursor {
        let i = m.pattern_index;
        let i = qqq.enabled_pattern_index(i).unwrap();
        res[i as usize] += 1;
    }
    Ok(res)
}

pub(crate) fn matches_with_precomputeds<'a>(
    with_spaces_stores: &hyperast::store::SimpleStores<hyperast_vcs_git::TStore>,
    tr: NodeIdentifier,
    queries: impl Iterator<Item = &'a str>,
    precomputeds: impl hyperast_tsquery::ArrayStr,
) -> Result<Vec<usize>, String> {
    let mut len = 0;
    let (_, qqq) = hyperast_tsquery::Query::with_precomputed(
        &queries
            .map(|x| {
                len += 1;
                format!("{}\n", x)
            })
            .collect::<String>(),
        hyperast_gen_ts_java::language(),
        precomputeds,
    )
    .map_err(|e| e.to_string())?;
    if qqq.enabled_pattern_count() != len {
        dbg!(qqq.pattern_count(), len);
        return Err("different number of patterns".to_string());
    }
    let qcursor = qqq.matches(hyperast_tsquery::hyperast_cursor::TreeCursor::new(
        with_spaces_stores,
        hyperast::position::StructuralPosition::new(tr),
    ));
    let mut res = vec![0; len];
    for m in qcursor {
        let i = m.pattern_index;
        let i = qqq.enabled_pattern_index(i).unwrap();
        res[i as usize] += 1;
    }
    Ok(res)
}

pub(crate) fn matches_with_precomputed(
    with_spaces_stores: &hyperast::store::SimpleStores<hyperast_vcs_git::TStore>,
    tr: NodeIdentifier,
    result: &mut SearchResult,
) -> Result<(), String> {
    let precomputeds: &[_] = todo!();
    let (_, qqq) = hyperast_tsquery::Query::with_precomputed(
        &result.query,
        hyperast_gen_ts_java::language(),
        precomputeds,
    )
    .map_err(|e| e.to_string())?;
    if qqq.pattern_count() != 1 + precomputeds.len() {
        dbg!(qqq.pattern_count());
        return Err("different number of patterns".to_string());
    }
    let qcursor = qqq.matches(hyperast_tsquery::hyperast_cursor::TreeCursor::new(
        with_spaces_stores,
        hyperast::position::StructuralPosition::new(tr),
    ));
    for m in qcursor {
        let i = m.pattern_index;
        let i = qqq.enabled_pattern_index(i).unwrap();
        assert_eq!(i, 0);
        result.matches += 1;
    }
    Ok(())
}
