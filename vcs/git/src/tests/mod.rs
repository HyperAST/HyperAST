#[cfg(feature = "impact")]
pub mod direct_type_ref;
#[cfg(feature = "impact")]
#[cfg(test)]
pub mod extends_package_local;
pub mod obj_creation;

use crate::{git::fetch_github_repository, preprocessed::PreProcessedRepository};
#[cfg(feature = "impact")]
use std::env;

use hyperast::store::labels::LabelStore;
#[cfg(feature = "impact")]
use hyperast::utils::memusage;

#[cfg(feature = "impact")]
#[test]
fn example_main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    let repo_name = &args
        .get(1)
        .expect("give an argument like openjdk/jdk or INRIA/spoon"); //"openjdk/jdk";//"INRIA/spoon";
    let before = &args.get(2).map_or("", |x| x);
    let after = &args.get(3).map_or("", |x| x);
    let dir_path = &args.get(4).map_or("", |x| x);

    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    preprocessed.pre_process(
        &mut fetch_github_repository(&repo_name),
        before,
        after,
        dir_path,
    );

    find_refs_from_canonical_type(&mut preprocessed, before, after, dir_path);
}

#[cfg(feature = "impact")]
pub fn find_refs_from_canonical_type(
    preprocessed: &mut PreProcessedRepository,
    _before: &str,
    _after: &str,
    _dir_path: &str,
) {
    use hyperast_gen_ts_java::impact::{
        element::{IdentifierFormat, LabelPtr, RefsEnum},
        partial_analysis::PartialAnalysis,
    };
    {
        let mut ana = PartialAnalysis::default(); //&mut commits[0].meta_data.0;

        macro_rules! scoped {
            ( $o:expr, $i:expr ) => {{
                let o = $o;
                let i = $i;
                let f = IdentifierFormat::from(i);
                let i = preprocessed.processor.get_or_insert_label(i);
                let i = LabelPtr::new(i, f);
                ana.solver.intern(RefsEnum::ScopedIdentifier(o, i))
            }};
        }
        // let i = ana.solver.intern(RefsEnum::MaybeMissing);
        // // let i = scoped!(i, "ReferenceQueue");
        // let i = scoped!(i, "Reference");
        let i = ana.solver.intern(RefsEnum::Root);
        // let i = scoped!(scoped!(scoped!(i, "java"), "security"), "PrivilegedAction");
        // let i = scoped!(scoped!(scoped!(i, "java"), "util"), "Objects");
        // let i = scoped!(scoped!(scoped!(i, "java"), "util"), "Comparator");
        // let i = scoped!(scoped!(scoped!(i, "java"), "util"), "Arrays");
        // let i = scoped!(scoped!(scoped!(scoped!(i,"jdk"),"internal"),"misc"),"SharedSecrets");
        // let i = scoped!(scoped!(scoped!(scoped!(i,"java"),"util"),"concurrent"),"ThreadFactory");
        // let i = scoped!(scoped!(scoped!(scoped!(i,"java"),"nio"),"file"),"FilePermission");
        // let i = scoped!(scoped!(scoped!(scoped!(i, "java"), "nio"), "file"), "Files");
        let i = scoped!(
            scoped!(scoped!(scoped!(i, "java"), "nio"), "file"),
            "InvalidPathException"
        );
        let _ = i;
        // let i = scoped!(scoped!(scoped!(scoped!(i,"java"),"nio"),"file"),"Path");
        preprocessed.processor.print_refs(&ana);

        // println!("{}", java_tree_gen.stores.label_store);

        // let repository = fetch_github_repository(preprocessed.name());
        // let root = preprocessed
        //     .commits
        //     .get(&repository.refname_to_id(before).unwrap())
        //     .unwrap()
        //     .ast_root;
        // preprocessed.print_matched_references(&mut ana, i, root);
    }

    let mu = memusage();
    // drop(java_tree_gen);
    // drop(full_nodes);
    // drop(commits);
    drop(preprocessed);
    let mu = mu - memusage();
    println!("memory used {}", mu);
}

#[test]
fn example_process_make_cpp_project() {
    use std::io::Write;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .format(|buf, record| {
            if record.level().to_level_filter() > log::LevelFilter::Debug {
                writeln!(buf, "{}", record.args())
            } else {
                writeln!(
                    buf,
                    "[{} {}] {}",
                    buf.timestamp_millis(),
                    record.level(),
                    record.args()
                )
            }
        })
        .init();
    let name = &"official-stockfish/Stockfish";
    let mut preprocessed = PreProcessedRepository::new(name);
    let a = preprocessed.pre_process_make_project_with_limit(
        &mut fetch_github_repository(name),
        "",
        // "587bc647d7d14b53d8625c4446006e23a4acd82a",
        "f97c5b6909d22277f28e3dea2f146e9314d634dc", // issue with operator[]('K') = KB;
        "src",
        2,
    );

    let id = preprocessed.commits.get(&a[0]).unwrap().ast_root;
    eprintln!(
        "{}",
        hyperast::nodes::SyntaxSerializer::new(&preprocessed.processor.main_stores, id)
    );
}

#[test]
fn test_tsg_incr_inner_classes() -> std::result::Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .format(|buf, record| {
            use std::io::Write;
            if record.level().to_level_filter() > log::LevelFilter::Debug {
                writeln!(buf, "{}", record.args())
            } else {
                writeln!(
                    buf,
                    "[{} {}] {}",
                    buf.timestamp_millis(),
                    record.level(),
                    record.args()
                )
            }
        })
        .init();
    // let repo_spec = hyperast_vcs_git::git::Forge::Github.repo("INRIA", "spoon");
    // let config = hyperast_vcs_git::processing::RepoConfig::JavaMaven;
    // let commit = "56e12a0c0e0e69ea70863011b4f4ca3305e0542b";
    // let language = "Java";
    let tsg = r#"
(class_declaration name:(_)@name)@class {
    node @class.decl
    attr (@class.decl) name = (source-text @name)
}
"#;
    use hyperast_gen_ts_java::legion_with_refs::Acc;
    use hyperast_gen_ts_java::types::TStore;
    let tsg = {
        let tsg = tsg;
        type M<HAST, Acc> = hyperast_tsquery::QueryMatcher<HAST, Acc>;
        type ExtQ<HAST, Acc> =
            hyperast_tsquery::ExtendingStringQuery<M<HAST, Acc>, tree_sitter::Language>;

        let source: &str = tsg;
        let language = hyperast_gen_ts_java::language();

        let mut file = tree_sitter_graph::ast::File::<
            hyperast_tsquery::QueryMatcher<
                hyperast::store::SimpleStores<
                    TStore,
                    &hyperast::store::nodes::legion::NodeStoreInner,
                    &LabelStore,
                >,
                &Acc,
            >,
        >::new(language.clone());

        let query_source: ExtQ<
            hyperast::store::SimpleStores<
                TStore,
                &hyperast::store::nodes::legion::NodeStoreInner,
                &LabelStore,
            >,
            &Acc,
        > = {
            let x: &[&str] = &[];
            ExtQ::new(language.clone(), Box::new(x), source.len())
        };
        tree_sitter_graph::parser::Parser::<
            ExtQ<
                hyperast::store::SimpleStores<
                    TStore,
                    &hyperast::store::nodes::legion::NodeStoreInner,
                    &LabelStore,
                >,
                &Acc,
            >,
        >::with_ext(query_source, source)
        .parse_into_file(&mut file)
        .unwrap();
        use tree_sitter_graph::GenQuery;

        M::check(&mut file).unwrap();
        file
    };
    let _t = INNER_CLASSES;
    let _spec: &tree_sitter_graph::ast::File<hyperast_tsquery::QueryMatcher<_, &Acc>> = &tsg;
    let _query: Option<&hyperast_tsquery::Query> = None;
    // let functions = tree_sitter_graph::functions::Functions::<
    // tree_sitter_graph::graph::Graph<
    //     hyperast_tsquery::Node<
    //         hyperast::store::SimpleStores<
    //             TStore,
    //             &hyperast::store::nodes::legion::NodeStoreInner,
    //             &hyperast::store::labels::LabelStore,
    //         >,
    //         &Acc,
    //     >,
    // >,
    // // tree_sitter_graph::graph::GraphErazing<
    // //     hyperast_tsquery::MyNodeErazing<
    // //         hyperast::store::SimpleStores<
    // //             TStore,
    // //             &hyperast::store::nodes::legion::NodeStoreInner,
    // //             &hyperast::store::labels::LabelStore,
    // //         >,
    // //         &Acc,
    // //     >,
    // // >,
    // >::default();
    todo!();
    // let functions = functions.as_any();
    // let more = hyperast_tsquery::PreparedOverlay {
    //     query,
    //     overlayer: spec,
    //     functions,
    // };
    // let mut stores = hyperast::store::SimpleStores::default();
    // let mut md_cache = std::collections::HashMap::default();
    // let line_break = "\n".as_bytes().to_vec();
    // let mut java_tree_gen = hyperast_gen_ts_java::legion_with_refs::JavaTreeGen::<
    //     hyperast_gen_ts_java::types::TStore,
    //     _,
    //     _,
    // >::with_preprocessing(&mut stores, &mut md_cache, more)
    // .with_line_break(line_break);
    // let r = crate::java::handle_java_file(&mut java_tree_gen, &b"".into(), t.as_bytes()).unwrap();
    // log::error!("height : {:3?}", r.local.metrics.height);
    // log::error!("{:?}", stores.node_store);
    // // ASSERT one node per class_declaration
    // // TODO make an automatic test once nodes can be accessed after the contruction
    // Ok(())
}

static INNER_CLASSES: &str = r#"package spoon.test.imports.testclasses;

import spoon.test.imports.testclasses.internal.ChildClass;

public class ClientClass extends ChildClass {
	private class InnerClass {}
	private class InnerClass2 {}
	private class InnerClass3a {}
}
"#;
