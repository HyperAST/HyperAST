use std::{
    fmt::{Debug, Display},
    fs,
    path::{Path, PathBuf},
    process,
};

pub use git2::Error;
pub use git2::Oid;
pub use git2::Repository;
use git2::{RemoteCallbacks, Revwalk, TreeEntry};
use hyperast::{position::Position, utils::Url};

use crate::processing::ObjectName;

pub struct Builder<'a>(git2::Revwalk<'a>, &'a git2::Repository, bool);

impl<'a> Builder<'a> {
    pub fn new(repository: &'a Repository) -> Result<Self, git2::Error> {
        let mut rw = repository.revwalk()?;
        rw.set_sorting(git2::Sort::TOPOLOGICAL & git2::Sort::TIME)?;
        Ok(Self(rw, repository, false))
    }
    pub fn before(mut self, before: &str) -> Result<Self, git2::Error> {
        if before.is_empty() {
            return Ok(self);
        }
        let c = retrieve_commit(self.1, before)?;
        for c in c.parents() {
            self.0.hide(c.id())?;
        }
        Ok(self)
    }

    pub fn after(mut self, after: &str) -> Result<Self, git2::Error> {
        if after.is_empty() {
            return Ok(self);
        }
        let c = retrieve_commit(self.1, after)?;
        self.0.push(c.id())?;
        self.2 = true;
        Ok(self)
    }

    pub fn first_parents(mut self) -> Result<Self, git2::Error> {
        self.0.simplify_first_parent()?;
        Ok(self)
    }

    pub fn walk(mut self) -> Result<Revwalk<'a>, git2::Error> {
        if !self.2 {
            self.0.push_head()?;
        }
        Ok(self.0)
    }
}

/// Initialize a [git2::revwalk::Revwalk] to explore commits between before and after.
///
/// # Arguments
///
/// * `repository` - The repository where the walk is done
/// * `before` - The the parent commit
/// * `after` - The the child commit
///
/// # Property
///
/// if `after` is not a descendant of before then only walk `before`
///
/// # Errors
///
/// This function just lets errors from [git2] bubble up.
pub(crate) fn all_commits_between<'a>(
    repository: &'a Repository,
    befor: &str,
    after: &str,
) -> Result<Revwalk<'a>, git2::Error> {
    let b = if befor.is_empty() { &[][..] } else { &[befor] };
    let a = if after.is_empty() { &[][..] } else { &[after] };
    all_commits_between_multi(repository, b, a)
}

pub(crate) fn all_commits_between_multi<'a>(
    repository: &'a Repository,
    before: &[impl AsRef<str>],
    after: &[impl AsRef<str>],
) -> Result<Revwalk<'a>, git2::Error> {
    use git2::*;
    let mut rw = repository.revwalk()?;
    for before in before {
        let before = before.as_ref();
        if before.is_empty() {
            return Err(git2::Error::from_str("one `before` is empty"));
        }
        let c = retrieve_commit(repository, before)?;
        for c in c.parents() {
            rw.hide(c.id())?;
        }
    }
    if after.is_empty() {
        rw.push_head()?;
    } else {
        for after in after {
            let after = after.as_ref();
            if after.is_empty() {
                return Err(git2::Error::from_str("one `after` is empty"));
            }
            let c = retrieve_commit(repository, after)?;
            rw.push(c.id())?;
        }
    }
    rw.set_sorting(Sort::TOPOLOGICAL)?;
    Ok(rw)
}

pub(crate) fn all_first_parents_between<'a>(
    repository: &'a Repository,
    before: &str,
    after: &str,
) -> Result<Revwalk<'a>, git2::Error> {
    let mut rw = all_commits_between(repository, before, after)?;
    rw.simplify_first_parent()?;
    Ok(rw)
}

pub fn retrieve_commit<'a>(
    repository: &'a Repository,
    s: &str,
) -> Result<git2::Commit<'a>, git2::Error> {
    // TODO make a more advanced search with helpful error msgs
    match repository.find_reference(&format!("refs/tags/{}", s)) {
        Ok(c) => match c.peel_to_commit() {
            Ok(c) => Ok(c),
            Err(err) => {
                log::warn!("{}", err);
                repository.find_commit(Oid::from_str(s)?)
            }
        },
        Err(err) => {
            let oid = Oid::from_str(s).map_err(|e| {
                log::warn!("{}", e);
                err
            })?;
            repository.find_commit(oid)
        }
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

    let repository = up_to_date_repo(&path, Some(fo), url.clone());
    if let Ok(repository) = repository {
        return repository;
    }

    if let Err(err) = process::Command::new("git")
        .arg("clone")
        .arg(url.to_string())
        .current_dir(&*path.to_string_lossy())
        .spawn()
    {
        log::error!("tryed to use the git executable, but failed. {}", err);
    }

    nofetch_repository(url, path)
}

pub fn nofetch_repository<'a, T: TryInto<Url>, U: Into<PathBuf>>(url: T, path: U) -> Repository
where
    <T as TryInto<Url>>::Error: std::fmt::Debug,
{
    let url: Url = url.try_into().unwrap();
    let mut path: PathBuf = path.into();
    path.push(url.path.clone());

    let repository = up_to_date_repo(&path, None, url);
    repository.unwrap()
}

#[derive(Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub enum Forge {
    Github,
    Gitlab,
    GitlabInria,
}

impl std::str::FromStr for Forge {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "github.com" => Self::Github,
            "gitlab.com" => Self::Gitlab,
            "gitlab.inria.fr" => Self::GitlabInria,
            x => return Err(format!("'{}' is not an authorize forge", x)),
        })
    }
}

impl Forge {
    fn url(&self) -> &str {
        match self {
            Forge::Github => "https://github.com/",
            Forge::Gitlab => "https://gitlab.com/",
            Forge::GitlabInria => "https://gitlab.inria.fr/",
        }
    }

    /// panics in case `user`` or `name`` contain '/' '#'
    pub fn repo(self, user: impl Into<String>, name: impl Into<String>) -> Repo {
        self.try_repo(user, name).unwrap()
    }

    pub fn try_repo(
        self,
        user: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Repo, String> {
        let user = user.into();
        if user.contains("#") || user.contains("/") {
            return Err("attempting to inject stuff!".to_string());
        }
        let name = name.into();
        if name.contains("#") || name.contains("/") {
            return Err("attempting to inject stuff!".to_string());
        }
        Ok(Repo {
            forge: self,
            user,
            name,
        })
    }
}

// TODO use `&'static str`s to derive with Copy
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Repo {
    forge: Forge,
    user: String,
    name: String,
}

impl Repo {
    pub fn url(&self) -> String {
        format!("{}{}/{}", self.forge.url(), self.user, self.name)
    }
    pub fn fetch(&self) -> Repository {
        let url = self.url();
        let path = format!("{}", "/tmp/hyperastgitresources/repo/");
        fetch_repository(url, path)
    }
    pub fn nofetch(&self) -> Repository {
        let url = self.url();
        let path = format!("{}", "/tmp/hyperastgitresources/repo/");
        nofetch_repository(url, path)
    }

    pub fn fetch_to(&self, path: impl Into<PathBuf>) -> Repository {
        let url = self.url();
        let path = path.into();
        fetch_repository(url, path)
    }

    pub fn nofetch_to(&self, path: impl Into<PathBuf>) -> Repository {
        let url = self.url();
        let path = path.into();
        nofetch_repository(url, path)
    }

    pub fn forge(&self) -> Forge {
        self.forge
    }
    pub fn user(&self) -> &str {
        &self.user
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}{}/{}", self.forge.url(), self.user, self.name)
    }
}

impl std::str::FromStr for Repo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (forge, repo) = s
            .split_once("/")
            .ok_or("give a valid repository address without 'https://' and '.git'")?;
        let (user, name) = repo
            .split_once("/")
            .ok_or("give a valid repository address without 'https://' and '.git'")?;
        let forge: Forge = forge.parse()?;
        if name.contains("/") {
            return Err(format!(
                "{} should not contain anymore '/' give a valid repository address",
                name
            ));
        }
        forge.try_repo(user, name)
    }
}

pub fn fetch_github_repository(repo_name: &str) -> Repository {
    let url = format!("{}{}", "https://github.com/", repo_name);
    let path = format!("{}", "/tmp/hyperastgitresources/repo/");
    fetch_repository(url, path)
}

pub fn fetch_fork(mut x: git2::Remote, head: &str) -> Result<(), git2::Error> {
    let mut callbacks = RemoteCallbacks::new();

    callbacks.transfer_progress(|x| {
        log::info!(
            "fork transfer {}/{}",
            x.received_objects(),
            x.total_objects()
        );
        true
    });

    let mut fo = git2::FetchOptions::new();

    fo.remote_callbacks(callbacks);
    x.fetch(&[head], Some(&mut fo), None)
}

/// avoid mixing providers
pub fn up_to_date_repo(
    path: &Path,
    fo: Option<git2::FetchOptions>,
    url: Url,
) -> Result<Repository, git2::Error> {
    if path.join(".git").exists() {
        let repository = match Repository::open(path) {
            Ok(repo) => repo,
            Err(e) if e.code() == git2::ErrorCode::NotFound => {
                if path.starts_with("/tmp") {
                    if let Err(e) = fs::remove_dir_all(path.join(".git")) {
                        panic!("failed to remove currupted clone: {}", e)
                    } else {
                        return if let Some(fo) = fo {
                            clone_helper(url, path, fo)
                        } else {
                            Err(e)
                        };
                    }
                } else {
                    panic!("failed to open: {}", e)
                }
            }
            Err(e) => panic!("failed to open: {}", e),
        };

        if let Some(mut fo) = fo {
            log::info!("fetch: {:?}", path);
            repository
                .find_remote("origin")
                .unwrap()
                .fetch(&["main"], Some(&mut fo), None)
                .unwrap_or_else(|e| log::error!("{}", e));
        }

        Ok(repository)
    } else if path.exists() && path.read_dir().map_or(true, |mut x| x.next().is_some()) {
        todo!()
    } else if let Some(fo) = fo {
        clone_helper(url, path, fo)
    } else {
        panic!("there is no repo there, you can enable the cloning by provinding a fetch callback.")
    }
}

fn clone_helper(url: Url, path: &Path, fo: git2::FetchOptions) -> Result<Repository, git2::Error> {
    let mut builder = git2::build::RepoBuilder::new();

    builder.bare(true);

    builder.fetch_options(fo);

    log::info!("clone {} in {:?}", url, path);
    let repository = match builder.clone(&url.to_string(), path.join(".git").as_path()) {
        Ok(repo) => repo,
        Err(e) if e.code() == git2::ErrorCode::Locked => {
            match builder.clone(&url.to_string(), path.join(".git").as_path()) {
                Ok(repo) => repo,
                Err(e) if e.code() == git2::ErrorCode::Locked => return Err(e),
                Err(e) => return Err(e),
            }
        }
        Err(e) => return Err(e),
    };
    Ok(repository)
}

pub(crate) enum BasicGitObject {
    Blob(Oid, ObjectName),
    Tree(Oid, ObjectName),
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
            Ok(Self::Tree(x.id(), x.name_bytes().into()))
        } else if x.kind().unwrap().eq(&git2::ObjectType::Blob) {
            Ok(Self::Blob(x.id(), x.name_bytes().into()))
        } else {
            Err(x)
        }
    }
}

pub trait NamedObject {
    fn name(&self) -> &ObjectName;
}

impl NamedObject for BasicGitObject {
    fn name(&self) -> &ObjectName {
        match self {
            BasicGitObject::Blob(_, n) => n,
            BasicGitObject::Tree(_, n) => n,
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
