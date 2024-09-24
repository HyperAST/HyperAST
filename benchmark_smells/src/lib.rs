#![feature(array_chunks)]
#![feature(iter_array_chunks)]
//! Benchmark of smells finder using the hyperAST.
//!
//! validity: baseline tree-sitter, same number of matches (tree-sitter and our query syntax have slightly diverged, so it can only be done on a subset)
//!
//! performances: baseline tree-sitter, time/memory show perf issues when not using our approach
//!
//! code: repository (reuse known repositories) (but tree-sitter does not work on entire commits) ) / files (reuse tsg dataset)
//!
//! Priorities:
//! The main objective of this benchmark suite is to measure performances (mem, latency, wall time).
//! The validity will first be limitated to the capabilities of the baseline.

pub mod github_ranges;
pub mod positions;
pub mod queries;
pub mod simple;

use std::{env, fs, io, path, time};

pub fn tempfile() -> io::Result<(path::PathBuf, fs::File)> {
    let mut path = env::temp_dir();
    let file_name = time::SystemTime::UNIX_EPOCH;
    path.push(file_name.elapsed().unwrap().as_nanos().to_string());
    let file = fs::File::create(&path)?;
    Ok((path, file))
}

pub fn with_profiling<F: Fn()>(out: &path::Path, f: F) {
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&[
            // "libc",
            "libgcc", "pthread", "vdso",
        ])
        .build()
        .unwrap();
    f();
    match guard.report().build() {
        Ok(report) => {
            let mut file = fs::File::create(out).unwrap();
            let profile = report.pprof().unwrap();
            use pprof::protos::Message;
            let mut content = Vec::new();
            profile.encode(&mut content).unwrap();
            use io::Write;
            file.write_all(&content).unwrap();
        }
        Err(_) => {}
    };
}

pub 
const DATASET: [(&str, &str, &str); 24] = [
    (
        "maven",
        "apache/maven",
        "be2b7f890d98af20eb0753650b6605a68a97ac05",
    ),
    (
        "spoon",
        "INRIA/spoon",
        "56e12a0c0e0e69ea70863011b4f4ca3305e0542b",
    ),
    (
        "quarkus",
        "quarkusio/quarkus",
        "5ac8332061fbbd4f11d5f280ff12b65fe7308540",
    ),
    (
        "logging-log4j2",
        "apache/logging-log4j2",
        "ebfc8945a5dd77b617f4667647ed4b740323acc8",
    ),
    (
        "javaparser",
        "javaparser/javaparser",
        "046bf8be251189452ad6b25bf9107a1a2167ce6f",
    ),
    (
        "spark",
        "apache/spark",
        "885f4733c413bdbb110946361247fbbd19f6bba9",
    ),
    (
        "gson",
        "google/gson",
        "f79ea208b1a42d0ee9e921dcfb3694221a2037ed",
    ),
    (
        "junit4",
        "junit-team/junit4",
        "cc7c500584fcb85eaf98c568b7441ceac6dd335c",
    ),
    (
        "jenkins",
        "jenkinsci/jenkins",
        "be6713661c120c222c17026e62401191bdc4035c",
    ),
    (
        "dubbo",
        "apache/dubbo",
        "e831b464837ae5d2afac9841559420aeaef6c52b",
    ),
    (
        "skywalking",
        "apache/skywalking",
        "38a9d4701730e674c9646173dbffc1173623cf24",
    ),
    (
        "flink",
        "apache/flink",
        "d67338a140bf1b744d95a514b82824bba5b16105",
    ),
    (
        "aws-sdk-java",
        "aws/aws-sdk-java",
        "0b01b6c8139e050b36ef79418986cdd8d9704998",
    ),
    (
        "aws-sdk-java-v2",
        "aws/aws-sdk-java-v2",
        "edea5de18755962cb864cb4c88652ec8748d877c",
    ),
    (
        "aws-toolkit-eclipse",
        "aws/aws-toolkit-eclipse",
        "85417f68e1eb6d90d46e145229e390cf55a4a554",
    ),
    (
        "netty",
        "netty/netty",
        "c2b846750dd2131d65aa25c8cf66bf3649b248f9",
    ),
    (
        "fastjson",
        "alibaba/fastjson",
        "f56b5d895f97f4cc3bd787c600a3ee67ba56d4db",
    ),
    (
        "arthas",
        "alibaba/arthas",
        "c661d2d24892ce8a09a783ca3ba82eda90a66a85",
    ),
    (
        "guava",
        "google/guava",
        "b30a7120f901b4a367b8a9839a8b8ba62457fbdf",
    ),
    (
        "hadoop",
        "apache/hadoop",
        "d5e97fe4d6baf43a5576cbd1700c22b788dba01e",
    ),
    (
        "jackson-core",
        "FasterXML/jackson-core",
        "3cb5ce818e476d5b0b504b1833c7d33be80e9ca4",
    ),
    (
        "slf4j",
        "qos-ch/slf4j",
        "2b0e15874aaf5502c9d6e36b0b81fc6bc14a8531",
    ),
    (
        "jacoco",
        "jacoco/jacoco",
        "62a2b556c26f0f42a2ae791a86dc39dd36d35392",
    ),
    (
        "graphhopper",
        "graphhopper/graphhopper",
        "90acd4972610ded0f1581143f043eb4653a4c691",
    ),
];


pub mod diffing {
    use hyper_ast::types::{self, HyperAST};
    use hyper_diff::{
        decompressed_tree_store::{lazy_post_order::LazyPostOrder, ShallowDecompressedTreeStore},
        matchers::{
            heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
            mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore},
            Mapper, Mapping,
        },
    };
    use std::fmt::Debug;

    fn _top_down<'store, HAST: HyperAST<'store>>(
        mapper: &mut Mapper<
            'store,
            HAST,
            &mut LazyPostOrder<HAST::T, u32>,
            &mut LazyPostOrder<HAST::T, u32>,
            VecStore<u32>,
        >,
    ) where
        HAST::IdN: Clone + Debug + Eq,
        HAST::Label: Clone + Copy + Eq + Debug,
        <HAST::T as types::WithChildren>::ChildIdx: Debug,
        HAST::T: 'store + types::WithHashs + types::WithStats,
    {
        let mm = LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::compute_multi_mapping::<
            DefaultMultiMappingStore<_>,
        >(mapper);
        LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::filter_mappings(mapper, &mm);
    }

    pub fn top_down<'store, 'a, HAST: HyperAST<'store>>(
        hyperast: &'store HAST,
        src_arena: &'a mut LazyPostOrder<HAST::T, u32>,
        dst_arena: &'a mut LazyPostOrder<HAST::T, u32>,
    ) -> Mapper<
        'store,
        HAST,
        &'a mut LazyPostOrder<HAST::T, u32>,
        &'a mut LazyPostOrder<HAST::T, u32>,
        VecStore<u32>,
    >
    where
        HAST::IdN: Clone + Debug + Eq,
        HAST::Label: Clone + Copy + Eq + Debug,
        <HAST::T as types::WithChildren>::ChildIdx: Debug,
        HAST::T: 'store + types::WithHashs + types::WithStats,
    {
        let mappings = VecStore::<u32>::default();
        let mut mapper = Mapper {
            hyperast,
            mapping: Mapping {
                src_arena,
                dst_arena,
                mappings,
            },
        };
        mapper.mapping.mappings.topit(
            mapper.mapping.src_arena.len(),
            mapper.mapping.dst_arena.len(),
        );
        _top_down(&mut mapper);
        mapper
    }
}
