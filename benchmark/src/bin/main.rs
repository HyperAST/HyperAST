#![feature(test)]

use std::{
    collections::BTreeMap,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

use git2::{ObjectType, RemoteCallbacks, Repository, Revwalk};
use rusted_gumtree_gen_ts_java::utils::memusage_linux;

extern crate test;

use test::Bencher;

#[test]
fn all() {
    use std::fs::read_to_string;
    use std::path::PathBuf;

    use pommes::Project;

    let path: PathBuf = Path::new("pom.xml").to_path_buf();
    println!("path: {}", &path.display());

    let contents = read_to_string(path).unwrap();
    let _parsed: Project = serde_xml_rs::from_str(&contents).unwrap();

    println!("{:#?}", _parsed);
}

fn main() {
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

    bench_1_aux(&mut repository, repo_name, before, after, dir_path);
}

#[bench]
fn bench_1(bencher: &mut Bencher) {
    let repo_name = "INRIA/spoon";
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
    bencher.iter(|| bench_1_aux(&mut repository, repo_name, "", "", ""));
}

fn bench_1_aux(
    repository: &mut Repository,
    repo_name: &str,
    before: &str,
    after: &str,
    dir_path: &str,
) {
    let rw = all_commits_between(&repository, before, after);
    let mut i: u32 = 0;

    // let mut commits_full_nodes = vec![];
    let mut full_nodes: BTreeMap<git2::Oid, _> = BTreeMap::default();
    use rusted_gumtree_gen_ts_java::java_tree_gen_full_compress_legion_ref::{
        JavaTreeGen, SimpleStores,
    };
    let mut java_tree_gen = JavaTreeGen {
        line_break: "\n".as_bytes().to_vec(),
        stores: SimpleStores::default(),
    };
    for oid in rw {
        if i >= 1000 {
            break;
        }
        i += 1;
        let oid = oid.unwrap();
        let commit = repository.find_commit(oid).unwrap();
        // println!(
        //     "{} {:?}",
        //     &oid,
        //     &commit.parent_ids().into_iter().collect::<Vec<_>>(),
        //     // commit.summary().unwrap_or("")
        // );
        let tree = commit.tree().unwrap();
        tree.walk(git2::TreeWalkMode::PostOrder, |x, y| {
            if y.kind().unwrap().eq(&ObjectType::Blob)
                && y.name().unwrap_or("").ends_with(".java")
                && if repo_name == "INRIA/spoon" {
                    spoon_filter(x)
                } else {
                    x.starts_with(dir_path)
                }
            {
                let a = y.to_object(&repository).unwrap();
                let a = a.as_blob().unwrap();
                if full_nodes.contains_key(&a.id()) {
                } else if let Ok(z) = std::str::from_utf8(a.content()) {
                    println!("{}{}", x, y.name().unwrap_or(""));
                    // println!("content: {}", z);

                    use tree_sitter::{Language, Parser};

                    let mut parser = Parser::new();

                    extern "C" {
                        fn tree_sitter_java() -> Language;
                    }
                    // {
                    //     pub type TSSymbol = u16;

                    //     #[repr(C)]
                    //     #[derive(Debug, Copy, Clone)]
                    //     pub struct TSLanguage {
                    //         _unused: [u8; 0],
                    //     }
                    //     extern "C" {
                    //         pub fn ts_language_alias_at(language: *const TSLanguage, production_id: u32, child_index: u32) -> TSSymbol;
                    //     }
                    //     extern "C" {
                    //         pub fn tree_sitter_java() -> *const TSLanguage;
                    //     }
                    //     let t = unsafe { ts_language_alias_at(tree_sitter_java(), 0,0) };
                    // };
                    {
                        let language = unsafe { tree_sitter_java() };
                        parser.set_language(language).unwrap();
                        // fn ts_language_alias_at(
                        //     const TSLanguage *self,
                        //     uint32_t production_id,
                        //     uint32_t child_index
                        //   )
                    }

                    // let mut parser: Parser, old_tree: Option<&Tree>
                    let tree = parser.parse(a.content(), None).unwrap();
                    // let mut acc_stack = vec![Accumulator::new(java_tree_gen.stores.type_store.get("file"))];

                    println!("tree: {}", tree.root_node().to_sexp());

                    if tree.root_node().has_error() {
                        println!(
                            "{}{}{}{}",
                            x,
                            x.contains("/src/test/resources/"),
                            x.ends_with("/src/test/resources"),
                            !(x.contains("/src/test/resources/")
                                || x.ends_with("/src/test/resources"))
                        );
                        {
                            let mut fe = PathBuf::new();
                            fe.extend(&[
                                "/home/quentin/resources/file_error",
                                repo_name,
                                &oid.to_string(),
                                x,
                            ]);
                            std::fs::create_dir_all(&fe).unwrap();
                            fe.extend(&[&y.name().unwrap()]);
                            let mut fe = fs::File::create(&fe).unwrap();
                            fe.write(a.content()).unwrap();

                            let mut fe = PathBuf::new();
                            fe.extend(&[
                                "/home/quentin/resources/tree_error",
                                repo_name,
                                &oid.to_string(),
                                x,
                            ]);
                            std::fs::create_dir_all(&fe).unwrap();
                            fe.extend(&[&y.name().unwrap()]);
                            let mut fe = fs::File::create(&fe).unwrap();
                            fe.write(tree.root_node().to_sexp().as_bytes()).unwrap();
                        }
                        panic!();
                    }

                    // println!("{} {} {}",full_nodes.len(),a.id(),java_tree_gen.stores.node_store.len());
                    let full_node = java_tree_gen.generate_default(a.content(), tree.walk());
                    full_nodes.insert(a.id(), full_node);
                    // println!(
                    //     "commit: {} size: {} {:?}",
                    //     i,
                    //     full_nodes.len(),
                    //     &java_tree_gen.stores.node_store
                    // )
                } else {
                    // println!(
                    //     "{} {:?} {:?}",
                    //     x,
                    //     y.name(),
                    //     y.kind(),
                    // );
                    // stdout().write(a.content()).unwrap();
                }
            }
            git2::TreeWalkResult::Ok
        })
        .unwrap();
    }
    let mu = memusage_linux();
    drop(java_tree_gen);
    drop(full_nodes);
    let mu = mu - memusage_linux();
    println!("memory used {}", mu);
}

fn spoon_filter(x: &str) -> bool {
    !(x.contains("src/test/resources/") || x.ends_with("src/test/resources"))
}

fn get_up_to_date_repo(path: &String, mut fo: git2::FetchOptions, url: &String) -> Repository {
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

fn all_commits_between<'a>(
    repository: &'a Repository,
    before: &'a str,
    after: &'a str,
) -> Revwalk<'a> {
    use git2::*;
    let mut rw = repository.revwalk().unwrap();
    if !before.is_empty() {
        rw.hide_ref(before).unwrap();
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
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test() {}
}
