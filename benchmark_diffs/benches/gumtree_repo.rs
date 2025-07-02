use criterion::measurement::Measurement;
use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
};
use hyper_diff::algorithms;
use hyperast::store::defaults::NodeIdentifier;
use hyperast::types::WithStats;
use hyperast::types::{self, HyperAST, NodeId};
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;
use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub enum GumtreeVariant {
    Greedy,
    Stable,
    GreedyLazy,
    StableLazy,
}

impl std::fmt::Display for GumtreeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GumtreeVariant::Greedy => write!(f, "Greedy"),
            GumtreeVariant::Stable => write!(f, "Stable"),
            GumtreeVariant::GreedyLazy => write!(f, "Lazy Greedy"),
            GumtreeVariant::StableLazy => write!(f, "Lazy Stable"),
        }
    }
}

impl GumtreeVariant {
    pub fn variants() -> Vec<Self> {
        vec![
            Self::Greedy,
            Self::Stable,
            Self::GreedyLazy,
            Self::StableLazy,
        ]
    }
}

fn diff_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("bottom_up_repo");
    struct Input {
        repo: hyperast_vcs_git::git::Repo,
        commit: &'static str,
        config: hyperast_vcs_git::processing::RepoConfig,
        fetch: bool,
    }
    // NOTE no good way of selecting them to avoid preparing, so comment and uncomment inputs when needed
    let inputs: &[Input] = &[
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("chromium", "chromium"),
        //     commit: "f461f9752e5918c5c87f2e3767bcb24945ee0fa0",
        //     config: hyperast_vcs_git::processing::RepoConfig::CppMake,
        //     fetch: false,
        // },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "maven"),
            commit: "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("INRIA", "spoon"),
            commit: "56e12a0c0e0e69ea70863011b4f4ca3305e0542b",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("quarkusio", "quarkus"),
        //     commit: "5ac8332061fbbd4f11d5f280ff12b65fe7308540",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "logging-log4j2"),
        //     commit: "ebfc8945a5dd77b617f4667647ed4b740323acc8",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("javaparser", "javaparser"),
        //     commit: "046bf8be251189452ad6b25bf9107a1a2167ce6f",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "spark"),
        //     commit: "885f4733c413bdbb110946361247fbbd19f6bba9",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("google", "gson"),
        //     commit: "f79ea208b1a42d0ee9e921dcfb3694221a2037ed",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // // Input {
        // //     repo: hyperast_vcs_git::git::Forge::Github.repo("junit-team", "junit4"),
        // //     commit: "cc7c500584fcb85eaf98c568b7441ceac6dd335c",
        // //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        // //     fetch: true,
        // // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("jenkinsci", "jenkins"),
        //     commit: "be6713661c120c222c17026e62401191bdc4035c",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "dubbo"),
        //     commit: "e831b464837ae5d2afac9841559420aeaef6c52b",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // // Input {
        // //     repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "skywalking"),
        // //     commit: "38a9d4701730e674c9646173dbffc1173623cf24",
        // //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        // //     fetch: true,
        // // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "flink"),
        //     commit: "d67338a140bf1b744d95a514b82824bba5b16105",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // // Input {
        // //     repo: hyperast_vcs_git::git::Forge::Github.repo("aws", "aws-sdk-java"),
        // //     commit: "0b01b6c8139e050b36ef79418986cdd8d9704998",
        // //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        // //     fetch: true,
        // // },
        // // Input {
        // //     repo: hyperast_vcs_git::git::Forge::Github.repo("aws", "aws-sdk-java-v2"),
        // //     commit: "edea5de18755962cb864cb4c88652ec8748d877c",
        // //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        // //     fetch: true,
        // // },
        // // Input {
        // //     repo: hyperast_vcs_git::git::Forge::Github.repo("aws", "aws-toolkit-eclipse"),
        // //     commit: "85417f68e1eb6d90d46e145229e390cf55a4a554",
        // //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        // //     fetch: true,
        // // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("netty", "netty"),
        //     commit: "c2b846750dd2131d65aa25c8cf66bf3649b248f9",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("alibaba", "fastjson"),
        //     commit: "f56b5d895f97f4cc3bd787c600a3ee67ba56d4db",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // // Input {
        // //     repo: hyperast_vcs_git::git::Forge::Github.repo("alibaba", "arthas"),
        // //     commit: "c661d2d24892ce8a09a783ca3ba82eda90a66a85",
        // //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        // //     fetch: true,
        // // },
        // // Input {
        // //     repo: hyperast_vcs_git::git::Forge::Github.repo("google", "guava"),
        // //     commit: "b30a7120f901b4a367b8a9839a8b8ba62457fbdf",
        // //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        // //     fetch: true,
        // // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "hadoop"),
        //     commit: "d5e97fe4d6baf43a5576cbd1700c22b788dba01e",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("FasterXML", "jackson-core"),
        //     commit: "3cb5ce818e476d5b0b504b1833c7d33be80e9ca4",
        //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        //     fetch: true,
        // },
        // // Input {
        // //     repo: hyperast_vcs_git::git::Forge::Github.repo("qos-ch", "slf4j"),
        // //     commit: "2b0e15874aaf5502c9d6e36b0b81fc6bc14a8531",
        // //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        // //     fetch: true,
        // // },
        // // Input {
        // //     repo: hyperast_vcs_git::git::Forge::Github.repo("jacoco", "jacoco"),
        // //     commit: "62a2b556c26f0f42a2ae791a86dc39dd36d35392",
        // //     config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        // //     fetch: true,
        // // },
    ];
    let mut repositories = PreProcessedRepositories::default();

    let inputs = inputs
        .into_iter()
        .map(|p| {
            repositories.register_config(p.repo.clone(), p.config);
            let repo = repositories
                .get_config((&p.repo).clone())
                .ok_or_else(|| "missing config for repository".to_string())
                .unwrap();
            let repository = if p.fetch {
                repo.fetch()
            } else {
                repo.nofetch()
            };
            (repository, p.commit)
        })
        .collect::<Vec<_>>();
    eprintln!("fetched repositories");
    diff_benchmark_project(&mut group, &mut repositories, &inputs);
    group.finish();
}

fn diff_benchmark_project(
    group: &mut BenchmarkGroup<impl Measurement>,
    repositories: &mut PreProcessedRepositories,
    inputs: &[(hyperast_vcs_git::processing::ConfiguredRepo2, &str)],
) {
    for (repository, commit) in inputs {
        let commits = repositories
            .pre_process_with_limit(repository, "", commit, 2)
            .unwrap();
        let src = repositories
            .get_commit(&repository.config, &commits[1])
            .unwrap()
            .ast_root;
        let dst = repositories
            .get_commit(&repository.config, &commits[0])
            .unwrap()
            .ast_root;
        let hyperast = &repositories.processor.main_stores;
        group.throughput(Throughput::Elements(
            (hyperast.node_store().resolve(src).size() + hyperast.node_store().resolve(dst).size())
                .div_ceil(2) as u64,
        ));
        let stores = hyperast_vcs_git::no_space::as_nospaces2(&repositories.processor.main_stores);
        for variant in GumtreeVariant::variants() {
            let file_name = repository.spec.name();
            group.bench_with_input(
                BenchmarkId::new(variant.to_string(), file_name),
                &(src, dst),
                |b, (src, dst)| {
                    b.iter(|| {
                        run(&stores, *src, *dst, variant);
                    });
                },
            );
        }
    }
}

pub fn run<HAST: HyperAST<IdN = NodeIdentifier> + Copy>(
    stores: HAST,
    src: NodeIdentifier,
    dst: NodeIdentifier,
    variant: GumtreeVariant,
) where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let diff = match variant {
        GumtreeVariant::Greedy => algorithms::gumtree::diff,
        GumtreeVariant::Stable => algorithms::gumtree_stable::diff,
        GumtreeVariant::GreedyLazy => algorithms::gumtree_lazy::diff,
        GumtreeVariant::StableLazy => algorithms::gumtree_stable_lazy::diff,
    };

    diff(stores, &src, &dst);
}

criterion_group!(
    name = benches;
    config = Criterion::default().configure_from_args()
        .measurement_time(Duration::from_secs(10))
        .sample_size(10);
    targets = diff_benchmark
);
criterion_main!(benches);
