// window of one is just consecutive commits

use hyper_ast_cvs_git::multi_preprocessed::PreProcessedRepositories;
use std::{io::Write, path::PathBuf, str::FromStr};

use hyper_ast_benchmark_diffs::cross_repo::{windowed_commits_compare, CommitCompareParameters};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
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

    // let params = vec![
    //     CommitCompareParameters {
    //         name: "apache/logging-log4j2",
    //         before: "", //"7e745b42bda9bf6f8ea681d38992d18036fc021e",
    //         after: "ebfc8945a5dd77b617f4667647ed4b740323acc8",
    //         dir_path: "",
    //     },
    //     CommitCompareParameters {
    //         name: "INRIA/spoon",
    //         before: "", //"76ffd3353a535b0ce6edf0bf961a05236a40d3a1",
    //         after: "74ee133f4fe25d8606e0775ade577cd8e8b5cbfd",
    //         dir_path: "",
    //     },
    //     CommitCompareParameters {
    //         name: "apache/spark",
    //         before: "", //"14211a19f53bd0f413396582c8970e3e0a74281d",
    //         after: "885f4733c413bdbb110946361247fbbd19f6bba9",
    //         dir_path: "",
    //     },
    // ];

    let args: Vec<String> = std::env::args().collect();
    log::warn!("args: {:?}", args);

    let out_validity = args.get(1).and_then(|x| {
        if x.is_empty() {
            None
        } else {
            Some(PathBuf::from_str(x).unwrap())
        }
    });
    let out_perfs = args.get(2).and_then(|x| {
        if x.is_empty() {
            None
        } else {
            Some(PathBuf::from_str(x).unwrap())
        }
    });
    let out = out_validity.zip(out_perfs);

    let diff_algorithm = args.get(3).expect("the diff algo for java gumtree eg. Chawathe or None if it takes too long");

    let wanted = &args[4..];

    let params = DATASET
        .iter()
        .filter_map(|(short, name, after)| {
            if !wanted.iter().any(|x| x == short) {
                return None
            }
            Some(CommitCompareParameters {
                name,
                before: "",
                after,
                dir_path: "",
            })
        })
        .collect();

    let limit = 1000;
    let preprocessed = PreProcessedRepositories::default();
    windowed_commits_compare(2, preprocessed, params, diff_algorithm, limit, out);
}

const DATASET: [(&str, &str, &str); 23] = [
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
];
