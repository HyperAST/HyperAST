use std::path::Path;

use git2::{RemoteCallbacks, Repository, Revwalk};

fn main() {
    let url = "https://github.com/INRIA/spoon";
    let path = "/home/quentin/resources/repo/INRIA/spoon";

    let mut callbacks = RemoteCallbacks::new();

    callbacks.transfer_progress(|x| {
        println!("transfer {}/{}", x.received_objects(), x.total_objects());
        true
    });

    let mut fo = git2::FetchOptions::new();

    fo.remote_callbacks(callbacks);

    let mut repository = if Path::new(path).join(".git").exists() {
        let mut repository = match Repository::open(path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };
        println!("fetch: {}", &path);
        {
            let mut remote = repository.find_remote("origin").unwrap();
            remote.fetch(&["main"], Some(&mut fo), None).unwrap();
        };

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
    };

    // let index = repo.index().unwrap();
    let rw = all_commits_from_head(&repository);

    for oid in rw {
        let oid = oid.unwrap();
        let commit = repository.find_commit(oid).unwrap();

        println!(
            "{} {:?}",
            oid,
            &commit.parent_ids().into_iter().collect::<Vec<_>>()
        );
    }
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
