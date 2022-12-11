use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

pub use git2::Oid;
use git2::{RemoteCallbacks, Repository, Revwalk, TreeEntry};
use hyper_ast::position::Position;

pub fn all_commits_between<'a>(
    repository: &'a Repository,
    before: &'a str,
    after: &'a str,
) -> Revwalk<'a> {
    use git2::*;
    let mut rw = repository.revwalk().unwrap();
    if !before.is_empty() {
        // rw.hide_ref(before).unwrap();
        // log::debug!("{}", before);
        let c = retrieve_commit(repository, before).unwrap();
        // log::debug!("{:?}", c);
        for c in c.parents() {
            rw.hide(c.id()).unwrap();
        }
    }
    if after.is_empty() {
        rw.push_head().unwrap();
    } else {
        let c = retrieve_commit(repository, after).unwrap();
        rw.push(c.id()).unwrap();
    }
    rw.set_sorting(Sort::TOPOLOGICAL).unwrap();
    rw
}

pub fn retrieve_commit<'a>(
    repository: &'a Repository,
    s: &str,
) -> Result<git2::Commit<'a>, git2::Error> {
    if let Ok(c) = repository.find_reference(s) {
        if let Ok(c) = c.peel_to_commit() {
            Ok(c)
        } else {
            repository.find_commit(Oid::from_str(s)?)
        }
    } else {
        let oid = Oid::from_str(s)?;
        repository.find_commit(oid)
    }
}

pub fn all_commits_from_head(repository: &Repository) -> Revwalk {
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

pub struct Url {
    protocol: String,
    domain: String,
    path: String,
}

impl TryFrom<String> for Url {
    type Error = ();

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let (protocol, rest) = match s.split_once("://") {
            Some((protocol, rest)) => (protocol, rest),
            None => ("https", s.as_ref()),
        };

        let (domain, path) = rest.split_once("/").ok_or(())?;

        Ok(Self {
            protocol: protocol.to_string(),
            domain: domain.to_string(),
            path: path.to_string(),
        })
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}/{}", self.protocol, self.domain, self.path)
    }
}

pub fn fetch_repository<'a, T: TryInto<Url>, U: Into<PathBuf>>(url: T, path: U) -> Repository
where
    <T as TryInto<Url>>::Error: std::fmt::Debug,
{
    let url: Url = url.try_into().unwrap();
    let mut path: PathBuf = path.into();
    path.push(url.path.clone());
    // let url = &format!("{}{}", "https://github.com/", repo_name);
    // let path = &format!("{}{}", "/tmp/hyperastgitresources/repo/", repo_name);
    let mut callbacks = RemoteCallbacks::new();

    callbacks.transfer_progress(|x| {
        log::info!("transfer {}/{}", x.received_objects(), x.total_objects());
        true
    });

    let mut fo = git2::FetchOptions::new();

    fo.remote_callbacks(callbacks);

    let repository = up_to_date_repo(&path, fo, url);
    repository
}

pub fn fetch_github_repository(repo_name: &str) -> Repository {
    let url = format!("{}{}", "https://github.com/", repo_name);
    let path = format!("{}", "/tmp/hyperastgitresources/repo/");
    fetch_repository(url, path)
}

/// avoid mixing providers
pub fn up_to_date_repo(path: &Path, mut fo: git2::FetchOptions, url: Url) -> Repository {
    if path.join(".git").exists() {
        let repository = match Repository::open(path) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to open: {}", e),
        };
        log::info!("fetch: {:?}", path);
        repository
            .find_remote("origin")
            .unwrap()
            .fetch(&["main"], Some(&mut fo), None)
            .unwrap_or_else(|e| log::error!("{}", e));

        repository
    } else if path.exists() {
        todo!()
    } else {
        let mut builder = git2::build::RepoBuilder::new();

        builder.bare(true);

        builder.fetch_options(fo);

        log::info!("clone {} in {:?}", url, path);
        let repository = match builder.clone(&url.to_string(), path.join(".git").as_path()) {
            Ok(repo) => repo,
            Err(e) => panic!("failed to clone: {}", e),
        };
        repository
    }
}

pub(crate) enum BasicGitObject {
    Blob(Oid, Vec<u8>),
    Tree(Oid, Vec<u8>),
}

// impl<'a> From<TreeEntry<'a>> for BasicGitObjects {
//     fn from(x: TreeEntry<'a>) -> Self {
//         if x.kind().unwrap().eq(&git2::ObjectType::Tree) {
//             Self::Tree(x.id(), x.name_bytes().to_owned())
//         } else if x.kind().unwrap().eq(&git2::ObjectType::Blob) {
//             Self::Blob(x.id(), x.name_bytes().to_owned())
//         } else {
//             panic!("{:?} {:?}",x.kind(), x.name_bytes())
//         }
//     }
// }

impl<'a> TryFrom<TreeEntry<'a>> for BasicGitObject {
    type Error = TreeEntry<'a>;

    fn try_from(x: TreeEntry<'a>) -> Result<Self, Self::Error> {
        if x.kind().unwrap().eq(&git2::ObjectType::Tree) {
            Ok(Self::Tree(x.id(), x.name_bytes().to_owned()))
        } else if x.kind().unwrap().eq(&git2::ObjectType::Blob) {
            Ok(Self::Blob(x.id(), x.name_bytes().to_owned()))
        } else {
            Err(x)
        }
    }
}

impl NamedObject for BasicGitObject {
    fn name(&self) -> &[u8] {
        match self {
            BasicGitObject::Blob(_, n) => &n,
            BasicGitObject::Tree(_, n) => &n,
        }
    }
}
impl TypedObject for BasicGitObject {
    fn r#type(&self) -> ObjectType {
        match self {
            BasicGitObject::Blob(..) => ObjectType::File,
            BasicGitObject::Tree(..) => ObjectType::Dir,
        }
    }
}
impl UniqueObject for BasicGitObject {
    type Id = Oid;
    fn id(&self) -> &Oid {
        match self {
            BasicGitObject::Tree { 0: id, .. } => id,
            BasicGitObject::Blob { 0: id, .. } => id,
        }
    }
}

pub trait NamedObject {
    fn name(&self) -> &[u8];
}

pub enum ObjectType {
    File,
    Dir,
}

pub trait TypedObject {
    fn r#type(&self) -> ObjectType;
}
pub trait UniqueObject {
    type Id: Clone;
    fn id(&self) -> &Self::Id;
}

// enum File<'a, 'b, Id> {
//     File(Id, Vec<u8>, &'a [u8]),
//     Dir(Id, Vec<u8>, &'b [Id]),
// }
// impl<'a, 'b, Id> NamedObject for File<'a, 'b, Id> {
//     fn name(&self) -> &[u8] {
//         match self {
//             File::Dir { 1: name, .. } => name,
//             File::File { 1: name, .. } => name,
//         }
//     }
// }
// impl<'a, 'b, Id: Clone> UniqueObject for File<'a, 'b, Id> {
//     type Id = Id;
//     fn id(&self) -> &Id {
//         match self {
//             File::Dir { 0: id, .. } => id,
//             File::File { 0: id, .. } => id,
//         }
//     }
// }
// impl<'a, 'b, Id> TypedObject for File<'a, 'b, Id> {
//     fn r#type(&self) -> ObjectType {
//         match self {
//             File::Dir(..) => ObjectType::Dir,
//             File::File(..) => ObjectType::File,
//         }
//     }
// }

pub fn read_position(
    repo: &Repository,
    commit: &str,
    position: &Position,
) -> Result<String, git2::Error> {
    read_position_floating(repo, commit, position, 0).map(|x| x.1)
}

// let mut before = 0;
// for _ in 0..border_lines {
//     let x = text[before..]
//         .find(|x: char| x == '\n')
//         .unwrap_or(text.len() - before);
//     before = before + x + 1;
//     before = before.min(text.len() - 1)
// }
// let mut after = text.len();
// for _ in 0..border_lines {
//     let x = text[..after].rfind(|x: char| x == '\n').unwrap_or_default();
//     after = x;
// }
pub fn read_position_floating_lines(
    repo: &Repository,
    commit: &str,
    position: &Position,
    lines: usize,
) -> Result<(String, String, String), git2::Error> {
    let blob = blob_position(repo, Oid::from_str(commit)?, &position)?;
    compute_range_floating(
        blob.content(),
        position,
        |r| {
            if r.is_empty() {
                return r;
            }
            let mut i = r.len();
            for _ in 0..=lines {
                i = r[..i].iter().rposition(|x| *x == b'\n').unwrap_or_default();
            }
            &r[i..]
        },
        |r| {
            if r.is_empty() {
                return r;
            }
            let mut i = 0;
            for _ in 0..=lines {
                let x = r[i..]
                    .iter()
                    .position(|x| *x == b'\n')
                    .unwrap_or(r.len() - i);
                i = i + x + 1;
                i = i.min(r.len() - 1)
            }
            &r[..i]
        },
    )
    .map_err(|err| {
        git2::Error::new(
            err.code(),
            err.class(),
            position.file().to_str().unwrap().to_string() + err.message(),
        )
    })
}

pub fn read_position_floating(
    repo: &Repository,
    commit: &str,
    position: &Position,
    radius: usize,
) -> Result<(String, String, String), git2::Error> {
    let blob = blob_position(repo, Oid::from_str(commit)?, &position)?;
    compute_range_floating(
        blob.content(),
        position,
        |r| {
            let x = r.len().saturating_sub(radius);
            &r[x..]
        },
        |r| {
            let x = radius.min(r.len());
            &r[..x]
        },
    )
    .map_err(|err| {
        git2::Error::new(
            err.code(),
            err.class(),
            position.file().to_str().unwrap().to_string() + err.message(),
        )
    })
}
fn compute_range_floating<F, G>(
    text: &[u8],
    position: &Position,
    f_start: F,
    f_end: G,
) -> Result<(String, String, String), git2::Error>
where
    F: Fn(&[u8]) -> &[u8],
    G: Fn(&[u8]) -> &[u8],
{
    let range = position.range();
    let before = f_start(&text.get(..range.start).ok_or_else(|| {
        git2::Error::from_str(&format!(
            "range {:?} out of text ({}) {:?}",
            range,
            text.len(),
            std::str::from_utf8(text)
        ))
    })?);
    let after = f_end(&text.get(range.end..).ok_or_else(|| {
        git2::Error::from_str(&format!(
            "range {:?} out of text ({}) {:?}",
            range,
            text.len(),
            std::str::from_utf8(text)
        ))
    })?);
    Ok((own(before)?, own(&text[range])?, own(after)?))
}
fn blob_position<'a>(
    repo: &'a Repository,
    commit: Oid,
    position: &Position,
) -> Result<git2::Blob<'a>, git2::Error> {
    let commit = repo.find_commit(commit)?;
    let tree = commit.tree()?;
    let file = tree.get_path(position.file())?;
    let obj = file.to_object(repo)?;
    let blob = obj.into_blob();
    blob.map_err(|_| git2::Error::from_str("file path in position should be a valid file"))
}

fn own(r: &[u8]) -> Result<String, git2::Error> {
    let r = std::str::from_utf8(r);
    let r = r.map_err(|x| git2::Error::from_str(&x.to_string()));
    r.map(|s| s.to_string())
}
