use std::{env, fs::File, io::Write, path::Path, time::Instant};

use crate::{
    algorithms::{self, DiffResult},
    other_tools,
    postprocess::{CompressedBfPostProcess, SimpleJsonPostProcess},
    preprocess::{iter_dirs, parse_dir_pair, parse_string_pair, JavaPreprocessFileSys},
    tempfile,
};
use hyper_ast::store::{labels::LabelStore, nodes::legion::NodeStore, SimpleStores, TypeStore};
use hyper_ast_gen_ts_java::legion_with_refs::JavaTreeGen;
use hyper_gumtree::actions::Actions;

#[test]
fn test_simple_1() {
    let buggy = r#"class A{class C{}class B{{while(1){if(1){}else{}};}}}class D{class E{}class F{{while(2){if(2){}else{}};}}}"#;
    let fixed = r#"class A{class C{}}class B{{while(1){if(1){}else{}};}}class D{class E{}}class F{{while(2){if(2){}else{}};}}"#;
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);

    println!(
        "{}",
        algorithms::gumtree::diff(
            &java_tree_gen.stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node
        )
        .actions
        .len()
    )
}

#[test]
fn test_crash1() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let buggy_path = Path::new(
        "../../gt_datasets/defects4j/buggy/Cli/22/src_java_org_apache_commons_cli_PosixParser.java",
    );
    let fixed_path = Path::new(
        "../../gt_datasets/defects4j/fixed/Cli/22/src_java_org_apache_commons_cli_PosixParser.java",
    );
    let buggy = std::fs::read_to_string(buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    print!("{:?} len={}: ", buggy_path, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);
    let len = algorithms::gumtree::diff(
        &java_tree_gen.stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    )
    .actions
    .len();
    println!("{}", len);
}

#[cfg(test)]
mod examples {

    use hyper_ast_gen_ts_java::legion_with_refs::TreeJsonSerializer;

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
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        };
        let mut md_cache = Default::default();
        let mut java_tree_gen = JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
            md_cache: &mut md_cache,
        };
        print!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);
        let len = algorithms::gumtree::diff(
            &java_tree_gen.stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        )
        .actions
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
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        };
        let mut md_cache = Default::default();
        let mut java_tree_gen = JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
            md_cache: &mut md_cache,
        };
        print!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);
        let len = algorithms::gumtree::diff(
            &java_tree_gen.stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        )
        .actions
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

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(1000)
            .blocklist(&[
                // "libc",
                "libgcc", "pthread", "vdso",
            ])
            .build()
            .unwrap();

        let mut stores = SimpleStores {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        };
        let mut md_cache = Default::default();
        let mut java_tree_gen = JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
            md_cache: &mut md_cache,
        };
        let now = Instant::now();

        println!("{} len={}", "buggy", buggy.len());
        let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);
        let len = algorithms::gumtree::diff(
            &java_tree_gen.stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        )
        .actions
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
        let (src, mut src_f) = tempfile().unwrap();
        // dbg!(&src);

        src_f
            .write_all(
                (TreeJsonSerializer::<true>::new(
                    &java_tree_gen.stores.node_store,
                    &java_tree_gen.stores.label_store,
                    src_tr.local.compressed_node.clone(),
                )
                .to_string())
                .as_bytes(),
            )
            .unwrap();
        drop(src_f);
        let (dst, mut dst_f) = tempfile().unwrap();
        // dbg!(&dst);
        dst_f
            .write_all(
                (TreeJsonSerializer::<true>::new(
                    &java_tree_gen.stores.node_store,
                    &java_tree_gen.stores.label_store,
                    dst_tr.local.compressed_node.clone(),
                )
                .to_string())
                .as_bytes(),
            )
            .unwrap();
        drop(dst_f);
        let (json, mut json_f) = tempfile().unwrap();

        let now = Instant::now();
        std::process::Command::new("/usr/bin/bash")
            .arg(root.join("gt_script.sh").to_str().unwrap())
            .arg(src)
            .arg(dst)
            .arg(&"JSON")
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

        let o = serde_json::from_reader::<_, diff_output::F>(
            File::open(json).expect("should be a file"),
        )
        .unwrap();
        let gt_len: usize = o.actions.len();
        dbg!(&o.times);
        println!("gt_tt={} gt_l={}", gt_processing_time, gt_len);
    }
}

#[cfg(test)]
mod test {
    use std::io::stdout;

    use crate::{other_tools::gumtree::subprocess, postprocess::print_mappings};

    use super::*;
    use hyper_ast::{
        nodes::{print_tree_syntax_with_ids, IoOut},
        store::{defaults::NodeIdentifier, nodes::legion::HashedNodeRef},
    };
    use hyper_ast_gen_ts_xml::legion::XmlTreeGen;
    use hyper_gumtree::{
        decompressed_tree_store::CompletePostOrder,
        matchers::{
            heuristic::gt::greedy_subtree_matcher::{GreedySubtreeMatcher, SubtreeMatcher},
            mapping_store::VecStore,
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
        let mut stores = SimpleStores {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        };
        let mut tree_gen = XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
        };
        print!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = {
            let tree_gen = &mut tree_gen;
            let full_node1 = {
                let tree = match XmlTreeGen::tree_sitter_parse(buggy.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                let full_node1 =
                    tree_gen.generate_file("".as_bytes(), buggy.as_bytes(), tree.walk());
                full_node1
            };
            let full_node2 = {
                let tree = match XmlTreeGen::tree_sitter_parse(fixed.as_bytes()) {
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

        use hyper_ast::types::LabelStore as _;
        print_tree_syntax_with_ids(
            |id: &NodeIdentifier| -> _ {
                tree_gen
                    .stores
                    .node_store
                    .resolve(id.clone())
                    .into_compressed_node()
                    .unwrap()
            },
            |id| -> _ { tree_gen.stores.label_store.resolve(id).to_owned() },
            &src,
            &mut Into::<IoOut<_>>::into(stdout()),
        );
        print_tree_syntax_with_ids(
            |id: &NodeIdentifier| -> _ {
                tree_gen
                    .stores
                    .node_store
                    .resolve(id.clone())
                    .into_compressed_node()
                    .unwrap()
            },
            |id| -> _ { tree_gen.stores.label_store.resolve(id).to_owned() },
            &dst,
            &mut Into::<IoOut<_>>::into(stdout()),
        );
        {
            let stores = &tree_gen.stores;
            let mappings = VecStore::default();
            type DS = CompletePostOrder<NodeIdentifier, u32>;
            let mapper = GreedySubtreeMatcher::<DS, DS, _, HashedNodeRef, _, _>::matchh(
                &stores.node_store,
                &src,
                &dst,
                mappings,
            );
            let SubtreeMatcher {
                src_arena,
                dst_arena,
                mappings,
                ..
            } = mapper.into();
            print_mappings(&dst_arena, &src_arena, stores, &mappings);
        }

        subprocess(&stores, src, dst, "gumtree-simple", "JSON");
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
        let mut stores = SimpleStores {
            label_store: LabelStore::new(),
            type_store: TypeStore {},
            node_store: NodeStore::new(),
        };
        let mut tree_gen = XmlTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: &mut stores,
        };
        print!("len={}: ", buggy.len());
        let (src_tr, dst_tr) = {
            let full_node1 = {
                let tree = match XmlTreeGen::tree_sitter_parse(buggy.as_bytes()) {
                    Ok(t) => t,
                    Err(t) => t,
                };
                tree_gen.generate_file("pom.xml".as_bytes(), buggy.as_bytes(), tree.walk())
            };
            let full_node2 = {
                let tree = match XmlTreeGen::tree_sitter_parse(fixed.as_bytes()) {
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

        use hyper_ast::types::LabelStore as _;
        print_tree_syntax_with_ids(
            |id: &NodeIdentifier| -> _ {
                tree_gen
                    .stores
                    .node_store
                    .resolve(id.clone())
                    .into_compressed_node()
                    .unwrap()
            },
            |id| -> _ { tree_gen.stores.label_store.resolve(id).to_owned() },
            &src,
            &mut Into::<IoOut<_>>::into(stdout()),
        );
        print_tree_syntax_with_ids(
            |id: &NodeIdentifier| -> _ {
                tree_gen
                    .stores
                    .node_store
                    .resolve(id.clone())
                    .into_compressed_node()
                    .unwrap()
            },
            |id| -> _ { tree_gen.stores.label_store.resolve(id).to_owned() },
            &dst,
            &mut Into::<IoOut<_>>::into(stdout()),
        );
        let stores = &tree_gen.stores;
        let mappings = VecStore::default();
        type DS = CompletePostOrder<NodeIdentifier, u32>;
        let mapper = GreedySubtreeMatcher::<DS, DS, _, HashedNodeRef, _, _>::matchh(
            &stores.node_store,
            &src,
            &dst,
            mappings,
        );
        let SubtreeMatcher {
            src_arena,
            dst_arena,
            mappings,
            ..
        } = mapper.into();
        print_mappings(&dst_arena, &src_arena, stores, &mappings);

        let gt_out = subprocess(&stores, src, dst, "gumtree-subtree", "JSON");

        let pp = SimpleJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        dbg!(gt_timings, counts.mappings, counts.actions);
        let valid = pp.validity_mappings(&stores, &src_arena, src, &dst_arena, dst, &mappings);
        dbg!(valid.additional_mappings, valid.missing_mappings);
    }
}

#[test]
fn compare_perfs() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    // let buggy_path = Path::new("../../gt_datasets/defects4j/buggy/JxPath/8/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationRelationalExpression.java");
    // let fixed_path = Path::new("../../gt_datasets/defects4j/fixed/JxPath/8/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationRelationalExpression.java");
    let buggy_path = Path::new(
        "../../gt_datasets/defects4j/buggy/Cli/22/src_java_org_apache_commons_cli_PosixParser.java",
    );
    let fixed_path = Path::new(
        "../../gt_datasets/defects4j/fixed/Cli/22/src_java_org_apache_commons_cli_PosixParser.java",
    );
    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let now = Instant::now();

    println!("{:?} len={}", buggy_path, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);
    let len = algorithms::gumtree::diff(
        &java_tree_gen.stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    )
    .actions
    .len();
    let processing_time = now.elapsed().as_secs_f64();
    println!("tt={} evos={}", processing_time, len);

    let (src, _) = tempfile().unwrap();
    let (dst, _) = tempfile().unwrap();

    let now = Instant::now();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    std::process::Command::new("/usr/bin/bash")
        .arg(root.join("gt_script.sh").to_str().unwrap())
        .arg(src)
        .arg(dst)
        .arg(&"JSON")
        .arg("/dev/null")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .output()
        .expect("failed to execute process");

    let processing_time = now.elapsed().as_secs_f64();
    println!("gt_tt={}", processing_time);

    // ./dist/build/install/gumtree/bin/gumtree textdiff /home/quentin/rusted_gumtree3/benchmark_diffs/src/C1.java.json /home/quentin/rusted_gumtree3/benchmark_diffs/src/C2.java.json -m gumtree -g java-hyperast
}

//

#[test]
fn test_bad_perfs() {
    // bad_perfs()
    // bad_perfs2()
    bad_perfs3()
}

pub fn bad_perfs() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&[
            // "libc",
            "libgcc", "pthread", "vdso",
        ])
        .build()
        .unwrap();
    let buggy_path = root.join(
        "gt_datasets/defects4j/buggy/JacksonDatabind/31/src_main_java_com_fasterxml_jackson_databind_util_TokenBuffer.java",
    );
    let fixed_path = root.join(
        "gt_datasets/defects4j/fixed/JacksonDatabind/31/src_main_java_com_fasterxml_jackson_databind_util_TokenBuffer.java",
    );
    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let now = Instant::now();

    println!("{:?} len={}", buggy_path, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);
    let len = algorithms::gumtree::diff(
        &java_tree_gen.stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    )
    .actions
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&[
            // "libc",
            "libgcc", "pthread", "vdso",
        ])
        .build()
        .unwrap();
    let buggy_path =
        root.join("gt_datasets/defects4j/buggy/Chart/4/source_org_jfree_chart_plot_XYPlot.java");
    let fixed_path =
        root.join("gt_datasets/defects4j/fixed/Chart/4/source_org_jfree_chart_plot_XYPlot.java");
    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let now = Instant::now();

    println!("{:?} len={}", buggy_path, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);
    let len = algorithms::gumtree::diff(
        &java_tree_gen.stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    )
    .actions
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    //Cli/29/src_java_org_apache_commons_cli_Util.java
    //JxPath/8/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationRelationalExpression.java
    //JxPath/18/src_java_org_apache_commons_jxpath_ri_axes_AttributeContext.java
    let buggy_path = root.parent().unwrap().join(
        "gt_datasets/defects4j/buggy/Jsoup/91/src_main_java_org_jsoup_UncheckedIOException.java",
    );
    let fixed_path = root.parent().unwrap().join(
        "gt_datasets/defects4j/fixed/Jsoup/91/src_main_java_org_jsoup_UncheckedIOException.java",
    );
    bad_perfs_helper(buggy_path, fixed_path);

    // ./dist/build/install/gumtree/bin/gumtree textdiff /home/quentin/rusted_gumtree3/benchmark_diffs/src/C1.java.json /home/quentin/rusted_gumtree3/benchmark_diffs/src/C2.java.json -m gumtree -g java-hyperast
}

pub fn bad_perfs4() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    //Cli/29/src_java_org_apache_commons_cli_Util.java
    //JxPath/8/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationRelationalExpression.java
    //JxPath/18/src_java_org_apache_commons_jxpath_ri_axes_AttributeContext.java
    let buggy_path = root
        .parent()
        .unwrap()
        .join("gt_datasets/defects4j/buggy/JxPath/6/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationCompare.java");
    let fixed_path = root
        .parent()
        .unwrap()
        .join("gt_datasets/defects4j/fixed/JxPath/6/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationCompare.java");
    bad_perfs_helper(buggy_path, fixed_path);
}

pub fn bad_perfs5() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let buggy_path = root
        .parent()
        .unwrap()
        .join("gt_datasets/defects4j/buggy/Mockito/5/src_org_mockito_internal_verification_VerificationOverTimeImpl.java");
    let fixed_path = root
        .parent()
        .unwrap()
        .join("gt_datasets/defects4j/fixed/Mockito/5/src_org_mockito_internal_verification_VerificationOverTimeImpl.java");
    bad_perfs_helper(buggy_path, fixed_path);
}

pub fn bad_perfs6() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let buggy_path = root
        .parent()
        .unwrap()
        .join("gt_datasets/defects4j/buggy/JacksonDatabind/25/src_main_java_com_fasterxml_jackson_databind_module_SimpleAbstractTypeResolver.java");
    let fixed_path = root
        .parent()
        .unwrap()
        .join("gt_datasets/defects4j/fixed/JacksonDatabind/25/src_main_java_com_fasterxml_jackson_databind_module_SimpleAbstractTypeResolver.java");
    bad_perfs_helper(buggy_path, fixed_path);
}

pub fn bad_perfs7() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let buggy_path = root.parent().unwrap().join(
        "gt_datasets/defects4j/buggy/Chart/19/source_org_jfree_chart_plot_CategoryPlot.simp.java",
    );
    let fixed_path = root.parent().unwrap().join(
        "gt_datasets/defects4j/fixed/Chart/19/source_org_jfree_chart_plot_CategoryPlot.simp.java",
    );
    bad_perfs_helper(buggy_path, fixed_path);
}

pub fn bad_perfs8() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let buggy_path = root
        .parent()
        .unwrap()
        .join("gt_datasets/defects4j/buggy/Math/76/src_main_java_org_apache_commons_math_linear_SingularValueDecompositionImpl.simp.java");
    let fixed_path = root
        .parent()
        .unwrap()
        .join("gt_datasets/defects4j/fixed/Math/76/src_main_java_org_apache_commons_math_linear_SingularValueDecompositionImpl.simp.java");
    bad_perfs_helper(buggy_path, fixed_path);
}

pub fn bad_perfs9() {
    // https://github.com/GumTreeDiff/datasets/tree/2bd8397f5939233a7d6205063bac9340d59f5165/defects4j/{buggy,fixed}/*/[0-9]+/*
    println!("{:?}", std::env::current_dir());
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let buggy_path = root.parent().unwrap().join(
        "gt_datasets/defects4j/buggy/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
    );
    let fixed_path = root.parent().unwrap().join(
        "gt_datasets/defects4j/fixed/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
    );
    bad_perfs_helper(buggy_path, fixed_path);
}

fn bad_perfs_helper(buggy_path: std::path::PathBuf, fixed_path: std::path::PathBuf) {
    let buggy = std::fs::read_to_string(&buggy_path).expect("the buggy code");
    let fixed = std::fs::read_to_string(fixed_path).expect("the fixed code");
    let mut stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);
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
        mapping_durations: [subtree_matcher_t, bottomup_matcher_t],
        src_arena,
        dst_arena,
        mappings,
        actions,
        gen_t,
    } = algorithms::gumtree::diff(
        &java_tree_gen.stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );
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
    let hast_timings = [subtree_matcher_t, bottomup_matcher_t, gen_t];
    let gt_out = other_tools::gumtree::subprocess(
        &java_tree_gen.stores,
        src_tr.local.compressed_node,
        dst_tr.local.compressed_node,
        "gumtree",
        "JSON",
    );
    let now = Instant::now();
    let pp = SimpleJsonPostProcess::new(&gt_out);
    let gt_timings = pp.performances();
    let counts = pp.counts();
    let valid = pp.validity_mappings(
        &java_tree_gen.stores,
        &src_arena,
        src_tr.local.compressed_node,
        &dst_arena,
        dst_tr.local.compressed_node,
        &mappings,
    );
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
    } else if counts.actions != actions.len() {
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
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&[
            // "libc",
            "libgcc", "pthread", "vdso",
        ])
        .build()
        .unwrap();
    let root = Path::new("../../gt_datasets/defects4j");
    std::fs::read_dir(root).expect("should be a dir");
    let root_buggy = root.join("buggy");
    let root_fixed = root.join("fixed");
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
                type_store: TypeStore {},
                node_store: NodeStore::new(),
            };
            let mut md_cache = Default::default();
            let mut java_tree_gen = JavaTreeGen {
                line_break: "\n".as_bytes().to_vec(),
                stores: &mut stores,
                md_cache: &mut md_cache,
            };
            let now = Instant::now();

            println!("{:?} len={}", buggy_path, buggy.len());
            let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);
            let len = algorithms::gumtree::diff(
                &java_tree_gen.stores,
                &src_tr.local.compressed_node,
                &dst_tr.local.compressed_node,
            )
            .actions
            .len();
            let processing_time = now.elapsed().as_secs_f64();
            println!("tt={} evos={}", processing_time, len);
            break;
        }
        break;
    }
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
}

/// TODO add to CLI
pub fn all() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let data_root = root.parent().unwrap().join("gt_datasets/defects4j");
    let data_root = data_root.as_path();
    std::fs::read_dir(data_root).expect("should be a dir");
    let root_buggy = data_root.join("buggy");
    let root_fixed = data_root.join("fixed");
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
    let data_root = root.parent().unwrap().join("gt_datasets/defects4j");
    let data_root = data_root.as_path();
    std::fs::read_dir(data_root).expect("should be a dir");
    let root_buggy = data_root.join("buggy");
    let root_fixed = data_root.join("fixed");
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
        type_store: TypeStore {},
        node_store: NodeStore::new(),
    };
    let mut md_cache = Default::default();
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: &mut stores,
        md_cache: &mut md_cache,
    };
    println!("{:?} len={}", name, buggy.len());
    let (src_tr, dst_tr) = parse_string_pair(&mut java_tree_gen, &buggy, &fixed);
    let buggy_s = src_tr.local.metrics.size;
    let fixed_s = dst_tr.local.metrics.size;
    let gt_out_format = "COMPRESSED"; // JSON
    let gt_out = other_tools::gumtree::subprocess(
        &java_tree_gen.stores,
        src_tr.local.compressed_node,
        dst_tr.local.compressed_node,
        "gumtree",
        gt_out_format,
    );

    let DiffResult {
        mapping_durations: [subtree_matcher_t, bottomup_matcher_t],
        src_arena,
        dst_arena,
        mappings,
        actions,
        gen_t,
    } = algorithms::gumtree::diff(
        &java_tree_gen.stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );

    let timings = vec![subtree_matcher_t, bottomup_matcher_t, gen_t];

    let hast_actions = actions.len();
    dbg!(&timings);
    let res = if gt_out_format == "COMPRESSED" {
        let pp = CompressedBfPostProcess::create(&gt_out);
        let (pp, counts) = pp.counts();
        let (pp, gt_timings) = pp.performances();
        let valid = pp.validity_mappings(
            &java_tree_gen.stores,
            &src_arena,
            src_tr.local.compressed_node,
            &dst_arena,
            dst_tr.local.compressed_node,
            &mappings,
        );
        Some((gt_timings, counts, valid))
    } else if gt_out_format == "JSON" {
        let pp = SimpleJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        let valid = pp.validity_mappings(
            &java_tree_gen.stores,
            &src_arena,
            src_tr.local.compressed_node,
            &dst_arena,
            dst_tr.local.compressed_node,
            &mappings,
        );
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
            &timings[0],
            &timings[1],
            &timings[2],
            &gt_timings[0],
            &gt_timings[1],
            &gt_timings[2],
        )
    })
}

pub fn run_dir(src: &Path, dst: &Path) -> Option<String> {
    let stores = SimpleStores {
        label_store: LabelStore::new(),
        type_store: TypeStore {},
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

    dbg!(&parse_t);
    dbg!(&src_tr.metrics.size);
    dbg!(&dst_tr.metrics.size);
    let buggy_s = src_tr.metrics.size;
    let fixed_s = dst_tr.metrics.size;

    let gt_out_format = "COMPRESSED"; // JSON
    let gt_out = other_tools::gumtree::subprocess(
        &java_gen.main_stores,
        src_tr.compressed_node,
        dst_tr.compressed_node,
        "gumtree",
        gt_out_format,
    );

    let DiffResult {
        mapping_durations: [subtree_matcher_t, bottomup_matcher_t],
        src_arena,
        dst_arena,
        mappings,
        actions: hast_actions,
        gen_t,
    } = algorithms::gumtree::diff(
        &java_gen.main_stores,
        &src_tr.compressed_node,
        &dst_tr.compressed_node,
    );

    let timings = vec![subtree_matcher_t, bottomup_matcher_t, gen_t];

    dbg!(&timings);
    let res = if gt_out_format == "COMPRESSED" {
        let pp = CompressedBfPostProcess::create(&gt_out);
        let (pp, counts) = pp.counts();
        let (pp, gt_timings) = pp.performances();
        let valid = pp.validity_mappings(
            &java_gen.main_stores,
            &src_arena,
            src_tr.compressed_node,
            &dst_arena,
            dst_tr.compressed_node,
            &mappings,
        );
        Some((gt_timings, counts, valid))
    } else if gt_out_format == "JSON" {
        let pp = SimpleJsonPostProcess::new(&gt_out);
        let gt_timings = pp.performances();
        let counts = pp.counts();
        let valid = pp.validity_mappings(
            &java_gen.main_stores,
            &src_arena,
            src_tr.compressed_node,
            &dst_arena,
            dst_tr.compressed_node,
            &mappings,
        );
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
            hast_actions.len(),
            gt_counts.actions,
            valid.missing_mappings,
            valid.additional_mappings,
            &timings[0],
            &timings[1],
            &timings[2],
            &gt_timings[0],
            &gt_timings[1],
            &gt_timings[2],
        )
    })
}
