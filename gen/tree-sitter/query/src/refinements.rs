use crate::auto::tsq_transform;
use crate::code2query::{self, QueryLattice};
use hashbrown::{HashMap, HashSet};
use hyperast::position::StructuralPosition;
use hyperast::position::structural_pos::AAA;

type IdN = hyperast::store::defaults::NodeIdentifier;
type Idx = u16;

/// the patterns along with the ones they originally were.
/// The union of original patterns should stay the same across refactorings
pub type PattSet = HashMap<IdN, HashSet<IdN>>;

pub fn try_pattern_select(
    lattice: &mut QueryLattice<&StructuralPosition<IdN, Idx>>,
    roots: &PattSet,
    simp_query: &str,
) -> Option<PattSet> {
    let language = crate::language();
    let meta_simp = match hyperast_tsquery::Query::new(simp_query, language) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return None;
        }
    };
    let Some(cid) = meta_simp.capture_index_for_name("target") else {
        eprintln!("Failed to find capture index for 'target'");
        return None;
    };
    let mut dedup = HashMap::<_, HashSet<IdN>>::default();
    for (patt_id, count) in roots.iter() {
        let set = code2query::find_matches_aux(&lattice.query_store, *patt_id, &meta_simp, cid);

        set.iter()
            .map(|p| p.node())
            .for_each(|x| dedup.entry(x).or_default().extend(count));
    }
    Some(dedup)
}

pub fn try_pattern_union(
    lattice: &mut QueryLattice<&StructuralPosition<IdN, Idx>>,
    roots: &PattSet,
    simp_query: &str,
) -> Option<PattSet> {
    let source = simp_query.trim();
    let language = crate::language();
    let meta_simp = match hyperast_tsquery::Query::new(source, language) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return None;
        }
    };
    type TopIdN = IdN;
    type CapIdN = IdN;
    type OriIdN = IdN;
    type SubEleIdN = IdN;
    let mut dedup_tops = HashMap::<
        TopIdN,
        (
            HashMap<CapIdN, HashMap<SubEleIdN, HashSet<OriIdN>>>,
            HashSet<OriIdN>,
        ),
    >::new();
    for (top, count) in roots {
        let mut union = HashMap::new();
        let mut unions = HashMap::new();
        let mut global_preds = HashMap::new();
        {
            let query_store = &lattice.query_store;
            let query = *top;
            let meta_simp: &hyperast_tsquery::Query = &meta_simp;
            let mut pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
            let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
            let mut matches = meta_simp.matches(cursor);
            loop {
                let Some(m) = matches.next() else {
                    break;
                };
                for c in m.captures.iter() {
                    let cap = meta_simp.capture_name(c.index);
                    if cap.starts_with("union.") {
                        let cap = cap.replace("union.", "placeholder."); // TODO just replace at start
                        let p = c.node.pos.node();
                        let s = lattice.pretty(&p);
                        log::debug!("=> {}\n{}", cap, s);
                        // let s =
                        //     hyperast::nodes::SexpSerializer::new(&lattice.query_store, p);
                        // log::debug!("{s}");
                        union.insert((c.node.pos.node(), c.node.pos.clone().offsets()), cap);
                    } else if cap.starts_with("unions.") {
                        let cap = cap.replace("unions.", "placeholders."); // TODO just replace at start
                        let p = c.node.pos.node();
                        let s = lattice.pretty(&p);
                        log::debug!("=> {}\n{}", cap, s);
                        // let s =
                        //     hyperast::nodes::SexpSerializer::new(&lattice.query_store, p);
                        // log::debug!("{s}");
                        union.insert((c.node.pos.node(), c.node.pos.clone().offsets()), cap);
                    } else if cap == "global_pred_union" {
                        let cap = "global_preds".to_string();
                        let p = c.node.pos.node();
                        let s = lattice.pretty(&p);
                        log::debug!("=> {}\n{}", cap, s);
                        // let s =
                        //     hyperast::nodes::SexpSerializer::new(&lattice.query_store, p);
                        // log::debug!("{s}");
                        union.insert((c.node.pos.node(), c.node.pos.clone().offsets()), cap);
                    }
                }
            }
        };
        let mut extracteds = HashMap::<IdN, HashMap<IdN, HashSet<IdN>>>::new();
        let simp_tops = {
            let query_store = &mut lattice.query_store;
            let mut values: Vec<(String, IdN, Vec<u16>)> = union
                .iter()
                .chain(unions.iter())
                .chain(global_preds.iter())
                .map(|(v, k)| (k.to_string(), v.clone()))
                .map(|(k, (id, mut path))| {
                    path.pop();
                    path.reverse();
                    (k, id, path)
                })
                .collect();
            values.sort_by(|a, b| a.2.cmp(&b.2));
            let mut actions = vec![];
            let mut disjoin_k_counts = HashMap::<String, usize>::new();
            let mut prev_e = code2query::make_cap(query_store, "shouldnotbethere");
            let mut prev_k = "".to_string();
            let mut prev_v: Vec<u16> = vec![];
            for (k, id, v) in values {
                let are_same_k = prev_k == k && k == "global_preds";
                let are_consecutive = prev_v.len() == v.len()
                    && prev_v[..v.len() - 1] == v[..v.len() - 1]
                    && prev_v[v.len() - 1] == v[v.len() - 1] - 1;
                dbg!(&prev_v);
                dbg!(&v);
                dbg!(are_consecutive);
                let e = if are_same_k && are_consecutive {
                    prev_e
                } else {
                    let count = disjoin_k_counts.entry(k.clone()).or_insert(0);
                    *count += 1;
                    let e = code2query::make_cap(query_store, &format!("{k}.{count}"));
                    prev_k = k.clone();
                    prev_e = e;
                    e
                };
                let path = v.clone();
                let new = e;
                use tsq_transform::Action;
                if are_same_k && are_consecutive {
                    actions.push(Action::Delete { path });
                } else {
                    actions.push(Action::Replace { path, new });
                }
                extracteds
                    .entry(e)
                    .or_insert(HashMap::new())
                    .entry(id)
                    .or_insert(HashSet::new())
                    .insert(*top);
                prev_v = v;
            }
            if !actions.is_empty() {
                // actions.is_sorted_by(|a, b| a.path.cmp(&b.path));
                tsq_transform::regen_query(query_store, *top, actions)
            } else {
                None
            }
        };

        // if let Some(p) = &simp_tops {
        //     let p = lattice.pretty(p);
        //     dbg!();
        //     println!("{p}");
        // } else {
        //     dbg!();
        // }
        if let Some(simp_tops) = simp_tops {
            let val = dedup_tops
                .entry(simp_tops)
                .or_insert((HashMap::new(), Default::default()));
            val.1.extend(count);
            for (k, v) in extracteds {
                let val = val.0.entry(k).or_insert(Default::default());
                val.extend(v);
            }
        }
    }

    for (p, (subs, count)) in dedup_tops.iter() {
        println!(";;;;;detailed unions;;;;;");
        for (name, subs) in subs {
            let name = lattice.pretty(name);
            println!("=> {name}");
            for sub in subs {
                let p = lattice.pretty(sub.0);
                println!("{p}");
            }
        }
        let p = lattice.pretty(p);
        let count = count.len();
        println!(";;count: {count}");
        println!("{p}");
    }
    for (p, (subs, count)) in dedup_tops.iter() {
        println!(";;;;;;flattened unions;;;;;;");
        let p = lattice.pretty(p);
        let count = count.len();
        println!(";;count: {count}");
        let mut p = format!("{p}");
        for (name, subs) in subs {
            let name = lattice.pretty(name);
            let name = format!("{name}");
            let mut alternation = " [\n".to_string();
            for sub in subs {
                let p = lattice.pretty(sub.0);
                alternation.push_str(&format!("{p}\n"));
            }
            if name.starts_with("placeholders.") {
                alternation.push_str(&format!("]*{name}"));
            } else {
                alternation.push_str(&format!("]{name}"));
            }
            p = p.replace(&name, &alternation)
        }
        println!("{p}");
    }
    // Some(new_tops)
    Some(dedup_tops.into_iter().map(|(k, v)| (k, v.1)).collect())
}

pub fn try_pattern_removes(
    lattice: &mut QueryLattice<&StructuralPosition<IdN, Idx>>,
    roots: &PattSet,
    simp_query: &str,
) -> Option<PattSet> {
    let source = simp_query.trim();
    let language = crate::language();
    let meta_simp = match hyperast_tsquery::Query::new(source, language) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return None;
        }
    };
    let cid = meta_simp.capture_index_for_name("remove").unwrap();
    let mut new_tops = HashMap::<IdN, HashSet<IdN>>::new();
    for (root, count) in roots {
        dbg!();
        let rms: Vec<_> =
            code2query::find_matches_aux(&lattice.query_store, *root, &meta_simp, cid)
                .iter()
                .map(|p| p.offsets())
                .collect();

        let query_store = &mut lattice.query_store;
        let query = *root;
        let values: Vec<Vec<u16>> = rms;
        let mut actions: Vec<Vec<u16>> = values.into_iter().fold(vec![], |mut actions, x| {
            let mut path = x;
            path.pop();
            path.reverse();
            if let Some(prev) = actions.last() {
                if !path.starts_with(prev) {
                    actions.push(path);
                }
            }
            actions
        });
        let p = if actions.is_empty() {
            None
        } else {
            actions.sort_by(|a, b| a.cmp(&b));
            dbg!(&actions);
            let actions: Vec<_> = actions
                .into_iter()
                .map(|path| tsq_transform::Action::Delete { path })
                .collect();
            tsq_transform::regen_query(query_store, query, actions)
        };
        if let Some(p) = p {
            new_tops.entry(p).or_default().extend(count);
            let p = lattice.pretty(&p);
            println!("{p}");
        } else {
            new_tops.entry(*root).or_default().extend(count);
        }
    }
    Some(new_tops)
}

pub fn try_pattern_captures(
    lattice: &mut QueryLattice<&StructuralPosition<IdN, Idx>>,
    roots: &PattSet,
    simp_query: &str,
) -> Option<PattSet> {
    let source = simp_query.trim();
    let language = crate::language();
    let meta_simp = match hyperast_tsquery::Query::new(source, language) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return None;
        }
    };
    let mut new_tops = HashMap::<IdN, HashSet<IdN>>::new();
    for (top, count) in roots {
        dbg!();
        let caps = {
            let mut caps = HashMap::new();
            let query_store = &lattice.query_store;
            let query = *top;
            let meta_simp: &hyperast_tsquery::Query = &meta_simp;
            let mut pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
            let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
            let mut matches = meta_simp.matches(cursor);
            loop {
                let Some(m) = matches.next() else {
                    break;
                };
                dbg!();
                for c in m.captures.iter() {
                    let cap = meta_simp.capture_name(c.index);
                    if !cap.starts_with("capture.") {
                        continue;
                    }
                    let cap = cap.replace("capture.", ""); // TODO just replace at start
                    let p = c.node.pos.node();
                    let s = lattice.pretty(&p);
                    log::debug!("=> {}\n{}", cap, s);
                    let s = hyperast::nodes::SexpSerializer::new(&lattice.query_store, p);
                    log::debug!("{s}");
                    caps.insert(c.node.pos.clone().offsets(), cap);
                }
            }
            caps
        };
        let query_store = &mut lattice.query_store;
        let query = *top;
        let p = {
            let values: Vec<(String, Vec<u16>)> = caps.into_iter().map(|x| (x.1, x.0)).collect();
            let mut actions: Vec<_> = values
                .into_iter()
                // .filter(|l| l.len() == 2)
                .filter_map(|x| {
                    let new = if x.0.is_empty() {
                        unimplemented!()
                    } else {
                        code2query::make_cap(query_store, &x.0)
                    };
                    let mut path = x.1.clone();
                    path.pop();
                    path.reverse();
                    // dbg!(&path);
                    Some((path, new))
                })
                .collect();
            if !actions.is_empty() {
                actions.sort_by(|a, b| a.0.cmp(&b.0));
                let actions: Vec<_> = actions
                    .into_iter()
                    .map(|(path, new)| tsq_transform::Action::Insert { path, new })
                    .collect();
                tsq_transform::regen_query(query_store, query, actions)
            } else {
                None
            }
        };
        if let Some(p) = p {
            new_tops.entry(p).or_default().extend(count);
            let p = lattice.pretty(&p);
            println!("{p}");
        } else {
            new_tops.entry(*top).or_default().extend(count);
        }
    }
    Some(new_tops)
}

pub fn try_pattern_renames(
    lattice: &mut QueryLattice<&StructuralPosition<IdN, Idx>>,
    roots: &PattSet,
    simp_query: &str,
) -> Option<PattSet> {
    let meta_simp = simp_query.trim();
    let internal_query = "(capture) @capture";
    let language = crate::language();
    let internal_query = match hyperast_tsquery::Query::new(internal_query, language) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return None;
        }
    };
    let internal_cid = internal_query.capture_index_for_name("capture").unwrap();
    let language = crate::language();
    let meta_simp = match hyperast_tsquery::Query::new(meta_simp, language) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return None;
        }
    };
    let mut new_tops = HashMap::<IdN, HashSet<IdN>>::new();
    for (top, count) in roots {
        let caps = {
            let mut caps = HashMap::new();
            let query_store = &lattice.query_store;
            let query = *top;
            let meta_simp: &hyperast_tsquery::Query = &meta_simp;
            let mut pos = hyperast::position::structural_pos::CursorWithPersistance::new(query);
            let cursor = hyperast_tsquery::hyperast_opt::TreeCursor::new(query_store, pos);
            let mut matches = meta_simp.matches(cursor);
            let mut count = 0;
            loop {
                let Some(m) = matches.next() else {
                    break;
                };
                for c in m.captures.iter() {
                    let cap = meta_simp.capture_name(c.index);
                    if cap.starts_with("rename.") {
                        let cap = cap.replace("rename.", ""); // TODO just replace at start
                        let p = c.node.pos.node();
                        let s = lattice.pretty(&p);
                        log::debug!("=> {}\n{}", cap, s);
                        let s = hyperast::nodes::SexpSerializer::new(&lattice.query_store, p);
                        log::debug!("{s}");
                        caps.insert(p, cap);
                    }
                    if cap.starts_with("renameC.") {
                        count += 1;
                        let cap = cap.replace("renameC.", ""); // TODO just replace at start
                        let cap = format!("{cap}.{count}");
                        let p = c.node.pos.node();
                        let s = lattice.pretty(&p);
                        log::debug!("=> {}\n{}", cap, s);
                        let s = hyperast::nodes::SexpSerializer::new(&lattice.query_store, p);
                        log::debug!("{s}");
                        caps.insert(p, cap);
                    }
                }
            }
            caps
        };
        let set =
            code2query::find_matches_aux(&lattice.query_store, *top, &internal_query, internal_cid);
        let mut per_label = HashMap::new();
        for x in set.iter() {
            let p = x.node();
            if let Some(y) = caps.get(&p) {
                let s = lattice.pretty(&p);
                log::debug!("=> {y}: {}", s);
                let s = hyperast::nodes::SexpSerializer::new(&lattice.query_store, p);
                log::debug!("{s}");
                per_label
                    .entry(p)
                    .or_insert(vec![])
                    .push((y.clone(), x.offsets()));
            }
        }
        let per_label_values = per_label.values_mut().collect();
        let query_store = &mut lattice.query_store;
        let query = *top;
        let p = code2query::replace_preds_with_caps(query_store, query, per_label_values);
        if let Some(p) = p {
            new_tops.entry(p).or_default().extend(count);
            let p = lattice.pretty(&p);
            println!("{p}");
        } else {
            new_tops.entry(*top).or_default().extend(count);
        }
    }
    Some(new_tops)
}
