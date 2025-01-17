#[cfg(feature = "impact")]
pub mod direct_type_ref;
#[cfg(feature = "impact")]
#[cfg(test)]
pub mod extends_package_local;
pub mod obj_creation;

use crate::{git::fetch_github_repository, preprocessed::PreProcessedRepository};
use std::env;

use hyper_ast::utils::memusage;

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
    use hyper_ast_gen_ts_java::impact::{
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
    // let id = preprocessed.processor.object_map_make.get(&a).unwrap();
    // hyper_ast_gen_ts_cpp::legion::print_tree_syntax(&preprocessed.processor.main_stores.node_store, &preprocessed.processor.main_stores.label_store, &id.0);
}
