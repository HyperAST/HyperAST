use std::{collections::HashMap, i64};

use poll_promise::Promise;
use serde::{Deserialize, Serialize};

use crate::app::types::Resource;

use super::types::{Commit, CommitId, Repo};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommitMetadata {
    /// commit message
    pub(crate) message: Option<String>,
    /// parents commits
    /// if multiple parents, the first one should be where the merge happends
    pub(crate) parents: Vec<String>,
    /// tree corresponding to version
    pub(crate) tree: Option<String>,
    /// offset in minutes
    pub(crate) timezone: i32,
    /// seconds
    pub(crate) time: i64,
    /// (opt) ancestors in powers of 2; [2,4,8,16,32]
    /// important to avoid linear loading time
    pub(crate) ancestors: Vec<String>,
    pub(crate) forth_timestamp: i64,
}

impl CommitMetadata {
    pub(crate) fn show(&self, ui: &mut egui::Ui) {
        let tz = &chrono::FixedOffset::west_opt(self.timezone * 60).unwrap();
        let date = chrono::Duration::seconds(self.time);
        let date = chrono::DateTime::<chrono::FixedOffset>::default()
            .with_timezone(tz)
            .checked_add_signed(date);
        if let Some(date) = date {
            ui.label(format!("Date:\t{:?}", date));
        } else {
            // wasm_rs_dbg::dbg!(self.timezone, self.time);
        }
        if ui.available_width() > 300.0 {
            ui.label(format!("Parents: {}", self.parents.join(" + ")));
        } else {
            use itertools::intersperse;
            let text = intersperse(self.parents.iter().map(|x| &x[..8]), " + ").collect::<String>();
            let label = ui.label(format!("Parents: {}", text));
            if label.hovered() {
                let text = self.parents.join(" + ");
                egui::show_tooltip(ui.ctx(), ui.layer_id(), label.id.with("tooltip"), |ui| {
                    ui.label(&text);
                    ui.label("CTRL+C to copy (and send in the debug console)");
                });
                const SC_COPY: egui::KeyboardShortcut =
                    egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::C);
                wasm_rs_dbg::dbg!(&text);
                if ui.input_mut(|mem| mem.consume_shortcut(&SC_COPY)) {
                    wasm_rs_dbg::dbg!(&text);
                    ui.ctx().copy_text(text.to_string());
                }
            }
        }
        if let Some(msg) = &self.message {
            if let Some(head) = msg.lines().next() {
                let label0 = ui.label("Commit message:");
                let head =
                    egui::RichText::new(head).background_color(ui.style().visuals.extreme_bg_color);
                let label = ui.label(head);
                if label0.hovered() || label.hovered() {
                    egui::show_tooltip(ui.ctx(), ui.layer_id(), label.id.with("tooltip"), |ui| {
                        ui.text_edit_multiline(&mut msg.to_string());
                    });
                }
            }
        }
    }
}

pub(super) fn fetch_commit(
    ctx: &egui::Context,
    api_addr: &str,
    commit: &Commit,
) -> Promise<Result<super::CommitMdPayload, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/commit/github/{}/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name, &commit.id,
    );

    // wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource = response
            .and_then(|response| Resource::<CommitMetadata>::from_response(&ctx, response))
            .and_then(|x| x.content.ok_or("No content".into()))
            .map(|x| (x, None));
        sender.send(resource);
    });
    promise
}

pub(super) fn fetch_commit0(
    ctx: &egui::Context,
    api_addr: &str,
    commit: &Commit,
) -> Promise<Result<CommitMetadata, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/commit/github/{}/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name, &commit.id,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource = response
            .and_then(|response| Resource::<CommitMetadata>::from_response(&ctx, response))
            .and_then(|x| x.content.ok_or("No content".into()));
        sender.send(resource);
    });
    promise
}

impl Resource<CommitMetadata> {
    fn from_response(_ctx: &egui::Context, response: ehttp::Response) -> Result<Self, String> {
        // let content_type = response.content_type().unwrap_or_default();

        let text = response.text();
        let text = text.ok_or("")?;
        let text = serde_json::from_str(text).map_err(|x| x.to_string())?;

        Ok(Self {
            response,
            content: text,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MergePr {
    pub(crate) merge_commit: Option<Commit>,
    pub(crate) head_commit: Commit,
    pub(crate) title: String,
    pub(crate) number: i64,
}

impl Resource<MergePr> {
    fn from_response(_ctx: &egui::Context, response: ehttp::Response) -> Result<Self, String> {
        let text = response.text();
        let text = text.ok_or("nothing in response")?.to_string();
        let text = serde_json::from_str(&text).map_err(|x| x.to_string())?;

        Ok(Self {
            response,
            content: text,
        })
    }
}

pub(super) fn fetch_merge_pr(
    ctx: &egui::Context,
    api_addr: &str,
    commit: &Commit,
    md: CommitMetadata,
    pid: ProjectId,
) -> Promise<Result<super::CommitMdPayload, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/pr/github/{}/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name, &commit.id,
    );
    let url_fork = format!(
        "http://{}/fork/github/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource = response
            .and_then(|response| Resource::<MergePr>::from_response(&ctx, response))
            .and_then(|x| x.content.ok_or("No content".into()));

        let resource = resource.map(|x| {
            let request = ehttp::Request::post(
                &format!(
                    "{}/{}/{}/{}",
                    url_fork, x.head_commit.repo.user, x.head_commit.repo.name, x.head_commit.id,
                ),
                Default::default(),
            );
            ehttp::fetch(request, |x| log::info!("{:?}", x));

            log::error!("{:?}", x);
            // if !md.parents.contains(&x.head_commit.id) {
            //     md.parents.push(x.head_commit.id.clone());
            // }
            (md, Some((x.head_commit, pid)))
        });
        sender.send(resource);
    });
    promise
}

#[allow(unused)]
pub(super) fn fetch_commit_parents(
    ctx: &egui::Context,
    api_addr: &str,
    commit: &Commit,
    depth: usize,
) -> Promise<Result<Vec<String>, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/commit-parents/github/{}/{}/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name, &commit.id, depth
    );

    let request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        wasm_rs_dbg::dbg!(&response);
        ctx.request_repaint(); // wake up UI thread
        let resource = response
            .and_then(|response| Resource::<Vec<String>>::from_response(&ctx, response))
            .and_then(|x| x.content.ok_or("No content".into()));
        sender.send(resource);
    });
    promise
}

impl Resource<Vec<String>> {
    #[allow(unused)]
    fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Result<Self, String> {
        wasm_rs_dbg::dbg!(&response);
        // let content_type = response.content_type().unwrap_or_default();

        let text = response.text();
        let text = text.ok_or("")?;
        let text = serde_json::from_str(text).map_err(|x| x.to_string())?;

        Ok(Self {
            response,
            content: text,
        })
    }
}

pub(crate) fn validate_pasted_project_url(
    paste: &str,
) -> Result<(super::types::Repo, Vec<String>), &'static str> {
    use std::str::FromStr;
    match hyperast::utils::Url::from_str(paste) {
        Ok(url) if &url.domain == "github.com" && &url.protocol == "https" => {
            let path = url.path.split_once('&').map_or(url.path.as_str(), |x| x.0);
            let path: Vec<_> = path.split('/').collect();
            if path.len() > 1 && !path[0].is_empty() && !path[1].is_empty() {
                let repo = super::types::Repo {
                    user: path[0].to_string(),
                    name: path[1].to_string(),
                };
                if let Some(after_repo) = path.get(2) {
                    if *after_repo == "commit" {
                        if let Some(after_repo) = path.get(3) {
                            if after_repo.chars().all(|x| x.is_alphanumeric()) {
                                Ok((repo, vec![after_repo.to_string()]))
                            } else if let Some(_) = after_repo.split_once("..") {
                                Err("range of commits are not handled (WIP)")
                            } else {
                                Err("url scheme not handled")
                            }
                        } else {
                            Err("commit id missing")
                        }
                    } else if after_repo.chars().all(|x| x.is_alphanumeric()) {
                        Ok((repo, vec![after_repo.to_string()]))
                    } else if let Some(_) = after_repo.split_once("..") {
                        Err("range of commits are not handled (WIP)")
                    } else {
                        Err("url scheme not handled")
                    }
                } else {
                    Ok((repo, vec![]))
                }
            } else {
                Err("url scheme not handled")
            }
        }
        Ok(url) if &url.protocol == "https" => Err("not a github.com domain "),
        Ok(_) => Err("must be https protocol"),
        _ => Err("not a valid url"),
    }
}

/// Selection of projects.
///     Each project is identified by main repository.
///     Each project contains a selection of other repositories considered as forks,
///     and a set of commits (not branches)
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct SelectedProjects {
    len: usize,
    repositories: Vec<Repo>,
    offsets: Vec<u32>,
    // TODO use inline commit ids
    commits: Vec<CommitId>,
    // TODO add forks
    // NOTE using git remote add on backend should be the best approach https://stackoverflow.com/questions/66621183/github-how-to-work-across-multiple-forks-of-the-same-repository
}

impl Default for SelectedProjects {
    fn default() -> Self {
        let mut s = Self::empty();
        s.add_with_commit_slice(
            ["INRIA", "spoon"].into(),
            &["56e12a0c0e0e69ea70863011b4f4ca3305e0542b"],
        );
        s.add_with_commit_slice(
            ["tree-sitter", "tree-sitter"].into(),
            &["800f2c41d0e35e4383172d7a67a16f3933b86039"],
        );
        s.add_with_commit_slice(
            ["rerun-io", "egui_tiles"].into(),
            &["0fe81768278678db4f66a297178c04f23452c682"],
        );
        s.add_with_commit_slice(
            ["tree-sitter", "tree-sitter-cpp"].into(),
            &["ab1065fa23a43a447bd7e619a3af90253867af24"],
        );
        s.add_with_commit_slice(
            ["graphhopper", "graphhopper"].into(),
            &["90acd4972610ded0f1581143f043eb4653a4c691"],
        );
        s.add_with_commit_slice(
            ["apache", "dubbo"].into(),
            &["aaafad80bec93ddb167ec613eb930749f5ec90ec"],
        );
        s
        // Cpp:
        // https://github.com/tree-sitter/tree-sitter/commit/800f2c41d0e35e4383172d7a67a16f3933b86039

        // Rust (just for the commit history, I still don't have the proper parsing facilities setup for Rust):
        // https://github.com/rerun-io/egui_tiles/0fe81768278678db4f66a297178c04f23452c682

        // Java:
        // https://github.com/INRIA/spoon/commit/56e12a0c0e0e69ea70863011b4f4ca3305e0542b
        // https://github.com/graphhopper/graphhopper/commit/90acd4972610ded0f1581143f043eb4653a4c691
        // Java repos with merges
        // https://github.com/dubbo/dubbo/commit/aaafad80bec93ddb167ec613eb930749f5ec90ec
    }
}

/// Id of each project, ie. a repository and a selection of other repositories considered as forks
#[derive(Deserialize, Serialize, Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub(crate) struct ProjectId(usize);

impl SelectedProjects {
    fn empty() -> Self {
        Self {
            len: 0,
            repositories: vec![],
            offsets: vec![],
            commits: vec![],
        }
    }

    pub(crate) fn add_with_commit_slice(
        &mut self,
        repo: Repo,
        commits: &[impl ToString],
    ) -> ProjectId {
        self.add(repo, commits.into_iter().map(|x| x.to_string()).collect())
    }

    pub(crate) fn add(&mut self, repo: Repo, commits: Vec<CommitId>) -> ProjectId {
        if let Some(i) = self.repositories.iter().position(|x| x == &repo) {
            let i = ProjectId(i);
            let (_, mut cs) = self._get_mut(i);
            // TODO opti extend on empty set of commits
            for c in commits {
                cs.push(c);
            }
            i
        } else {
            self.len += 1;
            let i = ProjectId(self.repositories.len());
            self.repositories.push(repo);
            assert!(self.commits.len() <= u32::MAX as usize);
            self.offsets.push(self.commits.len() as u32);
            self.commits.extend(commits);
            i
        }
    }

    pub(crate) fn remove(&mut self, ProjectId(i): ProjectId) {
        self.len -= 1;
        let range = self.c_range(i);
        log::debug!("before proj({}) {:?} rm: {:?}", i, range, self.offsets);
        self.offsets[i + 1..]
            .iter_mut()
            .for_each(|x| *x -= range.len() as u32);
        self.commits.drain(range);
        // self.repositories.remove(i as usize);
        // self.offsets.remove(i as usize);
        log::debug!("after proj {} rm: {:?}", self.commits.len(), self.offsets);
        if i > 0 {
            assert!(self.offsets[i - 1] <= self.offsets[i])
        }
        for i in self.project_ids() {
            log::debug!("{:?}", self.get_mut(i));
        }
    }

    fn c_range(&self, i: usize) -> std::ops::Range<usize> {
        let end = if let Some(i) = self.offsets.get(i + 1) {
            *i as usize
        } else {
            self.commits.len()
        };
        self.offsets[i] as usize..end
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn commit_count(&self) -> usize {
        self.commits.len()
    }

    pub(crate) fn project_ids(&self) -> impl Iterator<Item = ProjectId> + use<> {
        (0..self.repositories.len()).into_iter().map(ProjectId)
    }

    pub(crate) fn get<'a>(&'a mut self, ProjectId(i): ProjectId) -> Option<&'a Repo> {
        self.repositories.get(i)
    }

    pub(crate) fn get_mut<'a>(
        &'a mut self,
        ProjectId(i): ProjectId,
    ) -> Option<(&'a mut Repo, CommitSlice<'a>)> {
        if i >= self.repositories.len() {
            return None;
        }
        let c_range = self.c_range(i);
        if c_range.is_empty() {
            return None;
        };
        let end = c_range.end;
        Some((
            self.repositories.get_mut(i)?,
            CommitSlice {
                end,
                commits: &mut self.commits,
                offsets: &mut self.offsets,
                i,
            },
        ))
    }

    fn _get_mut<'a>(&'a mut self, ProjectId(i): ProjectId) -> (&'a mut Repo, CommitSlice<'a>) {
        let c_range = self.c_range(i);
        let end = c_range.end;
        (
            self.repositories.get_mut(i).unwrap(),
            CommitSlice {
                end,
                commits: &mut self.commits,
                offsets: &mut self.offsets,
                i,
            },
        )
    }

    // pub(crate) fn get<'a>(&'a self, i: usize) -> Option<(&'a Repo, CommitSlice<'a>)> {
    //     if i >= self.len() {
    //         return None;
    //     }
    //     let end = self.c_range(i).end;
    //     Some((
    //         &mut self.repositories[i],
    //         CommitSlice {
    //             end,
    //             commits: &mut self.commits,
    //             offsets: &mut self.offsets,
    //             i,
    //         },
    //     ))
    // }

    pub(crate) fn repositories(&mut self) -> impl Iterator<Item = &mut Repo> {
        self.repositories.iter_mut()
    }
}

#[derive(Debug)]
pub(crate) struct CommitSlice<'a> {
    offsets: &'a mut Vec<u32>,
    commits: &'a mut Vec<CommitId>,
    i: usize,
    end: usize,
}

impl<'a> CommitSlice<'a> {
    pub(crate) fn push(&mut self, c: CommitId) {
        let start = self.offsets[self.i] as usize;
        if self.commits[start..self.end].contains(&c) {
            return;
        }
        self.commits.insert(self.end, c);
        self.end += 1;
        self.offsets[self.i + 1..].iter_mut().for_each(|x| *x += 1);
    }

    pub(crate) fn last_mut(&mut self) -> Option<&mut CommitId> {
        if self.end == 0 {
            return None;
        }
        Some(&mut self.commits[self.end - 1])
    }

    pub(crate) fn pop(&mut self) -> CommitId {
        if self.end == 0 {
            panic!("trying to remove a commit from a project without any")
        }
        self.end -= 1;
        self.offsets[self.i + 1..].iter_mut().for_each(|x| *x -= 1);
        self.commits.remove(self.end)
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut CommitId> {
        self.commits[self.offsets[self.i] as usize..self.end].iter_mut()
    }

    pub(crate) fn remove(&mut self, j: usize) -> CommitId {
        self.end -= 1;
        self.offsets[self.i + 1..].iter_mut().for_each(|x| *x -= 1);
        self.commits.remove(self.offsets[self.i] as usize + j)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CommitsLayout {
    pub(crate) commits: Vec<CommitId>,
    pub(crate) pos: Vec<egui::Pos2>,
    // indexing in subs
    pub(crate) branches: Vec<usize>,
    pub(crate) subs: Vec<Subs>,
    pub(crate) rect: egui::Rect,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Subs {
    // indexing in CommitsLayout.commits
    pub(crate) prev: usize,
    // indexing in CommitsLayout.commits
    pub(crate) start: usize,
    // indexing in CommitsLayout.commits
    pub(crate) end: usize,
    // indexing in CommitsLayout.commits
    pub(crate) succ: usize,
}

impl Default for CommitsLayout {
    fn default() -> Self {
        Self {
            commits: Default::default(),
            pos: Default::default(),
            branches: Default::default(),
            subs: Default::default(),
            rect: egui::Rect::ZERO,
        }
    }
}

pub(crate) fn compute_commit_layout(
    commits: impl Fn(&CommitId) -> Option<CommitMetadata>,
    branches: impl Iterator<Item = (String, CommitId)>,
) -> CommitsLayout {
    // let commits: Vec<commits_layouting::CommitInfo> = commits.map(|x| todo!()).collect();
    // let indices: std::collections::HashMap<commits_layouting::Oid, usize> = commits
    //     .iter()
    //     .enumerate()
    //     .map(|(i, c)| (c.oid, i))
    //     .collect();
    // let mut branches: Vec<commits_layouting::BranchInfo> = branches.map(|x| todo!()).collect();
    // let settings = commits_layouting::BranchSettings::new(branches.len());
    // branches.into_iter().map(|x| 42).collect()
    use egui::Pos2;
    let mut r = CommitsLayout {
        commits: vec![],
        pos: vec![],
        branches: vec![],
        subs: vec![],
        rect: egui::Rect::ZERO,
    };
    let mut index = HashMap::<CommitId, usize>::default();
    let mut v = 0.0;
    for (branch_name, target) in branches {
        let mut h = 0.0;
        r.commits.push(branch_name);
        r.branches.push(r.subs.len());
        let mut waiting: Vec<(String, usize)> = vec![(target, r.pos.len())];
        r.pos.push(Pos2::new(h, v));
        loop {
            let Some((mut current, prev)) = waiting.pop() else {
                break;
            };
            h = r.pos[prev].x;
            let start = r.pos.len();
            let mut succ = None;
            loop {
                if let Some(fork) = index.get(&current) {
                    succ = Some(*fork);
                    break;
                }
                index.insert(current.clone(), r.pos.len());
                h += 10.0;
                if let Some(commit) = commits(&current) {
                    if let Some(p) = commit.parents.get(0) {
                        r.commits.push(format!("{p}"));
                        r.pos.push(Pos2::new(h, v));
                        current = p.clone();
                    } else {
                        r.commits.push(format!("<end>"));
                        r.pos.push(Pos2::new(h, v));
                        r.rect.max.x = r.rect.max.x.max(h);
                        break;
                    }
                    if let Some(p) = commit.parents.get(1..) {
                        for p in p {
                            waiting.push((p.to_string(), r.pos.len() - 1));
                        }
                    }
                } else {
                    r.commits.push(format!("m|{current}"));
                    r.pos.push(Pos2::new(h, v));
                    r.rect.max.x = r.rect.max.x.max(h);
                    break;
                }
            }
            let end = r.pos.len();
            let succ = if let Some(succ) = succ {
                succ
            } else {
                usize::MAX
            };
            r.subs.push(Subs {
                prev,
                start,
                end,
                succ,
            });
            v += 10.0;
        }
        v += 5.0;
    }
    r.rect.set_height(v);
    r
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct SubsTimed {
    // indexing in CommitsLayout.subs
    pub(crate) prev_sub: usize,
    // indexing in CommitsLayout.commits
    pub(crate) prev: usize,
    // indexing in CommitsLayout.commits
    pub(crate) start: usize,
    // indexing in CommitsLayout.commits
    pub(crate) end: usize,
    // indexing in CommitsLayout.subs
    pub(crate) succ_sub: usize,
    // indexing in CommitsLayout.commits
    pub(crate) succ: usize,
    pub(crate) delta_time: i64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct CommitsLayoutTimed {
    pub(crate) commits: Vec<CommitId>,
    // times and lines
    pub(crate) times: Vec<i64>,
    // indexing in subs
    pub(crate) branches: Vec<usize>,
    pub(crate) subs: Vec<SubsTimed>,
    pub(crate) max_time: i64,
    pub(crate) min_time: i64,
    pub(crate) max_delta: i64,
}

impl Default for CommitsLayoutTimed {
    fn default() -> Self {
        Self {
            commits: Default::default(),
            times: Default::default(),
            branches: Default::default(),
            subs: Default::default(),
            max_time: 0,
            min_time: i64::MAX,
            // excluding subs with no prev AND succ
            max_delta: 0,
        }
    }
}

pub(crate) fn compute_commit_layout_timed(
    commits: impl Fn(&CommitId) -> Option<CommitMetadata>,
    branches: impl Iterator<Item = (String, CommitId)>,
) -> CommitsLayoutTimed {
    // let commits: Vec<commits_layouting::CommitInfo> = commits.map(|x| todo!()).collect();
    // let indices: std::collections::HashMap<commits_layouting::Oid, usize> = commits
    //     .iter()
    //     .enumerate()
    //     .map(|(i, c)| (c.oid, i))
    //     .collect();
    // let mut branches: Vec<commits_layouting::BranchInfo> = branches.map(|x| todo!()).collect();
    // let settings = commits_layouting::BranchSettings::new(branches.len());
    // branches.into_iter().map(|x| 42).collect()
    type TId = usize;
    type SId = usize;
    let mut r = CommitsLayoutTimed::default();
    let mut index = HashMap::<CommitId, (TId, SId)>::default();
    for (branch_name, target) in branches {
        // log::debug!("{} {}", branch_name, target);

        r.commits.push(branch_name);
        r.branches.push(r.subs.len());
        let branch_index = r.times.len();
        let mut waiting: Vec<(String, TId, SId)> = vec![(target, branch_index, r.subs.len())];
        // let mut prev_time = -1;
        r.times.push(-1);
        loop {
            let Some((mut current, prev, prev_sub)) = waiting.pop() else {
                break;
            };
            // h = r.pos[prev].x;
            // let
            let start = r.times.len();
            let end;
            let mut succ = None;
            loop {
                if let Some(fork) = index.get(&current) {
                    succ = Some(*fork);
                    end = r.times.len();
                    break;
                }
                index.insert(current.clone(), (r.times.len(), r.subs.len()));
                if let Some(commit) = commits(&current) {
                    // universal time then ?
                    let time = commit.time; // + commit.timezone as i64 * 60;
                    r.min_time = time.min(r.min_time);
                    r.max_time = time.max(r.max_time);
                    r.commits.push(format!("{current}"));
                    r.times.push(time);
                    if let Some(p) = commit.parents.get(0) {
                        current = p.clone();
                    } else {
                        end = r.times.len();
                        break;
                    }
                    if let Some(p) = commit.parents.get(1..) {
                        for p in p {
                            waiting.push((p.to_string(), r.times.len() - 1, r.subs.len()));
                        }
                    }
                } else {
                    r.commits.push(format!("{current}"));
                    r.times.push(-1);
                    end = r.times.len();
                    break;
                }
            }
            let delta_time;
            let (succ, succ_sub) = if let Some(succ) = succ {
                if r.times[prev] != -1 {
                    delta_time = (r.times[prev] - r.times[succ.0]).abs();
                    r.max_delta = r.max_delta.max(delta_time);
                } else {
                    delta_time = 100;
                }
                succ
            } else {
                if r.subs.is_empty() {
                    delta_time = 0;
                } else if r.times[prev] == -1 {
                    delta_time = 100;
                } else if r.times[end - 1] != -1 {
                    delta_time = (r.times[prev] - r.times[end - 1]).abs();
                    r.max_delta = r.max_delta.max(delta_time);
                } else if let Some(t) = r.times[start..end - 1].iter().rev().find(|x| **x != -1) {
                    delta_time = (r.times[prev] - t).abs();
                    r.max_delta = r.max_delta.max(delta_time);
                // } else if r.times[end - 2] != -1 {
                //     delta_time = r.times[prev] - r.times[end - 2];
                //     debug_assert!(delta_time >= 0);
                //     r.max_delta = r.max_delta.max(delta_time);
                } else {
                    delta_time = 100;
                }
                (usize::MAX, usize::MAX)
            };
            r.subs.push(SubsTimed {
                prev,
                prev_sub,
                start,
                end,
                succ,
                succ_sub,
                delta_time,
            });
        }
        r.times[branch_index] = r.times[branch_index + 1];
    }
    r
}

#[allow(unused)] // TODO check if mod is still needed
mod commits_layouting {
    use std::collections::HashMap;

    pub(super) struct Settings {
        branch_order: BranchOrder,
        branches: BranchSettings,
    }

    pub(super) struct Graph {}

    pub(super) fn compute(
        mut commits: Vec<CommitInfo>,
        indices: HashMap<Oid, usize>,
        settings: &Settings,
        branches: Vec<(String, Oid)>,
    ) -> Result<Graph, String> {
        assign_children(&mut commits, &indices);

        let mut all_branches = assign_branches(branches, &mut commits, &indices, settings)?;
        // correct_fork_merges(&commits, &indices, &mut all_branches, settings)?;
        // assign_sources_targets(&commits, &indices, &mut all_branches);

        let (shortest_first, forward) = match settings.branch_order {
            BranchOrder::ShortestFirst(fwd) => (true, fwd),
            BranchOrder::LongestFirst(fwd) => (false, fwd),
        };

        assign_branch_columns(
            &commits,
            &indices,
            &mut all_branches,
            &settings.branches,
            shortest_first,
            forward,
        );
        Ok(Graph {})
    }

    /// Walks through the commits and adds each commit's Oid to the children of its parents.
    fn assign_children(commits: &mut [CommitInfo], indices: &HashMap<Oid, usize>) {
        for idx in 0..commits.len() {
            let (oid, parents) = {
                let info = &commits[idx];
                (info.oid, info.parents)
            };
            for par_oid in &parents {
                if let Some(par_idx) = par_oid.and_then(|oid| indices.get(&oid)) {
                    commits[*par_idx].children.push(oid);
                }
            }
        }
    }

    /// Extracts branches from repository and merge summaries, assigns branches and branch traces to commits.
    ///
    /// Algorithm:
    /// * Find all actual branches (incl. target oid) and all extract branches from merge summaries (incl. parent oid)
    /// * Sort all branches by persistence
    /// * Iterating over all branches in persistence order, trace back over commit parents until a trace is already assigned
    fn assign_branches(
        branches: Vec<(String, Oid)>,
        commits: &mut [CommitInfo],
        indices: &HashMap<Oid, usize>,
        settings: &Settings,
    ) -> Result<Vec<BranchInfo>, String> {
        let mut branch_idx = 0;

        let mut branches = extract_branches(branches, commits, indices, settings)?;

        let mut index_map: Vec<_> = (0..branches.len())
            .map(|old_idx| {
                let (target, is_tag, is_merged) = {
                    let branch = &branches[old_idx];
                    (branch.target, branch.is_tag, branch.is_merged)
                };
                if let Some(&idx) = indices.get(&target) {
                    let info = &mut commits[idx];
                    if is_tag {
                        info.tags.push(old_idx);
                    } else if !is_merged {
                        info.branches.push(old_idx);
                    }
                    let oid = info.oid;
                    let any_assigned = trace_branch(commits, indices, &mut branches, oid, old_idx)
                        .unwrap_or(false);

                    if any_assigned || !is_merged {
                        branch_idx += 1;
                        Some(branch_idx - 1)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let mut commit_count = vec![0; branches.len()];
        for info in commits.iter_mut() {
            if let Some(trace) = info.branch_trace {
                commit_count[trace] += 1;
            }
        }

        let mut count_skipped = 0;
        for (idx, branch) in branches.iter().enumerate() {
            if let Some(mapped) = index_map[idx] {
                if commit_count[idx] == 0 && branch.is_merged && !branch.is_tag {
                    index_map[idx] = None;
                    count_skipped += 1;
                } else {
                    index_map[idx] = Some(mapped - count_skipped);
                }
            }
        }

        for info in commits.iter_mut() {
            if let Some(trace) = info.branch_trace {
                info.branch_trace = index_map[trace];
                for br in info.branches.iter_mut() {
                    *br = index_map[*br].unwrap();
                }
                for tag in info.tags.iter_mut() {
                    *tag = index_map[*tag].unwrap();
                }
            }
        }

        let branches: Vec<_> = branches
            .into_iter()
            .enumerate()
            .filter_map(|(arr_index, branch)| {
                if index_map[arr_index].is_some() {
                    Some(branch)
                } else {
                    None
                }
            })
            .collect();

        Ok(branches)
    }

    /// Traces back branches by following 1st commit parent,
    /// until a commit is reached that already has a trace.
    fn trace_branch(
        // repository: &Repository,
        commits: &mut [CommitInfo],
        indices: &HashMap<Oid, usize>,
        branches: &mut [BranchInfo],
        oid: Oid,
        branch_index: usize,
    ) -> Result<bool, String> {
        let mut curr_oid = oid;
        let mut prev_index: Option<usize> = None;
        let mut start_index: Option<i32> = None;
        let mut any_assigned = false;
        while let Some(index) = indices.get(&curr_oid) {
            let info = &mut commits[*index];
            if let Some(old_trace) = info.branch_trace {
                let (old_name, old_term, old_svg, old_range) = {
                    let old_branch = &branches[old_trace];
                    (
                        &old_branch.name.clone(),
                        old_branch.visual.term_color,
                        old_branch.visual.svg_color.clone(),
                        old_branch.range,
                    )
                };
                let new_name = &branches[branch_index].name;
                let old_end = old_range.0.unwrap_or(0);
                let new_end = branches[branch_index].range.0.unwrap_or(0);
                if new_name == old_name && old_end >= new_end {
                    let old_branch = &mut branches[old_trace];
                    if let Some(old_end) = old_range.1 {
                        if index > &old_end {
                            old_branch.range = (None, None);
                        } else {
                            old_branch.range = (Some(*index), old_branch.range.1);
                        }
                    } else {
                        old_branch.range = (Some(*index), old_branch.range.1);
                    }
                } else {
                    let branch = &mut branches[branch_index];
                    // if branch.name.starts_with(ORIGIN) && branch.name[7..] == old_name[..] {
                    //     branch.visual.term_color = old_term;
                    //     branch.visual.svg_color = old_svg;
                    // }
                    match prev_index {
                        None => start_index = Some(*index as i32 - 1),
                        Some(prev_index) => {
                            // TODO: in cases where no crossings occur, the rule for merge commits can also be applied to normal commits
                            // see also print::get_deviate_index()
                            if commits[prev_index].is_merge {
                                let mut temp_index = prev_index;
                                for sibling_oid in &commits[*index].children {
                                    if sibling_oid != &curr_oid {
                                        let sibling_index = indices[sibling_oid];
                                        if sibling_index > temp_index {
                                            temp_index = sibling_index;
                                        }
                                    }
                                }
                                start_index = Some(temp_index as i32);
                            } else {
                                start_index = Some(*index as i32 - 1);
                            }
                        }
                    }
                    break;
                }
            }

            info.branch_trace = Some(branch_index);
            any_assigned = true;

            let commit = &commits[indices[&curr_oid]]; //.find_commit(curr_oid)?;
            match commit.parents.len() {
                0 => {
                    start_index = Some(*index as i32);
                    break;
                }
                _ => {
                    prev_index = Some(*index);
                    curr_oid = commit.parents[0].unwrap() //parent_id(0)?;
                }
            }
        }

        let branch = &mut branches[branch_index];
        if let Some(end) = branch.range.0 {
            if let Some(start_index) = start_index {
                if start_index < end as i32 {
                    // TODO: find a better solution (bool field?) to identify non-deleted branches that were not assigned to any commits, and thus should not occupy a column.
                    branch.range = (None, None);
                } else {
                    branch.range = (branch.range.0, Some(start_index as usize));
                }
            } else {
                branch.range = (branch.range.0, None);
            }
        } else {
            branch.range = (branch.range.0, start_index.map(|si| si as usize));
        }
        Ok(any_assigned)
    }

    /// Extracts (real or derived from merge summary) and assigns basic properties.
    fn extract_branches(
        branches: Vec<(String, Oid)>,
        commits: &[CommitInfo],
        indices: &HashMap<Oid, usize>,
        settings: &Settings,
    ) -> Result<Vec<BranchInfo>, String> {
        // let filter = if settings.include_remote {
        //     None
        // } else {
        //     Some(BranchType::Local)
        // };
        // let actual_branches = repository
        //     .branches(filter)
        //     .map_err(|err| err.message().to_string())?
        //     .collect::<Result<Vec<_>, Error>>()
        //     .map_err(|err| err.message().to_string())?;

        let actual_branches = branches;

        // enum BranchType {
        //     Local, Remote
        // }

        let mut counter = 0;

        let mut valid_branches = actual_branches
            // .iter()
            .into_iter()
            // .filter_map(|(br, tp)| {
            .filter_map(|(name, target)| {
                // let Some(name) = br.get().name() else {return None};
                // name.and_then(|n| {
                let n = name;
                // let target = br.get().target();
                Some(target).map(|t| {
                    counter += 1;
                    let start_index = 11;
                    // let start_index = match tp {
                    //     BranchType::Local => 11,
                    //     BranchType::Remote => 13,
                    // };
                    let name = &n[start_index..];
                    let end_index = indices.get(&t).cloned();

                    // let term_color = match to_terminal_color(
                    //     &branch_color(
                    //         name,
                    //         &settings.branches.terminal_colors[..],
                    //         &settings.branches.terminal_colors_unknown,
                    //         counter,
                    //     )[..],
                    // ) {
                    //     Ok(col) => col,
                    //     Err(err) => return Err(err),
                    // };

                    Ok(BranchInfo::new(
                        t,
                        None,
                        name.to_string(),
                        // branch_order(name, &settings.branches.persistence) as u8,
                        0,
                        false,
                        // &BranchType::Remote == tp,
                        false,
                        false,
                        BranchVis::new(
                            0,
                            0,
                            "aabbcc".to_string(),
                            // branch_order(name, &settings.branches.order),
                            // term_color,
                            // branch_color(
                            //     name,
                            //     &settings.branches.svg_colors,
                            //     &settings.branches.svg_colors_unknown,
                            //     counter,
                            // ),
                        ),
                        end_index,
                    ))
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        // for (idx, info) in commits.iter().enumerate() {
        //     let commit = repository
        //         .find_commit(info.oid)
        //         .map_err(|err| err.message().to_string())?;
        //     if info.is_merge {
        //         if let Some(summary) = commit.summary() {
        //             counter += 1;

        //             let parent_oid = commit
        //                 .parent_id(1)
        //                 .map_err(|err| err.message().to_string())?;

        //             let branch_name = parse_merge_summary(summary, &settings.merge_patterns)
        //                 .unwrap_or_else(|| "unknown".to_string());

        //             let persistence =
        //                 branch_order(&branch_name, &settings.branches.persistence) as u8;

        //             let pos = branch_order(&branch_name, &settings.branches.order);

        //             let term_col = to_terminal_color(
        //                 &branch_color(
        //                     &branch_name,
        //                     &settings.branches.terminal_colors[..],
        //                     &settings.branches.terminal_colors_unknown,
        //                     counter,
        //                 )[..],
        //             )?;
        //             // let svg_col = branch_color(
        //             //     &branch_name,
        //             //     &settings.branches.svg_colors,
        //             //     &settings.branches.svg_colors_unknown,
        //             //     counter,
        //             // );

        //             let branch_info = BranchInfo::new(
        //                 parent_oid,
        //                 Some(info.oid),
        //                 branch_name,
        //                 persistence,
        //                 false,
        //                 true,
        //                 false,
        //                 BranchVis::new(pos, term_col, svg_col),
        //                 Some(idx + 1),
        //             );
        //             valid_branches.push(branch_info);
        //         }
        //     }
        // }

        // valid_branches.sort_by_cached_key(|branch| (branch.persistence, !branch.is_merged));

        // let mut tags = Vec::new();

        // repository
        //     .tag_foreach(|oid, name| {
        //         tags.push((oid, name.to_vec()));
        //         true
        //     })
        //     .map_err(|err| err.message().to_string())?;

        // for (oid, name) in tags {
        //     let name = std::str::from_utf8(&name[5..]).map_err(|err| err.to_string())?;

        //     let target = repository
        //         .find_tag(oid)
        //         .map(|tag| tag.target_id())
        //         .or_else(|_| repository.find_commit(oid).map(|_| oid));

        //     if let Ok(target_oid) = target {
        //         if let Some(target_index) = indices.get(&target_oid) {
        //             counter += 1;
        //             let term_col = to_terminal_color(
        //                 &branch_color(
        //                     name,
        //                     &settings.branches.terminal_colors[..],
        //                     &settings.branches.terminal_colors_unknown,
        //                     counter,
        //                 )[..],
        //             )?;
        //             let pos = branch_order(name, &settings.branches.order);
        //             let svg_col = branch_color(
        //                 name,
        //                 &settings.branches.svg_colors,
        //                 &settings.branches.svg_colors_unknown,
        //                 counter,
        //             );
        //             let tag_info = BranchInfo::new(
        //                 target_oid,
        //                 None,
        //                 name.to_string(),
        //                 settings.branches.persistence.len() as u8 + 1,
        //                 false,
        //                 false,
        //                 true,
        //                 BranchVis::new(pos, term_col, svg_col),
        //                 Some(*target_index),
        //             );
        //             valid_branches.push(tag_info);
        //         }
        //     }
        // }

        Ok(valid_branches)
    }

    /// Sorts branches into columns for visualization, that all branches can be
    /// visualized linearly and without overlaps. Uses Shortest-First scheduling.
    ///
    /// https://github.com/mlange-42/git-graph/blob/7b9bb72a310243cc53d906d1e7ec3c9aad1c75d2/src/graph.rs#L791
    pub(super) fn assign_branch_columns(
        commits: &[CommitInfo],
        indices: &HashMap<Oid, usize>,
        branches: &mut [BranchInfo],
        settings: &BranchSettings,
        shortest_first: bool,
        forward: bool,
    ) {
        let mut occupied: Vec<Vec<Vec<(usize, usize)>>> = vec![vec![]; settings.order.len() + 1];

        let length_sort_factor = if shortest_first { 1 } else { -1 };
        let start_sort_factor = if forward { 1 } else { -1 };

        let mut branches_sort: Vec<_> = branches
            .iter()
            .enumerate()
            .filter(|(_idx, br)| br.range.0.is_some() || br.range.1.is_some())
            .map(|(idx, br)| {
                (
                    idx,
                    br.range.0.unwrap_or(0),
                    br.range.1.unwrap_or(branches.len() - 1),
                    br.visual
                        .source_order_group
                        .unwrap_or(settings.order.len() + 1),
                    br.visual
                        .target_order_group
                        .unwrap_or(settings.order.len() + 1),
                )
            })
            .collect();

        branches_sort.sort_by_cached_key(|tup| {
            (
                std::cmp::max(tup.3, tup.4),
                (tup.2 as i32 - tup.1 as i32) * length_sort_factor,
                tup.1 as i32 * start_sort_factor,
            )
        });

        for (branch_idx, start, end, _, _) in branches_sort {
            let branch = &branches[branch_idx];
            let group = branch.visual.order_group;
            let group_occ = &mut occupied[group];

            let align_right = branch
                .source_branch
                .map(|src| branches[src].visual.order_group > branch.visual.order_group)
                .unwrap_or(false)
                || branch
                    .target_branch
                    .map(|trg| branches[trg].visual.order_group > branch.visual.order_group)
                    .unwrap_or(false);

            let len = group_occ.len();
            let mut found = len;
            for i in 0..len {
                let index = if align_right { len - i - 1 } else { i };
                let column_occ = &group_occ[index];
                let mut occ = false;
                for (s, e) in column_occ {
                    if start <= *e && end >= *s {
                        occ = true;
                        break;
                    }
                }
                if !occ {
                    if let Some(merge_trace) = branch
                        .merge_target
                        .and_then(|t| indices.get(&t))
                        .and_then(|t_idx| commits[*t_idx].branch_trace)
                    {
                        let merge_branch = &branches[merge_trace];
                        if merge_branch.visual.order_group == branch.visual.order_group {
                            if let Some(merge_column) = merge_branch.visual.column {
                                if merge_column == index {
                                    occ = true;
                                }
                            }
                        }
                    }
                }
                if !occ {
                    found = index;
                    break;
                }
            }

            let branch = &mut branches[branch_idx];
            branch.visual.column = Some(found);
            if found == group_occ.len() {
                group_occ.push(vec![]);
            }
            group_occ[found].push((start, end));
        }

        let group_offset: Vec<usize> = occupied
            .iter()
            .scan(0, |acc, group| {
                *acc += group.len();
                Some(*acc)
            })
            .collect();

        for branch in branches {
            if let Some(column) = branch.visual.column {
                let offset = if branch.visual.order_group == 0 {
                    0
                } else {
                    group_offset[branch.visual.order_group - 1]
                };
                branch.visual.column = Some(column + offset);
            }
        }
    }
    pub struct BranchSettings {
        order: Vec<BranchOrder>,
    }
    impl BranchSettings {
        pub(crate) fn new(len: usize) -> Self {
            Self {
                order: (0..len).map(|_| BranchOrder::ShortestFirst(true)).collect(),
            }
        }
    }
    /// Ordering policy for branches in visual columns.
    pub enum BranchOrder {
        /// Recommended! Shortest branches are inserted left-most.
        ///
        /// For branches with equal length, branches ending last are inserted first.
        /// Reverse (arg = false): Branches ending first are inserted first.
        ShortestFirst(bool),
        /// Longest branches are inserted left-most.
        ///
        /// For branches with equal length, branches ending last are inserted first.
        /// Reverse (arg = false): Branches ending first are inserted first.
        LongestFirst(bool),
    }

    pub type Oid = [u8; 20];
    /// Represents a commit.
    pub struct CommitInfo {
        pub oid: Oid,
        pub is_merge: bool,
        pub parents: [Option<Oid>; 2],
        pub children: Vec<Oid>,
        pub branches: Vec<usize>,
        pub tags: Vec<usize>,
        pub branch_trace: Option<usize>,
    }
    pub struct Commit {
        pub oid: Oid,
        pub parents: Vec<Oid>,
    }
    impl Commit {
        fn id(&self) -> Oid {
            self.oid.clone()
        }

        fn parent_count(&self) -> usize {
            self.parents.len()
        }

        fn parent_id(&self, i: usize) -> Result<Oid, ()> {
            Ok(self.parents[i].clone())
        }
    }

    impl CommitInfo {
        fn new(commit: &Commit) -> Self {
            CommitInfo {
                oid: commit.id(),
                is_merge: commit.parent_count() > 1,
                parents: [commit.parent_id(0).ok(), commit.parent_id(1).ok()],
                children: Vec::new(),
                branches: Vec::new(),
                tags: Vec::new(),
                branch_trace: None,
            }
        }
    }

    /// Represents a branch (real or derived from merge summary).
    pub struct BranchInfo {
        pub target: Oid,
        pub merge_target: Option<Oid>,
        pub source_branch: Option<usize>,
        pub target_branch: Option<usize>,
        pub name: String,
        pub persistence: u8,
        pub is_remote: bool,
        pub is_merged: bool,
        pub is_tag: bool,
        pub visual: BranchVis,
        pub range: (Option<usize>, Option<usize>),
    }
    impl BranchInfo {
        #[allow(clippy::too_many_arguments)]
        fn new(
            target: Oid,
            merge_target: Option<Oid>,
            name: String,
            persistence: u8,
            is_remote: bool,
            is_merged: bool,
            is_tag: bool,
            visual: BranchVis,
            end_index: Option<usize>,
        ) -> Self {
            BranchInfo {
                target,
                merge_target,
                target_branch: None,
                source_branch: None,
                name,
                persistence,
                is_remote,
                is_merged,
                is_tag,
                visual,
                range: (end_index, None),
            }
        }
    }

    /// Branch properties for visualization.
    pub struct BranchVis {
        /// The branch's column group (left to right)
        pub order_group: usize,
        /// The branch's merge target column group (left to right)
        pub target_order_group: Option<usize>,
        /// The branch's source branch column group (left to right)
        pub source_order_group: Option<usize>,
        /// The branch's terminal color (index in 256-color palette)
        pub term_color: u8,
        /// SVG color (name or RGB in hex annotation)
        pub svg_color: String,
        /// The column the branch is located in
        pub column: Option<usize>,
    }

    impl BranchVis {
        fn new(order_group: usize, term_color: u8, svg_color: String) -> Self {
            BranchVis {
                order_group,
                target_order_group: None,
                source_order_group: None,
                term_color,
                svg_color,
                column: None,
            }
        }
    }
}
