use std::collections::HashSet;

use hyperast::{
    position::{self, compute_position_with_no_spaces, position_accessors::WithPreOrderOffsets},
    types::{
        Children, Childrn as _, DecompressedFrom as _, HyperAST as _, HyperType, LabelStore,
        Labeled, NodeStore, TypedNodeStore, WithChildren,
    },
    utils::memusage_linux,
};
use hyperast_benchmark_smells::{
    diffing,
    github_ranges::{format_pos_as_github_url, Pos, PositionWithContext},
    simple::count_matches,
    DATASET,
};
use hyperast_vcs_git::preprocessed::PreProcessedRepository;

use hyper_diff::{
    decompressed_tree_store::{
        lazy_post_order::LazyPostOrder, DecompressedWithParent, ShallowDecompressedTreeStore,
    },
    matchers::{mapping_store::MonoMappingStore, Decompressible},
};
#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

/// enables uses of [`hyperast::utils::memusage_linux()`]
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    use std::env;
    let args: Vec<String> = env::args().collect();
    log::warn!("args: {:?}", args);
    let repo_name = args
        .get(1)
        .expect("give an argument like openjdk/jdk or INRIA/spoon");
    let commit = args.get(2).map_or("", |x| x);
    let limit = args.get(3).map_or(2, |x| x.parse().expect("a number"));
    let query = args.get(4).map_or("", |x| x);

    // WARN not to be mutated is some places, here is fine, change it at will
    // NOTE there is a upper limit for the number of usable subqueries
    let subs = [
        // invocation of the method "fail", surrounding string with r# makes that we don't have to escape the '"' in the string
        r#"(method_invocation
        (identifier) (#EQ? "fail")
    )"#,
        // a try block with a catch clause (does not match if there is no catch clause present)
        r#"(try_statement
        (block)
        (catch_clause)
    )"#,
        "(class_declaration)",
        "(method_declaration)",
        // an "@Test" annotation without parameters
        r#"(marker_annotation 
        name: (identifier) (#EQ? "Test")
    )"#,
        "(constructor_declaration)",
    ]
    .as_slice();

    removed_tracking(repo_name, commit, limit, query, subs);
}

const INCREMENTAL_QUERIES: bool = true;
const CSV_FORMATING: bool = false;

fn removed_tracking(
    repo_name: &str,
    commit: &str,
    limit: usize,
    query: &str,
    precomputeds: impl hyperast_tsquery::ArrayStr,
) {
    let query = if INCREMENTAL_QUERIES {
        hyperast_tsquery::Query::with_precomputed(
            &query,
            hyperast_gen_ts_java::language(),
            precomputeds,
        )
        .map_err(|x| x.to_string())
        .unwrap()
        .1
    } else {
        hyperast_tsquery::Query::new(&query, hyperast_gen_ts_java::language()).unwrap()
    };

    assert!(query.enabled_pattern_count() > 0);

    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    let oids = preprocessed.pre_process_first_parents_with_limit(
        &mut hyperast_vcs_git::git::fetch_github_repository(&preprocessed.name),
        "",
        commit,
        "",
        limit,
    );
    eprintln!("computing matches of {oids:?}");

    let stores = &preprocessed.processor.main_stores;
    let nospace = &hyperast_vcs_git::no_space::as_nospaces2(stores);

    if CSV_FORMATING {
        println!("commit_sha, ast_size, memory_used, processing_time, matches_count");
    } else {
        println!("matches_links");
    }

    let mut prev_oid = None;
    let mut old_matches_count = vec![];
    let mut old_matches_positions: Vec<Vec<_>> =
        vec![Default::default(); query.enabled_pattern_count()];
    let mut no_change_commits = vec![];
    for oid in oids {
        let commit = preprocessed.commits.get_key_value(&oid).unwrap();
        let time = commit.1.processing_time();
        let tr = commit.1.ast_root;
        use hyperast::types::WithStats;
        let s = stores.node_store.resolve(tr).size();

        let matches = hyperast_benchmark_smells::github_ranges::compute_postions_with_context(
            stores, tr, &query,
        );
        let matches_count: Vec<usize> = matches.iter().map(|x| x.len()).collect();

        let matches_count_print = matches_count
            .iter()
            .map(|x| format!(",{}", x))
            .collect::<String>();

        let matches_positions: Vec<Vec<_>> = matches
            .iter()
            .map(|x| x.into_iter().cloned().collect())
            .collect();
        let matches_links: Vec<String> = matches
            .into_iter()
            .map(|x| {
                let x: String = x
                    .into_iter()
                    .map(|position| {
                        format!(
                            "{},",
                            format_pos_as_github_url(
                                repo_name,
                                &oid.to_string(),
                                &(position.file, position.start, position.end)
                            )
                        )
                    })
                    .collect();
                x
                // format!(",[{:?}]", x)
            })
            .collect();

        let mu = memusage_linux();
        // TODO
        if old_matches_count != matches_count {
            eprintln!(
                "following commits did not show a change: {:?}",
                no_change_commits
            );
            old_matches_count = matches_count.clone();

            // do the tracking at least in the fast case top_down/subtree matching
            let curr_comm = &preprocessed.commits.get(&oid).unwrap();

            let mut src_arena = Decompressible::<_, LazyPostOrder<_, u32>>::decompress(
                nospace,
                &curr_comm.ast_root,
            );

            if let Some(prev_oid) = prev_oid {
                let prev_comm = preprocessed.commits.get(&prev_oid).unwrap();
                let mut dst_arena = Decompressible::<_, LazyPostOrder<_, u32>>::decompress(
                    nospace,
                    &prev_comm.ast_root,
                );

                let mut mapper = diffing::top_down(nospace, &mut src_arena, &mut dst_arena);
                let root = mapper.mapping.src_arena.root();
                for (i, p) in matches_positions.iter().enumerate() {
                    // let mut remove_occ = vec![];
                    for p_ctx in p {
                        let mut p = p_ctx.pos.iter_offsets();
                        let (_, _, no_spaces_path_to_target) =
                            compute_position_with_no_spaces(curr_comm.ast_root, &mut p, stores);
                        let src = mapper
                            .src_arena
                            .child_decompressed(&root, no_spaces_path_to_target.into_iter());
                        if let Some(dst) = mapper.mapping.mappings.get_dst(&src) {
                            // what if they are mapped but not the same, showing ?
                        } else {
                            let formated =
                                format_pos_as_github_url(repo_name, &oid.to_string(), p_ctx);
                            eprintln!("curr: {}", formated);
                            let formated =
                                format_pos_as_github_url(repo_name, &prev_oid.to_string(), p_ctx);
                            eprintln!("prev: {}", formated);
                            let mut src_parents = mapper.src_arena.parents(src);
                            'aaa: while let Some(src_parent) = src_parents.next() {
                                let parent_id = mapper.src_arena.original(&src_parent);
                                let t = nospace.resolve_type(&parent_id);
                                if let Some(dst_parent) =
                                    mapper.mapping.mappings.get_dst(&src_parent)
                                {
                                    if t.is_directory() || t.is_file() {
                                        // maybe, but more work each time
                                    } else {
                                        let mut c = dst_parent;
                                        let id = mapper.dst_arena.original(&c);
                                        let n = &nospace.node_store.resolve(id);
                                        let len = n.line_count();
                                        let mut line_offset = 0;
                                        let mut file_path = vec![];
                                        let mut offsets = vec![];
                                        // TODO construct position
                                        let t = nospace.resolve_type(&id);
                                        let mut file_trigg = t.is_file() || t.is_directory();
                                        if file_trigg {
                                            let l = n.try_get_label().unwrap();
                                            file_path.push(nospace.label_store.resolve(l));
                                        }
                                        loop {
                                            if let Some(o) = mapper.dst_arena.position_in_parent(&c)
                                            {
                                                offsets.push(o);
                                                let pid = mapper.dst_arena.original(
                                                    &mapper.dst_arena.parent(&c).unwrap(),
                                                );
                                                let parent_n = &stores.node_store.resolve(pid);
                                                let t = nospace.resolve_type(&pid);
                                                if !file_trigg {
                                                    let mut curr_off = 0;
                                                    for child in
                                                        parent_n.children().unwrap().iter_children()
                                                    {
                                                        if curr_off > o {
                                                            break;
                                                        }
                                                        let n = &stores.node_store.resolve(child);
                                                        line_offset += n.line_count();
                                                        use hyperast::types::TypeStore;
                                                        if !stores.resolve_type(&child).is_spaces()
                                                        {
                                                            curr_off += 1;
                                                        }
                                                    }
                                                }
                                                file_trigg = t.is_file() || t.is_directory();
                                                if file_trigg {
                                                    let l = parent_n.try_get_label().unwrap();
                                                    file_path.push(stores.label_store.resolve(l));
                                                }
                                                c = mapper.dst_arena.parent(&c).unwrap();
                                            } else {
                                                offsets.reverse();
                                                file_path.reverse();
                                                dbg!(
                                                    &p_ctx.file,
                                                    p_ctx.line_start(),
                                                    p_ctx.line_count()
                                                );
                                                dbg!(file_path, line_offset, len);
                                                todo!();
                                                break 'aaa;
                                            }
                                        }
                                    }
                                } else if t.is_directory() || t.is_file() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            prev_oid = Some(oid);
            old_matches_positions = matches_positions;
            no_change_commits = vec![];
        } else {
            no_change_commits.push(oid.to_string());
        }
    }
    eprintln!(
        "\nFollowing commits did not show a change: {:?}",
        no_change_commits
    );

    eprintln!("TODO summary")
}

fn track_heuristic1_bis<T: std::hash::Hash + Eq>(
    mut old_a: HashSet<T>,
    old_b: HashSet<T>,
) -> (HashSet<T>, Vec<T>) {
    // (old.difference(new).collect(), new.difference(old).collect()) // NOTE same
    let mut new_b = Vec::new();
    for x in old_b.into_iter() {
        if old_a.contains(&x) {
            old_a.remove(&x);
            new_b.push(x); // no copying here
        }
    }
    (old_a, new_b)
}

fn track_heuristic2(
    mut old_a: HashSet<PositionWithContext>,
    old_b: Vec<PositionWithContext>,
) -> (HashSet<PositionWithContext>, Vec<PositionWithContext>) {
    // (old.difference(new).collect(), new.difference(old).collect()) // NOTE same
    let mut new_b = Vec::new();
    for b in old_b.into_iter() {
        if let Some(i) = old_a.iter().find(|a| a.id == b.id && &a.file == &b.file) {
            old_a.remove(&i.clone());
            new_b.push(b); // no copying here
        }
    }
    (old_a, new_b)
}

// !!! query is currently incorrect but it is running :)
#[test]
fn conditional_test_logic() {
    let repo_name = "INRIA/spoon";
    let commit = "7c7f094bb22a350fa64289a94880cc3e7231468f";
    let limit = 2;
    let query = r#"(if_statement consequence: (_ 
    (_ (method_invocation 
         name: (identifier) (#EQ? "assertEquals") 
  ))
    ))"#;
    removed_tracking(repo_name, commit, limit, query, [].as_slice());
    eprintln!("conditional_test_logic done!")
}

#[test]
fn assertion_roulette_dubbo() {
    let data = DATASET.iter().find(|x| x.0 == "dubbo").unwrap();
    let repo_name = data.1;
    eprintln!("{}:", repo_name);
    let commit = data.2;
    let limit = 2000;
    let query = hyperast_benchmark_smells::queries::assertion_roulette();
    eprint!("{}", query);
    let subs = [
        r#"(method_invocation
                name: (identifier) (#EQ? "assertThat")
            )"#,
        "(class_declaration)",
        "(method_declaration)",
        r#"(marker_annotation 
        name: (identifier) (#EQ? "Test")
    )"#,
    ]
    .as_slice();
    removed_tracking(repo_name, commit, limit, &query, subs);
}

#[test]
fn exception_handling() {
    let data = DATASET.iter().find(|x| x.0 == "dubbo").unwrap();
    let repo_name = data.1;
    eprintln!("{}:", repo_name);
    let commit = data.2;
    let limit = 200;
    let query = hyperast_benchmark_smells::queries::exception_handling();
    let query = format!("{} @root {} @root", query[0], query[1]);
    println!("{}:", repo_name);
    println!("{}", query);
    removed_tracking(repo_name, commit, limit, &query, [].as_slice());
}

#[test]
fn exception_handling_graphhopper() {
    let data = DATASET.iter().find(|x| x.0 == "graphhopper").unwrap();
    let repo_name = data.1;
    eprintln!("{}:", repo_name);
    let commit = data.2;
    let limit = 4000;
    let query = hyperast_benchmark_smells::queries::exception_handling();
    let query = format!("{} @root", query[0]);
    println!("{}:", repo_name);
    println!("{}", query);
    removed_tracking(repo_name, commit, limit, &query, [].as_slice());
}
