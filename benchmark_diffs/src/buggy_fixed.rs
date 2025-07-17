use crate::postprocess::{CompressedBfPostProcess, PathJsonPostProcess, SimpleJsonPostProcess};
use crate::preprocess::{JavaPreprocessFileSys, iter_dirs, parse_dir_pair, parse_string_pair};
use crate::{other_tools, tempfile};
use hyper_diff::actions::Actions;
use hyper_diff::algorithms;
use hyper_diff::algorithms::{DiffResult, RuntimeMeasurement};
use hyperast::store::{SimpleStores, labels::LabelStore, nodes::legion::NodeStore};
use std::{env, fs::File, io::Write, path::Path, time::Instant};

const DATASET_FORMAT: i32 = 1; // ok as of 33024da8de4c519bb1c1146b19d91d6cb4c81ea6
// TODO find when format of dataset changed

pub fn buggy_fixed_dataset_roots(root: &Path, dataset: impl ToString) -> [std::path::PathBuf; 2] {
    let datasets = root.parent().unwrap().join("datasets");
    assert!(
        datasets.exists(),
        "you should clone the gumtree dataset:\n`cd ..; git clone git@github.com:GumTreeDiff/datasets.git gt_datasets; cd gt_datasets; git checkout 33024da8de4c519bb1c1146b19d91d6cb4c81ea6`"
    );
    let data_root = datasets.join(dataset.to_string());
    assert!(
        data_root.exists(),
        "this dataset does not exist or was renamed"
    );
    let data_root = data_root.as_path();
    std::fs::read_dir(data_root).expect("should be a dir");
    let src;
    let dst;
    if DATASET_FORMAT == 1 {
        src = data_root.join("before");
        dst = data_root.join("after");
    } else {
        src = data_root.join("buggy");
        dst = data_root.join("fixed");
    }
    assert!(src.exists(), "probably using the wrong format");
    assert!(dst.exists(), "probably using the wrong format");
    [src, dst]
}

#[test]
fn test_simple_1() {
    let buggy = r#"class A{class C{}class B{{while(1){if(1){}else{}};}}}class D{class E{}class F{{while(2){if(2){}else{}};}}}"#;
    let fixed = r#"class A{class C{}}class B{{while(1){if(1){}else{}};}}class D{class E{}}class F{{while(2){if(2){}else{}};}}"#;
    // use hyperast_gen_ts_java::types::TStore;
    let mut stores = SimpleStores::default();
    let mut md_cache = Default::default();
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, buggy, fixed);

    let stores = stores.change_type_store::<hyperast_gen_ts_java::types::TStore>();

    println!(
        "{}",
        algorithms::gumtree::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        )
        .actions
        .unwrap()
        .len()
    )
}

#[test]
fn test_crash1() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
    let [buggy_path, fixed_path] =
        src_dst.map(|x| x.join("Cli/22/src_java_org_apache_commons_cli_PosixParser.java"));
    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(&fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: Default::default(),
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    print!("{:?} len={}: ", buggy_path, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &buggy, &fixed);
    let len = algorithms::gumtree::diff(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    )
    .actions
    .unwrap()
    .len();
    println!("{}", len);
}

#[test]
fn test_perf_mokito() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
    let [buggy_path, fixed_path] = src_dst
        .map(|x| x.join("Mockito/34/src_org_mockito_internal_invocation_InvocationMatcher.java"));
    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(&fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: Default::default(),
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    print!("{:?} len={}: ", buggy_path, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &fixed, &buggy);
    let res = algorithms::gumtree::diff(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );
    println!("{:#?}", res.summarize());
    println!("{}", res);
}

#[test]
fn test_dir_increasing_ed() {
    use hyperast::types::{Childrn, HyperAST, WithChildren};
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = gh_dataset_roots(root);
    let [buggy_path, fixed_path] =
        src_dst.map(|x| x.join("google-guava/a194b64f31a7453a9be7e4573b2d92afab3aa37e/"));
    let md_cache = Default::default();
    let stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: Default::default(),
        node_store: NodeStore::new(),
    };

    let mut java_gen = JavaPreprocessFileSys {
        main_stores: stores,
        java_md_cache: md_cache,
    };
    let (src_tr, dst_tr) = parse_dir_pair(&mut java_gen, &buggy_path, &fixed_path);

    let stores = java_gen.main_stores;

    let src_trs = stores.node_store().resolve(src_tr.compressed_node);
    let src_trs = src_trs.children().unwrap();
    let src_trs: Vec<_> = src_trs.iter_children().collect();
    let dst_trs = stores.node_store().resolve(dst_tr.compressed_node);
    let dst_trs = dst_trs.children().unwrap();
    let dst_trs: Vec<_> = dst_trs.iter_children().collect();
    for (src_tr, dst_tr) in src_trs.into_iter().zip(dst_trs.into_iter()) {
        // to make a diff while ignoring spaces
        // let stores = hyperast_vcs_git::no_space::as_nospaces2(&stores);
        let res = algorithms::gumtree::diff_100(&stores, &src_tr, &dst_tr);
        println!("{:#?}", res.summarize());
        println!("{}", res);
        println!("==============================================");
        println!("==============================================");
        println!("==============now only subtree================");
        println!("==============================================");
        let res2 = algorithms::gumtree::diff_subtree(&stores, &src_tr, &dst_tr);
        println!("{:#?}", res2.summarize());
        println!("{}", res2);
        assert!(
            res.actions.unwrap().len() < res2.actions.unwrap().len(),
            "there are more matches so we expect less actions"
        );
    }
}

#[test]
fn test_increasing_ed() {
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = gh_dataset_roots(root);
    let [buggy_path, fixed_path] = src_dst.map(|x| {
        x.join(
            "google-guava/a194b64f31a7453a9be7e4573b2d92afab3aa37e/AbstractListenableFuture.java",
        )
    });
    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(&fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: Default::default(),
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    print!("{:?} len={}: ", buggy_path, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &fixed, &buggy);
    let stores = hyperast_vcs_git::no_space::as_nospaces2(&stores);
    let res = algorithms::gumtree::diff_100(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );
    println!("{:#?}", res.summarize());
    println!("{}", res);
    println!("==============================================");
    println!("====== now only with subtree matches =========");
    println!("==============================================");
    let res2 = algorithms::gumtree::diff_subtree(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );
    println!("{:#?}", res2.summarize());
    println!("{}", res2);
    assert!(
        res.actions.unwrap().len() < res2.actions.unwrap().len(),
        "there are more matches so we expect less actions"
    );
}

pub fn gh_dataset_roots(root: &Path) -> [std::path::PathBuf; 2] {
    let datasets = root.parent().unwrap().join("gt_datasets");
    assert!(
        datasets.exists(),
        "you should clone the gumtree dataset:\n`cd ..; git clone git@github.com:GumTreeDiff/datasets.git gt_datasets; cd gt_datasets; git checkout 33024da8de4c519bb1c1146b19d91d6cb4c81ea6`"
    );
    let data_root = datasets.join("gh-java");
    assert!(
        data_root.exists(),
        "this dataset does not exist or was renamed"
    );
    let data_root = data_root.as_path();
    std::fs::read_dir(data_root).expect("should be a dir");
    let src;
    let dst;
    if DATASET_FORMAT == 1 {
        src = data_root.join("before");
        dst = data_root.join("after");
    } else {
        src = data_root.join("buggy");
        dst = data_root.join("fixed");
    }
    assert!(src.exists(), "probably using the wrong format");
    assert!(dst.exists(), "probably using the wrong format");
    [src, dst]
}

#[cfg(test)]
mod examples {

    use hyperast::nodes::JsonSerializer2 as JsonSerializer;

    use crate::diff_output;

    use super::*;

    #[test]
    fn test_crash1_1() {
        // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
        println!("{:?}", std::env::current_dir());
        let buggy = CASE1;
        let fixed = CASE2;
        let mut stores = SimpleStores {
            label_store: LabelStore::new(),
            type_store: Default::default(),
            node_store: NodeStore::new(),
        };
        let mut md_cache = Default::default();
        // let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);
        print!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &buggy, &fixed);
        let len = algorithms::gumtree::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        )
        .actions
        .unwrap()
        .len();
        println!("{}", len);
    }

    static CASE1: &'static str = r#"class A {
        {
            if (1) {
            } else if (2) {
                h(42);
            } else if (3) {
                g(42);
            } else {
                h(42);
            }
        }
    }"#;

    static CASE2: &'static str = r#"class A {
        {
            } else {
                h(42, stopAtNonOption);
            }
        }
    }"#;

    #[test]
    fn test_crash1_2() {
        // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
        println!("{:?}", std::env::current_dir());
        let buggy = CASE3;
        let fixed = CASE4;
        let mut stores = SimpleStores {
            label_store: LabelStore::new(),
            type_store: Default::default(),
            node_store: NodeStore::new(),
        };
        let mut md_cache = Default::default();
        print!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &buggy, &fixed);
        let len = algorithms::gumtree::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        )
        .actions
        .unwrap()
        .len();
        println!("{}", len);
    }

    static CASE3: &'static str = r#"class A {
        {
            if (1) {
            } else if (2) {
                g(t);
            } else if (3) {
                if (4) {
                    p(t, s);
                } else {
                    b(t, s);
                }
            } else if (s) {
                h(t);
            } else {
                g(t);
            }
        }
    }"#;

    static CASE4: &'static str = r#"class A {
        {
            if (1) {
            } else if (2) {
                g(t);
            } else if (3) {
                if (4) {
                    p(t, s);
                } else {
                    b(t, s);
                }
            } else {dst_c
                h(t, s);
            }
        }
    }"#;

    static CASE5: &'static str = r#"class A {
        {
            type.narrowBy(dst);
        }
    }"#;
    pub static CASE6: &'static str = r#"class A {
        {
            config.getTypeFactory().constructSpecializedType(type, dst);
        }
    }"#;

    #[test]
    fn test_disagreement() {
        // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
        println!("{:?}", std::env::current_dir());
        let buggy = CASE5;
        let fixed = CASE6;

        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        // let guard = pprof::ProfilerGuardBuilder::default()
        //     .frequency(1000)
        //     .blocklist(&[
        //         // "libc",
        //         "libgcc", "pthread", "vdso",
        //     ])
        //     .build()
        //     .unwrap();

        let mut stores = SimpleStores {
            label_store: LabelStore::new(),
            type_store: Default::default(),
            node_store: NodeStore::new(),
        };
        let mut md_cache = Default::default();
        let now = Instant::now();

        println!("{} len={}", "buggy", buggy.len());
        let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &buggy, &fixed);
        let len = algorithms::gumtree::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        )
        .actions
        .unwrap()
        .len();
        let processing_time = now.elapsed().as_secs_f64();
        println!("tt={} evos={}", processing_time, len);
        // match guard.report().build() {
        //     Ok(report) => {
        //         let file = File::create("flamegraph.svg").unwrap();
        //         report.flamegraph(file).unwrap();
        //         // let mut file = File::create("profile.pb").unwrap();
        //         // let profile = report.pprof().unwrap();
        //         // use pprof::protos::Message;
        //         // let mut content = Vec::new();
        //         // profile.encode(&mut content).unwrap();
        //         // file.write_all(&content).unwrap();
        //     }
        //     Err(_) => {}
        // };
        let (src, mut src_f) = tempfile().unwrap();
        // dbg!(&src);

        src_f
            .write_all(
                (JsonSerializer::<_, _, true>::new(&stores, src_tr.local.compressed_node.clone())
                    .to_string())
                .as_bytes(),
            )
            .unwrap();
        drop(src_f);
        let (dst, mut dst_f) = tempfile().unwrap();
        // dbg!(&dst);
        dst_f
            .write_all(
                (JsonSerializer::<_, _, true>::new(&stores, dst_tr.local.compressed_node.clone())
                    .to_string())
                .as_bytes(),
            )
            .unwrap();
        drop(dst_f);
        let (json, mut json_f) = tempfile().unwrap();

        let now = Instant::now();
        std::process::Command::new("/bin/bash")
            .arg(root.join("gt_script.sh").to_str().unwrap())
            .arg(src)
            .arg(dst)
            .arg("gumtree")
            .arg(&"JSON")
            .arg("Chawathe")
            .arg(&json)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .expect("failed to execute process");

        let gt_processing_time = now.elapsed().as_secs_f64();

        json_f.flush().unwrap();
        drop(json_f);
        dbg!(&json);

        let o = serde_json::from_reader::<_, diff_output::F<diff_output::Tree>>(
            File::open(json).expect("should be a file"),
        )
        .unwrap();
        let gt_len: usize = o.actions.unwrap().len();
        dbg!(&o.times);
        println!("gt_tt={} gt_l={}", gt_processing_time, gt_len);
    }
}

#[cfg(test)]
mod test {
    use hyperast::{
        nodes::SyntaxWithIdsSerializer,
        store::SimpleStores,
        types::{DecompressedFrom, HyperASTShared, Typed},
    };

    use hyperast_gen_ts_xml::legion::tree_sitter_parse_xml as parse_xml;
    use hyperast_gen_ts_xml::{legion::XmlTreeGen, types::TStore};

    use hyper_diff::{
        decompressed_tree_store::lazy_post_order::LazyPostOrder,
        matchers::{
            Decompressible,
            Mapper,
            Mapping,
            heuristic::gt::lazy_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
            // heuristic::gt::greedy_subtree_matcher::{GreedySubtreeMatcher, SubtreeMatcher},
            mapping_store::{DefaultMultiMappingStore, VecStore},
        },
    };

    use crate::{
        other_tools::gumtree::subprocess,
        postprocess::{
            CompressedBfPostProcess, PathJsonPostProcess, SimpleJsonPostProcess, print_mappings,
            print_mappings_no_ranges,
        },
    };
    static CASE7: &'static str = r#"<project>
    <dependency>
        <groupId>org.mockito</groupId>
        <artifactId>mockito-core</artifactId>
        <version>4.3.0</version>
        <scope>test</scope>
    </dependency>
    <dependency>
        <groupId>org.junit.jupiter</groupId>
        <artifactId>junit-jupiter-engine</artifactId>
        <version>5.8.2</version>
        <scope>test</scope>
    </dependency>
    <dependency>
        <groupId>org.junit.jupiter</groupId>
        <artifactId>junit-jupiter-params</artifactId>
        <version>5.8.2</version>
        <scope>test</scope>
    </dependency>
    <dependency>
        <groupId>org.mockito</groupId>
        <artifactId>mockito-junit-jupiter</artifactId>
        <version>4.3.0</version>
        <scope>test</scope>
    </dependency>
</project>"#;
    pub static CASE8: &'static str = r#"<project>
    <dependency>
        <groupId>org.mockito</groupId>
        <artifactId>mockito-core</artifactId>
        <version>4.3.1</version>
        <scope>test</scope>
    </dependency>
    <dependency>
        <groupId>org.junit.jupiter</groupId>
        <artifactId>junit-jupiter-engine</artifactId>
        <version>5.8.2</version>
        <scope>test</scope>
    </dependency>
    <dependency>
        <groupId>org.junit.jupiter</groupId>
        <artifactId>junit-jupiter-params</artifactId>
        <version>5.8.2</version>
        <scope>test</scope>
    </dependency>
    <dependency>
        <groupId>org.mockito</groupId>
        <artifactId>mockito-junit-jupiter</artifactqId>
        <version>4.3.1</version>
        <scope>test</scope>
    </dependency>
</project>"#;

    #[test]
    fn test_spoon_pom_bad_subtree_match() {
        // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
        println!("{:?}", std::env::current_dir());
        let buggy = CASE7;
        let fixed = CASE8;
        let mut stores = SimpleStores::<TStore>::default();
        let mut tree_gen = XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
        };
        println!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = {
            let tree_gen = &mut tree_gen;
            let full_node1 = {
                let tree = match parse_xml(buggy.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), buggy.as_bytes(), tree.walk());
                full_node1
            };
            let full_node2 = {
                let tree = match parse_xml(fixed.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), fixed.as_bytes(), tree.walk());
                full_node1
            };
            (full_node1, full_node2)
        };
        let src = src_tr.local.compressed_node;
        // let src = tree_gen.stores.node_store.resolve(src).get_child(&0);
        let dst = dst_tr.local.compressed_node;
        // let dst = tree_gen.stores.node_store.resolve(dst).get_child(&0);

        let stores = hyperast_vcs_git::no_space::as_nospaces2(tree_gen.stores);

        // print_tree_syntax_with_ids(
        //     |id: &NodeIdentifier| -> _ {
        //         node_store
        //             .resolve(&id.clone())
        //             .into_compressed_node()
        //             .unwrap()
        //     },
        //     |id| -> _ { tree_gen.stores.label_store.resolve(id).to_owned() },
        //     &src,
        //     &mut Into::<IoOut<_>>::into(stdout()),
        // );
        // println!();
        // print_tree_syntax_with_ids(
        //     |id: &NodeIdentifier| -> _ {
        //         node_store
        //             .resolve(&id.clone())
        //             .into_compressed_node()
        //             .unwrap()
        //     },
        //     |id| -> _ { tree_gen.stores.label_store.resolve(id).to_owned() },
        //     &dst,
        //     &mut Into::<IoOut<_>>::into(stdout()),
        // );
        // println!();
        // let stores = &tree_gen.stores;
        let mappings = VecStore::default();

        #[allow(type_alias_bounds)]
        type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
        // type DS<IdN> = LazyPostOrder<IdN, u32>;

        let mut src_arena = DS::decompress(&stores, &src);
        let mut dst_arena = DS::decompress(&stores, &dst);
        let src_arena = src_arena.as_mut();
        let dst_arena = dst_arena.as_mut();
        let mapper = Mapper {
            hyperast: &stores,
            mapping: Mapping {
                src_arena: src_arena,
                dst_arena: dst_arena,
                mappings: mappings,
            },
        };
        let mapper =
            LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
        // >(&stores, &src, &dst, mappings);
        let mappings = mapper.mapping.mappings;
        let src_arena = mapper.mapping.src_arena;
        let dst_arena = mapper.mapping.dst_arena;
        // let SubtreeMatcher {
        //     src_arena,
        //     dst_arena,
        //     mappings,
        //     ..
        // } = mapper.into();
        // print_mappings(&dst_arena, &src_arena, node_store, label_store, &mappings);

        let gt_out = subprocess(
            &stores,
            src,
            dst,
            "gumtree-subtree",
            "Chawathe",
            60 * 5,
            "PATH",
        )
        .unwrap();

        let pp = PathJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let valid = pp._validity_mappings(&src_arena, src, &dst_arena, dst, &mappings);
        dbg!(valid.additional_mappings, valid.missing_mappings);
    }

    static CASE_SIMPLE: &'static str = r#"<project></project>"#;

    #[test]
    fn test_spoon_pom_bad_subtree_match_same_content() {
        // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
        println!("{:?}", std::env::current_dir());
        let buggy = CASE_SIMPLE;
        let fixed = CASE_SIMPLE;
        let mut stores = SimpleStores::<TStore>::default();
        let mut tree_gen = XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
        };
        println!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = {
            let tree_gen = &mut tree_gen;
            let full_node1 = {
                let tree = match parse_xml(buggy.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), buggy.as_bytes(), tree.walk());
                full_node1
            };
            let full_node2 = {
                let tree = match parse_xml(fixed.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), fixed.as_bytes(), tree.walk());
                full_node1
            };
            (full_node1, full_node2)
        };
        let src = src_tr.local.compressed_node;
        // let src = tree_gen.stores.node_store.resolve(src).get_child(&0);
        let dst = dst_tr.local.compressed_node;
        // let dst = tree_gen.stores.node_store.resolve(dst).get_child(&0);

        // let label_store = &tree_gen.stores.label_store;
        // let node_store = &tree_gen.stores.node_store;
        let stores = &*tree_gen.stores;
        // let node_store = &AAA {
        //     s: &tree_gen.stores.node_store,
        // };

        // use hyperast::types::LabelStore as _;
        // print_tree_syntax_with_ids(
        //     |id: &NodeIdentifier| -> _ {
        //         tree_gen
        //             .stores
        //             .node_store
        //             .resolve(id.clone())
        //             .into_compressed_node()
        //             .unwrap()
        //     },
        //     |id| -> _ { tree_gen.stores.label_store.resolve(id).to_owned() },
        //     &src,
        //     &mut Into::<IoOut<_>>::into(stdout()),
        // );
        // println!();
        // print_tree_syntax_with_ids(
        //     |id: &NodeIdentifier| -> _ {
        //         tree_gen
        //             .stores
        //             .node_store
        //             .resolve(id.clone())
        //             .into_compressed_node()
        //             .unwrap()
        //     },
        //     |id| -> _ { tree_gen.stores.label_store.resolve(id).to_owned() },
        //     &dst,
        //     &mut Into::<IoOut<_>>::into(stdout()),
        // );
        // println!();
        #[allow(type_alias_bounds)]
        type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
        let mappings = VecStore::default();
        let mut src_arena = DS::decompress(stores, &src);
        let mut dst_arena = DS::decompress(stores, &dst);
        let src_arena = src_arena.as_mut();
        let dst_arena = dst_arena.as_mut();
        let mapper = Mapper {
            hyperast: stores,
            mapping: Mapping {
                src_arena: src_arena,
                dst_arena: dst_arena,
                mappings: mappings,
            },
        };
        let mapper =
            LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
        let mappings = mapper.mapping.mappings;
        let src_arena = mapper.mapping.src_arena;
        let dst_arena = mapper.mapping.dst_arena;
        // print_mappings(
        //     &dst_arena,
        //     &src_arena,
        //     node_store,
        //     label_store,
        //     &mappings,
        // );

        let gt_out = subprocess(
            stores,
            src,
            dst,
            "gumtree-subtree",
            "Chawathe",
            60 * 5,
            "JSON",
        )
        .unwrap();

        let pp = SimpleJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let valid = pp._validity_mappings(stores, &src_arena, src, &dst_arena, dst, &mappings);
        dbg!(valid.additional_mappings, valid.missing_mappings);
    }

    #[test]
    fn test_spoon_pom_bad_subtree_match_same_content_compressed() {
        // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
        println!("{:?}", std::env::current_dir());
        let buggy = CASE_SIMPLE;
        let fixed = CASE_SIMPLE;
        let mut stores = SimpleStores::<TStore>::default();
        let mut tree_gen = XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
        };
        println!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = {
            let tree_gen = &mut tree_gen;
            let full_node1 = {
                let tree = match parse_xml(buggy.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), buggy.as_bytes(), tree.walk());
                full_node1
            };
            let full_node2 = {
                let tree = match parse_xml(fixed.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), fixed.as_bytes(), tree.walk());
                full_node1
            };
            (full_node1, full_node2)
        };
        let src = src_tr.local.compressed_node;
        use hyperast::types::HyperAST;
        dbg!(tree_gen.stores.resolve_type(&src));
        let dst = dst_tr.local.compressed_node;

        // let label_store = &tree_gen.stores.label_store;
        // let node_store = &tree_gen.stores.node_store;
        // let node_store = &NoSpaceNodeStoreWrapper::from(node_store);
        let stores = hyperast_vcs_git::no_space::as_nospaces2(tree_gen.stores);

        #[allow(type_alias_bounds)]
        type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
        let mappings = VecStore::default();
        let mut _src_arena = DS::decompress(&stores, &src);
        let mut _dst_arena = DS::decompress(&stores, &dst);
        let src_arena = _src_arena.as_mut();
        let dst_arena = _dst_arena.as_mut();
        let mapper = Mapper {
            hyperast: &stores,
            mapping: Mapping {
                src_arena: src_arena,
                dst_arena: dst_arena,
                mappings: mappings,
            },
        };
        let mapper =
            LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
        let mappings = mapper.mapping.mappings;
        // print_mappings(
        //     &dst_arena,
        //     &src_arena,
        //     node_store,
        //     label_store,
        //     &mappings,
        // );

        let src_arena = _src_arena.decomp.complete(&stores);
        let dst_arena = _dst_arena.decomp.complete(&stores);

        let gt_out = subprocess(
            &stores,
            src,
            dst,
            "gumtree-subtree",
            "Chawathe",
            60 * 5,
            "COMPRESSED",
        )
        .unwrap();

        let pp = CompressedBfPostProcess::create(&gt_out);
        let (pp, counts) = pp.counts();
        let (pp, gt_timings) = pp.performances();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let src_arena = Decompressible {
            hyperast: &stores,
            decomp: &src_arena,
        };
        let dst_arena = Decompressible {
            hyperast: &stores,
            decomp: &dst_arena,
        };
        let valid = pp._validity_mappings(&stores, &src_arena, src, &dst_arena, dst, &mappings);
        dbg!(valid.additional_mappings, valid.missing_mappings);
    }
    pub static CASE9: &'static str = r#"<project>
    <dependencies>
    </dependencies>
    <scm>
    </scm>
    <build>
        <plugins>
            <plugin>
                <version>3.10.1</version>
            </plugin>
            <plugin>
                <version>3.0.0</version>
            </plugin>
            <plugin>
                <version>3.3.0</version>
            </plugin>
        </plugins>
        <pluginManagement>
            <plugins>
                <!--This plugin's configuration is used to store Eclipse m2e settings
                  only. It has no influence on the Maven build itself. -->
                <plugin>
                </plugin>
                <plugin>
                </plugin>
            </plugins>
        </pluginManagement>
    </build>
    <plugins>
        <plugin>
            <version>2.9</version>
        </plugin>
        <plugin>
        </plugin>
    </plugins>
</project>
"#;

    static CASE10: &'static str = r#"<project>
    <dependencies>
    </dependencies>
    <scm>
    </scm>
    <build>
        <plugins>
            <plugin>
            </plugin>
            <plugin>
            </plugin>
        </plugins>
        <pluginManagement>
            <plugins>
                <plugin>
                    <version>3.1.0</version>
                </plugin>
                <plugin>
                </plugin>
                <plugin>
                    <artifactId>maven-clean-plugin</artifactId>
                    <version>3.2.0</version>
                </plugin>
                <plugin>
                </plugin>
                <plugin>
                    <version>3.3.0</version>
                </plugin>
                <plugin>
                </plugin>
                <plugin>
                    <artifactId>maven-install-plugin</artifactId>
                    <version>3.0.1</version>
                </plugin>
                <plugin>
                </plugin>
                <plugin>
                    <version>3.4.1</version>
                </plugin>
                <plugin>
                </plugin>
                <plugin>
                </plugin>
                <plugin>
                </plugin>
                <plugin>
                </plugin>
                <!--This plugin's configuration is used to store Eclipse m2e settings
                  only. It has no influence on the Maven build itself. -->
                <plugin>
                </plugin>
                <plugin>
                </plugin>
            </plugins>
        </pluginManagement>
    </build>
    <plugins>
        <plugin>
        </plugin>
        <plugin>
        </plugin>
    </plugins>
</project>
"#;

    #[test]
    fn test_spoon_pom_2() {
        // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
        println!("{:?}", std::env::current_dir());
        let buggy = CASE10;
        let fixed = CASE9;
        let mut stores = SimpleStores::<TStore>::default();
        let mut tree_gen = XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
        };
        println!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = {
            let full_node1 = {
                let tree = match parse_xml(buggy.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                tree_gen.generate_file("pom.xml".as_bytes(), buggy.as_bytes(), tree.walk())
            };
            let full_node2 = {
                let tree = match parse_xml(fixed.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                tree_gen.generate_file("pom.xml".as_bytes(), fixed.as_bytes(), tree.walk())
            };
            (full_node1, full_node2)
        };
        let src = src_tr.local.compressed_node;
        // let src = tree_gen.stores.node_store.resolve(src).get_child(&0);
        let dst = dst_tr.local.compressed_node;
        // let dst = tree_gen.stores.node_store.resolve(dst).get_child(&0);

        println!(
            "{}",
            SyntaxWithIdsSerializer::<_, _, true>::new(tree_gen.stores, src)
        );
        println!(
            "{}",
            SyntaxWithIdsSerializer::<_, _, true>::new(tree_gen.stores, dst)
        );

        #[allow(type_alias_bounds)]
        type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
        let mappings = VecStore::default();
        let mut _src_arena = DS::decompress(&stores, &src);
        let mut _dst_arena = DS::decompress(&stores, &dst);
        let src_arena = _src_arena.as_mut();
        let dst_arena = _dst_arena.as_mut();
        let mapper = Mapper {
            hyperast: &stores,
            mapping: Mapping {
                src_arena: src_arena,
                dst_arena: dst_arena,
                mappings: mappings,
            },
        };
        let mapper =
            LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
        let mappings = mapper.mapping.mappings;

        let src_arena = _src_arena.decomp.complete(&stores);
        let dst_arena = _dst_arena.decomp.complete(&stores);

        let src_arena = Decompressible {
            hyperast: &stores,
            decomp: &src_arena,
        };
        let dst_arena = Decompressible {
            hyperast: &stores,
            decomp: &dst_arena,
        };

        print_mappings(&dst_arena, &src_arena, &stores, &mappings);

        let gt_out = subprocess(
            &stores,
            src,
            dst,
            "gumtree-subtree",
            "Chawathe",
            60 * 5,
            "JSON",
        )
        .unwrap();

        let pp = SimpleJsonPostProcess::new(&gt_out);
        let counts = pp.counts();
        let gt_timings = pp.performances();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let valid = pp._validity_mappings(&stores, &src_arena, src, &dst_arena, dst, &mappings);
        dbg!(valid.additional_mappings, valid.missing_mappings);
    }

    pub static CASE11: &'static str = r#"<build>
    <plugins>
        <plugin>
            <version>3.3.0</version>
        </plugin>
    </plugins>
    <pluginManagement>
    </pluginManagement>
</build>
"#;

    static CASE12: &'static str = r#"<build>
    <plugins>
    </plugins>
    <pluginManagement>
            <plugin>
                <version>3.3.0</version>
            </plugin>
    </pluginManagement>
</build>
"#;

    #[test]
    fn test_spoon_pom_bad_subtree_match_no_spaces() {
        // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
        println!("{:?}", std::env::current_dir());
        let buggy = CASE12;
        let fixed = CASE11;
        let mut stores = SimpleStores::<TStore>::default();
        let mut tree_gen = XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
        };
        println!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = {
            let tree_gen = &mut tree_gen;
            let full_node1 = {
                let tree = match parse_xml(buggy.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), buggy.as_bytes(), tree.walk());
                full_node1
            };
            let full_node2 = {
                let tree = match parse_xml(fixed.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), fixed.as_bytes(), tree.walk());
                full_node1
            };
            (full_node1, full_node2)
        };
        let src = src_tr.local.compressed_node;
        // let src = tree_gen.stores.node_store.resolve(src).child(&0).unwrap();
        // let src = tree_gen.stores.node_store.resolve(src).child(&6).unwrap();
        // let src = tree_gen.stores.node_store.resolve(src).child(&4).unwrap();
        dbg!(tree_gen.stores.node_store.resolve(src).get_type());
        let dst = dst_tr.local.compressed_node;
        // let dst = tree_gen.stores.node_store.resolve(dst).child(&0).unwrap();
        // let dst = tree_gen.stores.node_store.resolve(dst).child(&6).unwrap();
        // let dst = tree_gen.stores.node_store.resolve(dst).child(&4).unwrap();

        // let label_store = &tree_gen.stores.label_store;
        // let node_store = &tree_gen.stores.node_store;
        // let node_store = &NoSpaceNodeStoreWrapper::from(node_store);
        let stores = hyperast_vcs_git::no_space::as_nospaces2(tree_gen.stores);
        let mappings = VecStore::default();
        #[allow(type_alias_bounds)]
        type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
        let mut _src_arena = DS::decompress(&stores, &src);
        let mut _dst_arena = DS::decompress(&stores, &dst);
        let src_arena = _src_arena.as_mut();
        let dst_arena = _dst_arena.as_mut();
        let mapper = Mapper {
            hyperast: &stores,
            mapping: Mapping {
                src_arena: src_arena,
                dst_arena: dst_arena,
                mappings: mappings,
            },
        };
        let mapper =
            LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
        let mappings = mapper.mapping.mappings;

        let src_arena = _src_arena.decomp.complete(&stores);
        let dst_arena = _dst_arena.decomp.complete(&stores);

        let src_arena = Decompressible {
            hyperast: &stores,
            decomp: &src_arena,
        };
        let dst_arena = Decompressible {
            hyperast: &stores,
            decomp: &dst_arena,
        };

        print_mappings_no_ranges(&dst_arena, &src_arena, &stores, &mappings);

        let gt_out = subprocess(
            &stores,
            src,
            dst,
            "gumtree-subtree",
            "Chawathe",
            60 * 5,
            "COMPRESSED",
        )
        .unwrap();

        let pp = CompressedBfPostProcess::create(&gt_out);
        let (pp, counts) = pp.counts();
        let (pp, gt_timings) = pp.performances();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let valid = pp._validity_mappings(&stores, &src_arena, src, &dst_arena, dst, &mappings);
        dbg!(valid.additional_mappings, valid.missing_mappings);
    }

    #[test]
    fn test_spoon_pom_bad_subtree_match_no_spaces_2() {
        // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
        println!("{:?}", std::env::current_dir());
        let buggy = CASE10;
        let fixed = CASE9;
        let mut stores = SimpleStores::<TStore>::default();
        let mut tree_gen = XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
        };
        println!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = {
            let tree_gen = &mut tree_gen;
            let full_node1 = {
                let tree = match parse_xml(buggy.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), buggy.as_bytes(), tree.walk());
                full_node1
            };
            let full_node2 = {
                let tree = match parse_xml(fixed.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), fixed.as_bytes(), tree.walk());
                full_node1
            };
            (full_node1, full_node2)
        };
        let src = src_tr.local.compressed_node;
        // let src = tree_gen.stores.node_store.resolve(src).child(&0).unwrap();
        // let src = tree_gen.stores.node_store.resolve(src).child(&6).unwrap();
        // let src = tree_gen.stores.node_store.resolve(src).child(&4).unwrap();
        dbg!(tree_gen.stores.node_store.resolve(src).get_type());
        let dst = dst_tr.local.compressed_node;
        // let dst = tree_gen.stores.node_store.resolve(dst).child(&0).unwrap();
        // let dst = tree_gen.stores.node_store.resolve(dst).child(&6).unwrap();
        // let dst = tree_gen.stores.node_store.resolve(dst).child(&4).unwrap();

        // let label_store = &tree_gen.stores.label_store;
        // let node_store = &tree_gen.stores.node_store;
        // let node_store = &NoSpaceNodeStoreWrapper::from(node_store);
        let stores = hyperast_vcs_git::no_space::as_nospaces2(tree_gen.stores);
        let mappings = VecStore::default();
        #[allow(type_alias_bounds)]
        type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;

        // let mapper = LazyGreedySubtreeMatcher::<DS<_>, DS<_>, _, _, _>::matchh(
        //     node_store, &src, &dst, mappings,
        // );
        let mut src_arena = DS::decompress(&stores, &src);
        let mut dst_arena = DS::decompress(&stores, &dst);
        let mapper = {
            let mappings = mappings;
            let src_arena = src_arena.as_mut();
            let dst_arena = dst_arena.as_mut();
            // src_arena.decompress_descendants(node_store, &src_arena.root());
            // dst_arena.decompress_descendants(node_store, &dst_arena.root());
            // src_arena.go_through_descendants(node_store, &src_arena.root());
            // dst_arena.go_through_descendants(node_store, &dst_arena.root());
            let mapper = Mapper {
                hyperast: &stores,
                mapping: Mapping {
                    src_arena: src_arena,
                    dst_arena: dst_arena,
                    mappings: mappings,
                },
            };
            let mapper = LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<
                DefaultMultiMappingStore<_>,
            >(mapper);
            // let mut matcher = LazyGreedySubtreeMatcher::<_, _, _, _, _, 1>::new(
            //     &stores, src_arena, dst_arena, mappings,
            // );
            // LazyGreedySubtreeMatcher::execute::<DefaultMultiMappingStore<_>>(&mut matcher);
            mapper
        };
        // let SubtreeMatcher {
        //     src_arena,
        //     dst_arena,
        //     mappings,
        //     ..
        // } = mapper.into();
        let mappings = mapper.mapping.mappings;
        let src_arena = src_arena.decomp.complete(&stores);
        let dst_arena = dst_arena.decomp.complete(&stores);
        let src_arena = Decompressible {
            hyperast: &stores,
            decomp: &src_arena,
        };
        let dst_arena = Decompressible {
            hyperast: &stores,
            decomp: &dst_arena,
        };
        print_mappings_no_ranges(&dst_arena, &src_arena, &stores, &mappings);

        let gt_out = subprocess(
            &stores,
            src,
            dst,
            "gumtree-subtree",
            "Chawathe",
            60 * 5,
            "COMPRESSED",
        )
        .unwrap();

        let pp = CompressedBfPostProcess::create(&gt_out);
        let (pp, counts) = pp.counts();
        let (pp, gt_timings) = pp.performances();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let valid = pp._validity_mappings(&stores, &src_arena, src, &dst_arena, dst, &mappings);
        dbg!(valid.additional_mappings, valid.missing_mappings);
    }
}

#[test]
fn compare_perfs() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    // let buggy_path = Path::new("../../gt_datasets/defects4j/buggy/JxPath/8/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationRelationalExpression.java");
    // let fixed_path = Path::new("../../gt_datasets/defects4j/fixed/JxPath/8/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationRelationalExpression.java");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dst = buggy_fixed_dataset_roots(root.parent().unwrap(), "defects4j");
    let [buggy_path, fixed_path] =
        src_dst.map(|x| x.join("Cli/22/src_java_org_apache_commons_cli_PosixParser.java"));

    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: Default::default(),
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let now = Instant::now();

    println!("{:?} len={}", buggy_path, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &buggy, &fixed);
    let len = algorithms::gumtree::diff(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    )
    .actions
    .unwrap()
    .len();
    let processing_time = now.elapsed().as_secs_f64();
    println!("tt={} evos={}", processing_time, len);

    let (src, _) = tempfile().unwrap();
    let (dst, _) = tempfile().unwrap();

    let now = Instant::now();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::process::Command::new("/bin/bash")
        .arg(root.join("gt_script.sh").to_str().unwrap())
        .arg(src)
        .arg(dst)
        .arg("gumtree")
        .arg("JSON")
        .arg("Chawathe")
        .arg("/dev/null")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .output()
        .expect("failed to execute process");

    let processing_time = now.elapsed().as_secs_f64();
    println!("gt_tt={}", processing_time);
}

#[test]
fn test_bad_perfs() {
    // bad_perfs()
    // bad_perfs2()
    bad_perfs3()
}

pub fn bad_perfs() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&[
            // "libc",
            "libgcc", "pthread", "vdso",
        ])
        .build()
        .unwrap();

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dst = buggy_fixed_dataset_roots(root.parent().unwrap(), "defects4j");
    let [buggy_path, fixed_path] = src_dst.map(|x| {
        x.join(
            "JacksonDatabind/31/src_main_java_com_fasterxml_jackson_databind_util_TokenBuffer.java",
        )
    });

    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: Default::default(),
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let now = Instant::now();

    println!("{:?} len={}", buggy_path, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &buggy, &fixed);
    let len = algorithms::gumtree::diff(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    )
    .actions
    .unwrap()
    .len();
    let processing_time = now.elapsed().as_secs_f64();
    println!("tt={} evos={}", processing_time, len);
    match guard.report().build() {
        Ok(report) => {
            let mut file = File::create("profile.pb").unwrap();
            let profile = report.pprof().unwrap();
            use pprof::protos::Message;
            let mut content = Vec::new();
            profile.encode(&mut content).unwrap();
            file.write_all(&content).unwrap();
        }
        Err(_) => {}
    };

    // ./dist/build/install/gumtree/bin/gumtree textdiff /home/quentin/rusted_gumtree3/benchmark_diffs/src/C1.java.json /home/quentin/rusted_gumtree3/benchmark_diffs/src/C2.java.json -m gumtree -g java-hyperast
}

pub fn bad_perfs2() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&[
            // "libc",
            "libgcc", "pthread", "vdso",
        ])
        .build()
        .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
    let [buggy_path, fixed_path] =
        src_dst.map(|x| x.join("Chart/4/source_org_jfree_chart_plot_XYPlot.java"));
    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: Default::default(),
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let now = Instant::now();

    println!("{:?} len={}", buggy_path, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &buggy, &fixed);
    let len = algorithms::gumtree::diff(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    )
    .actions
    .unwrap()
    .len();
    let processing_time = now.elapsed().as_secs_f64();
    println!("tt={} evos={}", processing_time, len);
    match guard.report().build() {
        Ok(report) => {
            let file = File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
            // let mut file = File::create("profile.pb").unwrap();
            // let profile = report.pprof().unwrap();
            // use pprof::protos::Message;
            // let mut content = Vec::new();
            // profile.encode(&mut content).unwrap();
            // file.write_all(&content).unwrap();
        }
        Err(_) => {}
    };

    // ./dist/build/install/gumtree/bin/gumtree textdiff /home/quentin/rusted_gumtree3/benchmark_diffs/src/C1.java.json /home/quentin/rusted_gumtree3/benchmark_diffs/src/C2.java.json -m gumtree -g java-hyperast
}

pub fn bad_perfs3() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    //Cli/29/src_java_org_apache_commons_cli_Util.java
    //JxPath/8/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationRelationalExpression.java
    //JxPath/18/src_java_org_apache_commons_jxpath_ri_axes_AttributeContext.java
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
    let [buggy_path, fixed_path] =
        src_dst.map(|x| x.join("Jsoup/91/src_main_java_org_jsoup_UncheckedIOException.java"));
    bad_perfs_helper(&buggy_path, &fixed_path);

    // ./dist/build/install/gumtree/bin/gumtree textdiff /home/quentin/rusted_gumtree3/benchmark_diffs/src/C1.java.json /home/quentin/rusted_gumtree3/benchmark_diffs/src/C2.java.json -m gumtree -g java-hyperast
}

pub fn bad_perfs4() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    //Cli/29/src_java_org_apache_commons_cli_Util.java
    //JxPath/8/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationRelationalExpression.java
    //JxPath/18/src_java_org_apache_commons_jxpath_ri_axes_AttributeContext.java
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
    let [buggy_path, fixed_path] = src_dst.map(|x| {
        x.join("JxPath/6/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationCompare.java")
    });
    bad_perfs_helper(&buggy_path, &fixed_path);
}

pub fn bad_perfs5() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
    let [buggy_path, fixed_path] = src_dst.map(|x| {
        x.join("Mockito/5/src_org_mockito_internal_verification_VerificationOverTimeImpl.java")
    });
    bad_perfs_helper(&buggy_path, &fixed_path);
}

pub fn bad_perfs6() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
    let [buggy_path, fixed_path] =
        src_dst.map(|x| x.join("JacksonDatabind/25/src_main_java_com_fasterxml_jackson_databind_module_SimpleAbstractTypeResolver.java"));
    bad_perfs_helper(&buggy_path, &fixed_path);
}

pub fn bad_perfs7() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
    let [buggy_path, fixed_path] =
        src_dst.map(|x| x.join("Chart/19/source_org_jfree_chart_plot_CategoryPlot.simp.java"));
    bad_perfs_helper(&buggy_path, &fixed_path);
}

pub fn bad_perfs8() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
    let [buggy_path, fixed_path] =
        src_dst.map(|x| x.join("Math/76/src_main_java_org_apache_commons_math_linear_SingularValueDecompositionImpl.simp.java"));
    bad_perfs_helper(&buggy_path, &fixed_path);
}

pub fn bad_perfs9() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let src_dst = buggy_fixed_dataset_roots(root, "defects4j");
    let [buggy_path, fixed_path] =
        src_dst.map(|x| x.join("Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java"));
    bad_perfs_helper(&buggy_path, &fixed_path);
}

fn bad_perfs_helper(buggy_path: &Path, fixed_path: &Path) {
    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: Default::default(),
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &buggy, &fixed);
    let buggy_s = src_tr.local.metrics.size;
    let fixed_s = dst_tr.local.metrics.size;
    dbg!(buggy_s, fixed_s);
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&[
            // "libc",
            "libgcc", "pthread", "vdso",
        ])
        .build()
        .unwrap();
    let DiffResult {
        mapper,
        actions,
        exec_data,
    } = algorithms::gumtree::diff(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );
    let actions = actions.unwrap();
    // let MappingDurations([subtree_matcher_t, bottomup_matcher_t]) = mapping_durations.into();
    match guard.report().build() {
        Ok(report) => {
            let file = File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
            let mut file = File::create("profile.pb").unwrap();
            let profile = report.pprof().unwrap();
            use pprof::protos::Message;
            let mut content = Vec::new();
            profile.encode(&mut content).unwrap();
            file.write_all(&content).unwrap();
        }
        Err(_) => {}
    };
    let hast_timings = algorithms::Phased::sum::<std::time::Duration>(&exec_data);
    // let hast_timings = [subtree_matcher_t, bottomup_matcher_t, gen_t + prepare_gen_t];
    let gt_out = other_tools::gumtree::subprocess(
        &stores,
        src_tr.local.compressed_node,
        dst_tr.local.compressed_node,
        "gumtree",
        "Chawathe",
        60 * 5,
        "JSON",
    )
    .unwrap();
    let now = Instant::now();
    let pp = SimpleJsonPostProcess::new(&gt_out);
    let gt_timings = pp.performances();
    let counts = pp.counts();
    let valid = pp.validity_mappings(&mapper);
    let processing_time = now.elapsed().as_secs_f64();
    dbg!(processing_time);
    if valid.additional_mappings.len() > 0 || valid.missing_mappings.len() > 0 {
        dbg!(
            valid.additional_mappings,
            valid.missing_mappings,
            actions.len(),
            counts.actions
        );
        panic!()
    } else if counts.actions < 0 {
        dbg!(actions.len(), counts.actions);
        panic!()
    } else if counts.actions as usize != actions.len() {
        dbg!(actions.len(), counts.actions);
        panic!()
    } else {
        println!("gt_tt={:?} evos={}", &gt_timings, counts.actions);
        println!("tt={:?} evos={}", &hast_timings, actions.len())
    }
}

#[test]
fn test_all() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    // let guard = pprof::ProfilerGuardBuilder::default()
    //     .frequency(1000)
    //     .blocklist(&[
    //         // "libc",
    //         "libgcc", "pthread", "vdso",
    //     ])
    //     .build()
    //     .unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let [root_buggy, root_fixed] = buggy_fixed_dataset_roots(root, "defects4j");
    for buggy_project in iter_dirs(&root_buggy) {
        for buggy_case in iter_dirs(&buggy_project.path()) {
            let buggy_path = std::fs::read_dir(buggy_case.path())
                .expect("should be a dir")
                .into_iter()
                .filter_map(|x| x.ok())
                .filter(|x| x.file_type().unwrap().is_file())
                .next()
                .unwrap()
                .path();
            let fixed_path = root_fixed.join(buggy_path.strip_prefix(&root_buggy).unwrap());
            let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
            let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
            let mut stores = SimpleStores {
                label_store: LabelStore::new(),
                type_store: Default::default(),
                node_store: NodeStore::new(),
            };
            let mut md_cache = Default::default();
            let now = Instant::now();

            println!("{:?} len={}", buggy_path, buggy.len());
            let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &buggy, &fixed);
            let len = algorithms::gumtree::diff(
                &stores,
                &src_tr.local.compressed_node,
                &dst_tr.local.compressed_node,
            )
            .actions
            .unwrap()
            .len();
            let processing_time = now.elapsed().as_secs_f64();
            println!("tt={} evos={}", processing_time, len);
            break;
        }
        break;
    }
    // match guard.report().build() {
    //     Ok(report) => {
    //         let mut file = File::create("profile.pb").unwrap();
    //         let profile = report.pprof().unwrap();
    //         use pprof::protos::Message;
    //         let mut content = Vec::new();
    //         profile.encode(&mut content).unwrap();
    //         file.write_all(&content).unwrap();
    //     }
    //     Err(_) => {}
    // };
}

/// TODO add to CLI
pub fn all() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let [root_buggy, root_fixed] = buggy_fixed_dataset_roots(root, "defects4j");
    let args: Vec<String> = env::args().collect();
    let (out_path, mut out_file) = if let Some(op) = args.get(1) {
        (Path::new(op).to_owned(), File::create(&args[1]).unwrap())
    } else {
        tempfile().unwrap()
    };
    iter_dirs(&root_buggy)
        .flat_map(|buggy_project| iter_dirs(&buggy_project.path()))
        // .flat_map(|buggy_project|
        //     std::fs::read_dir(buggy_case.path())
        //     .expect("should be a dir")
        //     .into_iter()
        //     .filter_map(|x| x.ok())
        //     .filter(|x| x.file_type().unwrap().is_file())
        // )
        .map(|buggy_case| {
            std::fs::read_dir(buggy_case.path())
                .expect("should be a dir")
                .into_iter()
                .filter_map(|x| x.ok())
                .filter(|x| x.file_type().unwrap().is_file())
                .next()
                .unwrap()
                .path()
        })
        .map(|buggy_path| find(buggy_path, &root_buggy, &root_fixed))
        .for_each(
            |Case {
                 buggy_path,
                 fixed_path,
                 name,
             }| {
                run(&buggy_path, &fixed_path, &name).map(|x| writeln!(out_file, "{}", x).unwrap());
            },
        );
    println!("wrote csv at {:?}", out_path);
}

/// TODO add to CLI
pub fn once() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let [root_buggy, root_fixed] = buggy_fixed_dataset_roots(root, "defects4j");
    let args: Vec<String> = env::args().collect();

    let buggy_path = root_buggy.join(args.get(1).expect("path to buggy file"));
    let Case {
        buggy_path,
        fixed_path,
        name,
    } = find(buggy_path, &root_buggy, &root_fixed);
    run(&buggy_path, &fixed_path, &name).unwrap();
}

struct Case {
    buggy_path: std::path::PathBuf,
    fixed_path: std::path::PathBuf,
    name: std::path::PathBuf,
}

fn find(buggy_path: std::path::PathBuf, root_buggy: &Path, root_fixed: &Path) -> Case {
    let name = buggy_path
        .clone()
        .strip_prefix(root_buggy)
        .unwrap()
        .to_path_buf();
    let fixed_path = root_fixed.join(&name);
    Case {
        buggy_path,
        name,
        fixed_path,
    }
}

pub fn run(buggy_path: &Path, fixed_path: &Path, name: &Path) -> Option<String> {
    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: Default::default(),
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    println!("{:?} len={}", name, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, &buggy, &fixed);
    let buggy_s = src_tr.local.metrics.size;
    let fixed_s = dst_tr.local.metrics.size;
    let gt_out_format = "COMPRESSED"; // JSON
    let gt_out = other_tools::gumtree::subprocess(
        &stores,
        src_tr.local.compressed_node,
        dst_tr.local.compressed_node,
        "gumtree",
        "Chawathe",
        60 * 5,
        gt_out_format,
    )
    .unwrap();

    let DiffResult {
        mapper,
        actions,
        exec_data,
    } = algorithms::gumtree::diff(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );

    let timings = [
        exec_data.phase1().sum::<std::time::Duration>(),
        exec_data.phase2().sum(),
        exec_data.phase3().sum(),
    ]
    .map(|x| x.unwrap());

    let hast_actions = actions.unwrap().len();
    dbg!(&timings);
    let res = if gt_out_format == "COMPRESSED" {
        let pp = CompressedBfPostProcess::create(&gt_out);
        let (pp, counts) = pp.counts();
        let (pp, gt_timings) = pp.performances();
        let valid = pp.validity_mappings(&mapper);
        Some((gt_timings, counts, valid))
    } else if gt_out_format == "JSON" {
        let pp = SimpleJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        let valid = pp.validity_mappings(&mapper);
        Some((gt_timings, counts, valid.map(|x| x.len())))
    } else {
        unimplemented!("gt_out_format {} is not implemented", gt_out_format)
    };

    res.map(|(gt_timings, gt_counts, valid)| {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            name.to_string_lossy(),
            buggy_s,
            fixed_s,
            hast_actions,
            gt_counts.actions,
            valid.missing_mappings,
            valid.additional_mappings,
            &timings[0].as_secs_f64(),
            &timings[1].as_secs_f64(),
            &timings[2].as_secs_f64(),
            &gt_timings[0].as_secs_f64(),
            &gt_timings[1].as_secs_f64(),
            &gt_timings[2].as_secs_f64(),
        )
    })
}

pub fn run_dir(src: &Path, dst: &Path) -> Option<String> {
    let stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: Default::default(),
        node_store: NodeStore::new(),
    };
    let md_cache = Default::default();
    let mut java_gen = JavaPreprocessFileSys {
        main_stores: stores,
        java_md_cache: md_cache,
    };
    let now = Instant::now();
    let (src_tr, dst_tr) = parse_dir_pair(&mut java_gen, &src, &dst);
    let parse_t = now.elapsed().as_secs_f64();

    let stores = hyperast_vcs_git::no_space::as_nospaces2(&java_gen.main_stores);

    dbg!(&parse_t);
    dbg!(&src_tr.metrics.size);
    dbg!(&dst_tr.metrics.size);
    let buggy_s = src_tr.metrics.size;
    let fixed_s = dst_tr.metrics.size;

    let gt_out_format = "COMPRESSED"; // JSON

    let DiffResult {
        mapper,
        actions: hast_actions,
        exec_data,
    } = algorithms::gumtree::diff(&stores, &src_tr.compressed_node, &dst_tr.compressed_node);
    let gt_out = other_tools::gumtree::subprocess(
        &stores,
        src_tr.compressed_node,
        dst_tr.compressed_node,
        "gumtree",
        "Chawathe",
        60 * 5,
        gt_out_format,
    )
    .unwrap();

    let timings = [
        exec_data.phase1().sum::<std::time::Duration>(),
        exec_data.phase2().sum(),
        exec_data.phase3().sum(),
    ]
    .map(|x| x.unwrap());

    dbg!(&timings);
    let res = if gt_out_format == "COMPRESSED" {
        let pp = CompressedBfPostProcess::create(&gt_out);
        let (pp, counts) = pp.counts();
        let (pp, gt_timings) = pp.performances();
        let valid = pp.validity_mappings(&mapper);
        Some((gt_timings, counts, valid))
    } else if gt_out_format == "JSON" {
        let pp = PathJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        let valid = pp.validity_mappings(&mapper);
        Some((gt_timings, counts, valid.map(|x| x.len())))
    } else {
        unimplemented!("gt_out_format {} is not implemented", gt_out_format)
    };

    res.map(|(gt_timings, gt_counts, valid)| {
        format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            src.to_string_lossy(),
            buggy_s,
            fixed_s,
            hast_actions.unwrap().len(),
            gt_counts.actions,
            valid.missing_mappings,
            valid.additional_mappings,
            &timings[0].as_secs_f64(),
            &timings[1].as_secs_f64(),
            &timings[2].as_secs_f64(),
            &gt_timings[0].as_secs_f64(),
            &gt_timings[1].as_secs_f64(),
            &gt_timings[2].as_secs_f64(),
        )
    })
}
