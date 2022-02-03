#![feature(test)]

use std::{
    collections::{BTreeMap, HashMap},
    env, fmt, fs,
    io::{stdout, Write},
    path::{Path, PathBuf},
};

use git2::{ObjectType, Oid, RemoteCallbacks, Repository, Revwalk, TreeEntry};
use rusted_gumtree_core::tree::tree::LabelStore as _;
use rusted_gumtree_core::tree::tree::Type;
use rusted_gumtree_gen_ts_java::{
    filter::BloomSize,
    full::FullNode,
    hashed::{self, SyntaxNodeHashs},
    impact::{
        declaration::ExplorableDecl,
        element::{ExplorableRef, RefsEnum},
        partial_analysis::PartialAnalysis,
        usage,
    },
    java_tree_gen_full_compress_legion_ref::CS,
    tree_gen::TreeGen,
    utils::memusage_linux,
};
extern crate test;

use rusted_gumtree_gen_ts_java::java_tree_gen_full_compress_legion_ref as java_tree_gen;

pub fn get_up_to_date_repo(path: &String, mut fo: git2::FetchOptions, url: &String) -> Repository {
    if Path::new(path).join(".git").exists() {
        let repository = match Repository::open(path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };
        println!("fetch: {}", &path);
        repository
            .find_remote("origin")
            .unwrap()
            .fetch(&["main"], Some(&mut fo), None)
            .unwrap_or_else(|e| println!("{}", e));

        repository
    } else if Path::new(path).exists() {
        todo!()
    } else {
        let mut builder = git2::build::RepoBuilder::new();

        builder.bare(true);

        builder.fetch_options(fo);

        println!("clone: {}", &path);
        let repository = match builder.clone(url, Path::new(path).join(".git").as_path()) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };
        repository
    }
}

pub struct PreProcessedRepository {
    pub object_map: BTreeMap<git2::Oid, (java_tree_gen::NodeIdentifier, MD)>,
    pub hyper_ast: java_tree_gen::JavaTreeGen,
    pub commits: HashMap<git2::Oid, Commit>,
}

impl PreProcessedRepository {
    pub fn new(
        repository: &mut Repository,
        before: &str,
        after: &str,
        dir_path: &str,
    ) -> PreProcessedRepository {
        let rw = all_commits_between(&repository, before, after);
        use java_tree_gen::{JavaTreeGen, SimpleStores};
        let mut full_nodes: BTreeMap<git2::Oid, (java_tree_gen::NodeIdentifier, MD)> =
            BTreeMap::default();
        let mut java_tree_gen = JavaTreeGen {
            line_break: "\n".as_bytes().to_vec(),
            stores: SimpleStores::default(),
        };
        let commits: HashMap<git2::Oid, Commit> = rw
            .take(2)
            .map(|oid| {
                let oid = oid.unwrap();
                (
                    oid.clone(),
                    handle_commit(
                        &repository,
                        &mut java_tree_gen,
                        &mut full_nodes,
                        dir_path,
                        oid,
                    ),
                )
            })
            .collect();
        PreProcessedRepository {
            object_map: full_nodes,
            hyper_ast: java_tree_gen,
            commits,
        }
    }
    pub fn first(before: &str, after: &str) -> Diffs {
        todo!()
    }

    pub fn compute_diff(before: &str, after: &str) -> Diffs {
        todo!()
    }

    pub fn compute_impacts(diff: &Diffs) -> Impacts {
        todo!()
    }

    pub fn find_declaration(reff: ExplorableRef) {
        todo!()
    }

    pub fn find_references(decl: ExplorableDecl) {
        todo!()
    }
}

pub struct Diffs();
pub struct Impacts();

pub struct Commit {
    meta_data: (PartialAnalysis,),
    parents: Vec<git2::Oid>,
    pub ast_root: java_tree_gen::NodeIdentifier,
}

fn handle_commit(
    repository: &Repository,
    java_tree_gen: &mut java_tree_gen::JavaTreeGen,
    full_nodes: &mut BTreeMap<Oid, (java_tree_gen::NodeIdentifier, MD)>,
    dir_path: &str,
    commit_oid: git2::Oid,
) -> Commit {
    use java_tree_gen::{hash32, EntryR, NodeIdentifier, NodeStore, SubTreeMetrics};
    let dir_path = PathBuf::from(dir_path);
    let mut dir_path = dir_path.components().peekable();

    let dir_hash = hash32(&Type::Directory);
    let commit = repository.find_commit(commit_oid).unwrap();
    let tree = commit.tree().unwrap();
    let ast_root;
    let meta_data;
    {
        let id = tree.id();
        let mut stack: Vec<(Oid, Vec<E>, Acc)> = vec![(
            id,
            tree.iter().rev().map(Into::into).collect(),
            Acc::new("".to_string()),
        )];
        loop {
            if let Some(current_dir) = stack.last_mut().expect("never empty").1.pop() {
                match current_dir {
                    E::Tree(x, name) => {
                        if let Some(s) = dir_path.peek() {
                            if name.eq(std::os::unix::prelude::OsStrExt::as_bytes(s.as_os_str())) {
                                dir_path.next();
                                stack.last_mut().expect("never empty").1.clear();
                            } else {
                                continue;
                            }
                        }
                        if let Some(already) = full_nodes.get(&x) {
                            // reinit already computed node for post order
                            let full_node = (
                                already.0.clone(),
                                already.1.metrics.clone(),
                                already.1.ana.clone(),
                            );

                            if stack.is_empty() {
                                ast_root = already.0.clone();
                                meta_data = (full_node.2,);
                                break;
                            } else {
                                let w = &mut stack.last_mut().unwrap().2;
                                w.push_dir(full_node);
                            }
                            continue;
                        }
                        // TODO use maven pom.xml to find source_dir  and tests_dir ie. ignore resources, maybe also tests
                        println!("tree {:?}", std::str::from_utf8(&name));
                        let a = repository.find_tree(x).unwrap();
                        stack.push((
                            x,
                            a.iter().rev().map(Into::into).collect(),
                            Acc::new(std::str::from_utf8(&name).unwrap().to_string()),
                        ));
                    }
                    E::Blob(x, name) => {
                        if dir_path.peek().is_some() {
                            continue;
                        } else if !name.ends_with(b".java") {
                            continue;
                        } else if let Some(already) = full_nodes.get(&x) {
                            // TODO reinit already computed node for post order
                            let full_node = (
                                already.0.clone(),
                                already.1.metrics.clone(),
                                already.1.ana.clone(),
                            );

                            if stack.is_empty() {
                                ast_root = already.0.clone();
                                meta_data = (full_node.2,);
                                break;
                            } else {
                                let w = &mut stack.last_mut().unwrap().2;
                                w.push_dir(full_node);
                            }
                            continue;
                        }
                        println!("blob {:?}", std::str::from_utf8(&name));
                        let a = repository.find_blob(x).unwrap();
                        if let Ok(z) = std::str::from_utf8(a.content()) {
                            println!("content: {}", z);

                            use tree_sitter::{Language, Parser};

                            let mut parser = Parser::new();

                            extern "C" {
                                fn tree_sitter_java() -> Language;
                            }
                            {
                                let language = unsafe { tree_sitter_java() };
                                parser.set_language(language).unwrap();
                            }

                            let tree = parser.parse(a.content(), None).unwrap();
                            if tree.root_node().has_error() {
                                println!("bad CST");
                                println!("{}", z);
                                println!("{}", tree.root_node().to_sexp());
                                // {
                                //     let mut fe = PathBuf::new();
                                //     fe.extend(&[
                                //         "/home/quentin/resources/file_error",
                                //         repo_name,
                                //         &oid.to_string(),
                                //         x,
                                //     ]);
                                //     std::fs::create_dir_all(&fe).unwrap();
                                //     fe.extend(&[&y.name().unwrap()]);
                                //     let mut fe = fs::File::create(&fe).unwrap();
                                //     fe.write(a.content()).unwrap();

                                //     let mut fe = PathBuf::new();
                                //     fe.extend(&[
                                //         "/home/quentin/resources/tree_error",
                                //         repo_name,
                                //         &oid.to_string(),
                                //         x,
                                //     ]);
                                //     std::fs::create_dir_all(&fe).unwrap();
                                //     fe.extend(&[&y.name().unwrap()]);
                                //     let mut fe = fs::File::create(&fe).unwrap();
                                //     fe.write(tree.root_node().to_sexp().as_bytes()).unwrap();
                                // }
                                // panic!("do not handle bad CSTs")
                                continue;
                            }
                            let full_node =
                                java_tree_gen.generate_file(&name, a.content(), tree.walk());

                            let w = &mut stack.last_mut().unwrap().2;

                            full_nodes.insert(
                                a.id(),
                                (
                                    full_node.local().compressed_node.clone(),
                                    MD {
                                        metrics: full_node.local().metrics.clone(),
                                        ana: full_node.local().ana.clone().unwrap(),
                                    },
                                ),
                            );
                            w.push(full_node);
                        }
                    }
                }
            } else if let Some((id, _, acc)) = stack.pop() {
                // commit node
                let hashed_label = hash32(&Type::Directory);

                let hsyntax = hashed::inner_node_hash(
                    &dir_hash,
                    &0,
                    &acc.metrics.size,
                    &acc.metrics.hashs.syntax,
                );
                let label = java_tree_gen
                    .stores()
                    .label_store
                    .get_or_insert(acc.name.clone());

                let eq = |x: EntryR| {
                    let t = x.get_component::<Type>().ok();
                    if &t != &Some(&Type::Directory) {
                        return false;
                    }
                    let l = x.get_component::<java_tree_gen::LabelIdentifier>().ok();
                    if l != Some(&label) {
                        return false;
                    } else {
                        let cs = x.get_component::<Vec<NodeIdentifier>>().ok();
                        let r = cs == Some(&acc.children);
                        if !r {
                            return false;
                        }
                    }
                    true
                };
                let hashs = SyntaxNodeHashs {
                    structt: hashed::inner_node_hash(
                        &dir_hash,
                        &0,
                        &acc.metrics.size,
                        &acc.metrics.hashs.structt,
                    ),
                    label: hashed::inner_node_hash(
                        &dir_hash,
                        &hashed_label,
                        &acc.metrics.size,
                        &acc.metrics.hashs.label,
                    ),
                    syntax: hsyntax,
                };
                let ana = {
                    let ana = acc.ana;
                    println!("refs in directory");
                    println!("ref count in dir {}", ana.refs_count());
                    ana.print_refs(&java_tree_gen.stores().label_store);
                    println!("decls in directory");
                    ana.print_decls(&java_tree_gen.stores().label_store);
                    ana.resolve()
                };
                println!("ref count in dir after resolver {}", ana.refs_count());
                println!("refs in directory after resolve");
                ana.print_refs(&java_tree_gen.stores().label_store);
                let insertion = java_tree_gen
                    .stores()
                    .node_store
                    .prepare_insertion(&hsyntax, eq);
                let node_id = if let Some(id) = insertion.occupied_id() {
                    id
                } else {
                    let vacant = insertion.vacant();
                    NodeStore::insert_after_prepare(
                        vacant,
                        (
                            Type::Directory,
                            label,
                            hashs,
                            CS(acc.children),
                            BloomSize::Much,
                        ),
                    )
                };

                let metrics = SubTreeMetrics {
                    size: acc.metrics.size + 1,
                    height: acc.metrics.height + 1,
                    hashs,
                };

                let full_node = (node_id.clone(), metrics, ana.clone());

                full_nodes.insert(
                    id,
                    (
                        node_id.clone(),
                        MD {
                            metrics: metrics,
                            ana,
                        },
                    ),
                );

                if stack.is_empty() {
                    ast_root = node_id.clone();
                    meta_data = (full_node.2,);
                    break;
                } else {
                    let w = &mut stack.last_mut().unwrap().2;
                    w.push_dir(full_node);
                    println!("dir: {}", &acc.name);
                }
            } else {
                panic!("never empty")
            }
        }
    };
    Commit {
        meta_data,
        parents: commit.parents().into_iter().map(|x| x.id()).collect(),
        ast_root,
    }
}

pub struct MD {
    metrics: java_tree_gen::SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: PartialAnalysis,
}

pub struct Acc {
    name: String,
    children: Vec<java_tree_gen::NodeIdentifier>,
    // simple: BasicAccumulator<Type, NodeIdentifier>,
    metrics: java_tree_gen::SubTreeMetrics<SyntaxNodeHashs<u32>>,
    ana: PartialAnalysis,
    // padding_start: usize,
    // indentation: Spaces,
}

impl Acc {
    fn new(name: String) -> Self {
        Self {
            name,
            children: Default::default(),
            // simple: BasicAccumulator::new(kind),
            metrics: Default::default(),
            ana: PartialAnalysis::init(&Type::Directory, None, |x| panic!()),
        }
    }
}

impl Acc {
    pub(crate) fn push(
        &mut self,
        full_node: FullNode<java_tree_gen::Global, java_tree_gen::Local>,
    ) {
        self.children.push(full_node.local.compressed_node.clone());
        self.metrics.acc(full_node.local.metrics);
        full_node
            .local
            .ana
            .unwrap()
            .acc(&Type::Directory, &mut self.ana);
    }
    pub(crate) fn push_dir(
        &mut self,
        full_node: (
            java_tree_gen::NodeIdentifier,
            java_tree_gen::SubTreeMetrics<SyntaxNodeHashs<u32>>,
            PartialAnalysis,
        ),
    ) {
        self.children.push(full_node.0);
        self.metrics.acc(full_node.1);
        full_node.2.acc(&Type::Directory, &mut self.ana);
    }
}

enum E {
    Blob(Oid, Vec<u8>),
    Tree(Oid, Vec<u8>),
}

impl<'a> From<TreeEntry<'a>> for E {
    fn from(x: TreeEntry<'a>) -> Self {
        if x.kind().unwrap().eq(&ObjectType::Tree) {
            Self::Tree(x.id(), x.name_bytes().to_owned())
        } else if x.kind().unwrap().eq(&ObjectType::Blob) {
            Self::Blob(x.id(), x.name_bytes().to_owned())
        } else {
            panic!()
        }
    }
}

fn spoon_filter(x: &str) -> bool {
    !(x.contains("src/test/resources/") || x.ends_with("src/test/resources"))
}

fn all_commits_between<'a>(
    repository: &'a Repository,
    before: &'a str,
    after: &'a str,
) -> Revwalk<'a> {
    use git2::*;
    let mut rw = repository.revwalk().unwrap();
    if !before.is_empty() {
        // rw.hide_ref(before).unwrap();
        println!("{}", before);
        let c = repository.find_reference(before).unwrap();
        let c = c.peel_to_commit().unwrap();
        println!("{:?}", c);
        for c in c.parents() {
            rw.hide(c.id()).unwrap();
        }
    }
    if after.is_empty() {
        rw.push_head().unwrap();
    } else {
        rw.push_ref(after).unwrap();
    }
    rw.set_sorting(Sort::TOPOLOGICAL).unwrap();
    rw
}

fn all_commits_from_head(repository: &Repository) -> Revwalk {
    use git2::*;
    // let REMOTE_REFS_PREFIX = "refs/remotes/origin/";
    // let branch: Option<&str> = None;
    // let currentRemoteRefs:Vec<Object> = vec![];
    let mut rw = repository.revwalk().unwrap();
    rw.push_head().unwrap();
    rw.set_sorting(Sort::TOPOLOGICAL).unwrap();
    rw
    // Revwalk::
    // for reff in repository.references().expect("") {
    //     let reff = reff.unwrap();
    // 	let refName = reff.name().unwrap();
    // 	if refName.starts_with(REMOTE_REFS_PREFIX) {
    // 		if branch.is_none() || refName.ends_with(&("/".to_owned() + branch.unwrap())) {
    // 			currentRemoteRefs.push(reff.);
    // 		}
    // 	}
    // }

    // RevWalk walk = new RevWalk(repository);
    // for (ObjectId newRef : currentRemoteRefs) {
    // 	walk.markStart(walk.parseCommit(newRef));
    // }
    // walk.setRevFilter(commitsFilter);
    // return walk;
}

#[cfg(test)]
mod tests {
    use rusted_gumtree_gen_ts_java::impact::element::{IdentifierFormat, LabelPtr};

    use super::*;

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
        let url = &format!("{}{}", "https://github.com/", repo_name);
        let path = &format!("{}{}", "/home/quentin/resources/repo/", repo_name);
        let mut callbacks = RemoteCallbacks::new();

        callbacks.transfer_progress(|x| {
            println!("transfer {}/{}", x.received_objects(), x.total_objects());
            true
        });

        let mut fo = git2::FetchOptions::new();

        fo.remote_callbacks(callbacks);

        let mut repository = get_up_to_date_repo(path, fo, url);

        find_refs_from_canonical_type(&mut repository, repo_name, before, after, dir_path);
    }

    pub fn find_refs_from_canonical_type(
        repository: &mut Repository,
        repo_name: &str,
        before: &str,
        after: &str,
        dir_path: &str,
    ) {
        // let PreProcessedRepository {
        //     object_map: full_nodes,
        //     hyper_ast: mut java_tree_gen,
        //     mut commits,
        // }
        let mut preprocessed = PreProcessedRepository::new(repository, before, after, dir_path);

        {
            let mut ana = PartialAnalysis::default(); //&mut commits[0].meta_data.0;

            macro_rules! scoped {
                ( $o:expr, $i:expr ) => {{
                    let o = $o;
                    let i = $i;
                    let f = IdentifierFormat::from(i);
                    let i = preprocessed.hyper_ast.stores.label_store.get_or_insert(i);
                    let i = LabelPtr::new(i,f);
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
            // let i = scoped!(scoped!(scoped!(scoped!(i,"java"),"nio"),"file"),"Path");
            ana.print_refs(&preprocessed.hyper_ast.stores.label_store);

            // println!("{}", java_tree_gen.stores.label_store);

            let root = preprocessed
                .commits
                .get(&repository.refname_to_id(before).unwrap())
                .unwrap()
                .ast_root;
            usage::find_refs(&preprocessed.hyper_ast, &mut ana, i, root);
        }

        let mu = memusage_linux();
        // drop(java_tree_gen);
        // drop(full_nodes);
        // drop(commits);
        drop(preprocessed);
        let mu = mu - memusage_linux();
        println!("memory used {}", mu);
    }
}
