use std::ops::Mul as _;

use crate::app::commit;

use lazy_static::lazy_static;

use super::{
    CommitMdStore, ProjectId, QueryId, poll_md_with_pr2,
    querying::{MatchingError, StreamedDataTable},
    utils_results_batched::ComputeResultIdentified,
};

lazy_static! {
    static ref RES_PER_COMMIT: std::sync::Arc<
        std::sync::Mutex<
            crate::app::utils_commit::BorrowFrameCache<
                super::ResultsPerCommit,
                ComputeResPerCommit,
            >,
        >,
    > = Default::default();
}

lazy_static! {
    static ref LAYOUT: std::sync::Arc<
        std::sync::Mutex<
            crate::app::utils_commit::BorrowFrameCache<commit::CommitsLayoutTimed, ComputeLayout>,
        >,
    > = Default::default();
}

#[derive(Default)]
pub struct ComputeResPerCommit {}

type ResHash = u64;

impl
    egui::util::cache::ComputerMut<
        (
            (ProjectId, QueryId, ResHash),
            // &crate::app::utils_results_batched::ComputeResults,
            &StreamedDataTable<
                Vec<String>,
                std::result::Result<ComputeResultIdentified, MatchingError>,
            >,
        ),
        super::ResultsPerCommit,
    > for ComputeResPerCommit
{
    fn compute(
        &mut self,
        ((pid, qid, _), r): (
            (ProjectId, QueryId, ResHash),
            // &crate::app::utils_results_batched::ComputeResults,
            &StreamedDataTable<
                Vec<String>,
                std::result::Result<ComputeResultIdentified, MatchingError>,
            >,
        ),
    ) -> super::ResultsPerCommit {
        let mut res = super::ResultsPerCommit::default();

        update_results_per_commit(&mut res, r);
        res
    }
}

#[derive(Default)]
pub struct ComputeLayout {}

impl
    egui::util::cache::ComputerMut<
        ((&str, &str, usize), &super::CommitMdStore),
        commit::CommitsLayoutTimed,
    > for ComputeLayout
{
    fn compute(
        &mut self,
        ((name, target, _), fetched_commit_metadata): ((&str, &str, usize), &super::CommitMdStore),
    ) -> commit::CommitsLayoutTimed {
        commit::compute_commit_layout_timed(
            |id| fetched_commit_metadata.get(id)?.as_ref().ok().cloned(),
            Some((name.to_string(), target.to_string())).into_iter(),
        )
    }
}

impl crate::HyperApp {
    pub(crate) fn print_commit_graph_timed(&mut self, ui: &mut egui::Ui) {
        let res_per_commit = &mut RES_PER_COMMIT.lock().unwrap();
        let layout_cache = &mut LAYOUT.lock().unwrap();
        let mut caches_to_clear = vec![];
        ui.add_space(20.0);
        let ready_count = self.data.fetched_commit_metadata.len_local();
        for repo_id in self.data.selected_code_data.project_ids() {
            let Some((r, mut commit_slice)) = self.data.selected_code_data.get_mut(repo_id) else {
                continue;
            };
            let Some(branch) = commit_slice
                .iter_mut()
                .next()
                .map(|c| (format!("{}/{}", r.user, r.name), c.clone()))
            else {
                continue;
            };
            let id = ui.make_persistent_id("bottom_cache_layout");
            let r = r.clone();

            let results_per_commit: Option<&super::ResultsPerCommit> = {
                if let Some(r) = self
                    .data
                    .queries_results
                    .iter()
                    .find(|x| x.project == repo_id)
                {
                    let qid = r.query;
                    if let Some(Ok(r)) = r.content.get() {
                        let key = { (repo_id, qid, r.rows.lock().unwrap().0) };
                        Some(res_per_commit.get2(key, r))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            let cached = {
                if self.data.fetched_commit_metadata.is_absent(&branch.1) {
                    let commit = r.clone().with(&branch.1);
                    let fetching = commit::fetch_commit(ui.ctx(), &self.data.api_addr, &commit);
                    self.data
                        .fetched_commit_metadata
                        .insert(commit.id, fetching);
                }
                layout_cache.get2(
                    (
                        &branch.0,
                        &branch.1,
                        ready_count, // TODO find something more reliable
                    ),
                    &self.data.fetched_commit_metadata,
                )
            };

            let mut to_fetch = vec![];
            let mut to_poll = vec![];

            if true {
                let resp = show_commit_graph_timed_egui_plot(
                    ui,
                    self.data.max_fetch,
                    &self.data.fetched_commit_metadata,
                    results_per_commit,
                    cached,
                    repo_id,
                    &mut to_fetch,
                    &mut to_poll,
                );

                match resp.inner {
                    GraphInteration::ClickCommit(i) => {
                        if resp.response.secondary_clicked() && ui.input(|i| i.modifiers.command) {
                            let mut it = commit_slice.iter_mut();
                            *it.next().unwrap() = cached.commits[i].clone();
                            for _ in 0..it.count() {
                                commit_slice.pop();
                            }
                        } else if resp.response.clicked() {
                            self.selected_commit = Some((repo_id, cached.commits[i].to_string()));
                            self.selected_baseline = None;
                            let tabid = self
                                .data
                                .queries_results
                                .iter()
                                .find(|x| x.project == repo_id && x.query == 0)
                                .unwrap()
                                .tab;
                            if let super::Tab::QueryResults { id, format } =
                                &mut self.tabs[tabid as usize]
                            {
                                *format = crate::app::ResultFormat::Table
                            } else {
                                panic!()
                            }
                        } else if resp.response.secondary_clicked() {
                            let commit = format!(
                                "https://github.com/{}/{}/commit/{}",
                                r.user, r.name, cached.commits[i]
                            );
                            ui.ctx().copy_text(commit.to_string());
                            self.notifs.success(format!(
                                "Copied address of github commit to clipboard\n{}",
                                commit
                            ));
                            let id = &cached.commits[i];
                            let repo = r.clone();
                            let id = id.clone();
                            let commit = crate::app::types::Commit { repo, id };
                            let md = self.data.fetched_commit_metadata.remove(&commit.id);
                            log::debug!("fetch_merge_pr");
                            let waiting = commit::fetch_merge_pr(
                                ui.ctx(),
                                &self.data.api_addr,
                                &commit,
                                md.unwrap().unwrap().clone(),
                                repo_id,
                            );
                            self.data.fetched_commit_metadata.insert(commit.id, waiting);
                        }
                    }
                    GraphInteration::ClickErrorFetch(i) => {
                        for i in i {
                            let id = &cached.commits[i];
                            let repo = r.clone();
                            let id = id.clone();
                            let commit = crate::app::types::Commit { repo, id };
                            let v = commit::fetch_commit(ui.ctx(), &self.data.api_addr, &commit);
                            self.data.fetched_commit_metadata.insert(commit.id, v);
                        }
                    }
                    GraphInteration::ClickChange(i, after) => {
                        let commit = format!(
                            "https://github.com/{}/{}/commit/{}",
                            r.user, r.name, cached.commits[after]
                        );
                        self.notifs.add_log(re_log::LogMsg {
                            level: log::Level::Info,
                            target: format!("graph/commits"),
                            msg: format!("Selected\n{} vs {}", commit, cached.commits[i]),
                        });
                        if resp.response.clicked() {
                            self.selected_baseline = Some(cached.commits[i].to_string());
                            self.selected_commit =
                                Some((repo_id, cached.commits[after].to_string()));
                            // assert_eq!(self.data.queries.len(), 1); // need to retieve current query if multiple

                            for q in &self.data.queries {
                                dbg!(&q.lang);
                                dbg!(&q.name);
                                dbg!(q.max_matches);
                            }
                            let tabid = self
                                .data
                                .queries_results
                                .iter()
                                .find(|x| {
                                    x.project == repo_id
                                        && self.data.queries[x.query as usize].lang == "Java"
                                })
                                .unwrap()
                                .tab;
                            if let super::Tab::QueryResults { id, format } =
                                &mut self.tabs[tabid as usize]
                            {
                                *format = crate::app::ResultFormat::Hunks
                            } else {
                                panic!()
                            }
                        } else if resp.response.secondary_clicked() {
                            ui.ctx().copy_text(commit.to_string());
                            self.notifs.success(format!(
                                "Copied address of github commit to clipboard\n{}",
                                commit
                            ));
                        }
                    }
                    _ => (),
                }
            } else {
                show_commit_graph_timed_custom(
                    ui,
                    self.data.offset_fetch,
                    self.data.max_fetch,
                    &self.data.fetched_commit_metadata,
                    results_per_commit,
                    cached,
                    repo_id,
                    &mut to_fetch,
                    &mut to_poll,
                    &mut self.selected_commit,
                );
            }

            for id in to_fetch {
                if !self.data.fetched_commit_metadata.is_absent(id) {
                    continue;
                }
                let repo = r.clone();
                let id = id.clone();
                let commit = crate::app::types::Commit { repo, id };
                let v = commit::fetch_commit(ui.ctx(), &self.data.api_addr, &commit);
                self.data.fetched_commit_metadata.insert(commit.id, v);
            }
            let max_time = cached.max_time;
            for id in to_poll {
                let was_err = self
                    .data
                    .fetched_commit_metadata
                    .get(id)
                    .map_or(false, |x| x.is_err());

                if self.data.fetched_commit_metadata.try_poll_with(id, |x| {
                    x.map(|x| poll_md_with_pr2(x, repo_id, &mut commit_slice))
                }) {
                    if let Some(Ok(md)) = self.data.fetched_commit_metadata.get(id) {
                        let md = md.clone();
                        if was_err {
                            caches_to_clear.push(repo_id);
                        }
                        if md.forth_timestamp == i64::MAX {
                            continue;
                        }
                        if md.ancestors.is_empty() {
                            continue;
                        }
                        let id1 = md.ancestors[0].clone();
                        let id2 = md.ancestors.get(1).cloned();
                        let forth_timestamp = md.forth_timestamp;
                        if max_time - forth_timestamp < self.data.max_fetch
                            && self.data.fetched_commit_metadata.is_absent(&id1)
                        {
                            let id = id1;
                            if self.data.fetched_commit_metadata.is_waiting(&id) {
                                if self.data.fetched_commit_metadata.try_poll_with(&id, |x| {
                                    x.map(|x| poll_md_with_pr2(x, repo_id, &mut commit_slice))
                                }) {
                                    let repo = r.clone();
                                    let commit = crate::app::types::Commit {
                                        repo,
                                        id: id.clone(),
                                    };
                                    log::debug!("fetch_merge_pr");
                                    let waiting = commit::fetch_merge_pr(
                                        ui.ctx(),
                                        &self.data.api_addr,
                                        &commit,
                                        md.clone(),
                                        repo_id,
                                    );
                                    self.data
                                        .fetched_commit_metadata
                                        .insert(id.to_string(), waiting);
                                }
                            } else {
                                let repo = r.clone();
                                let commit = crate::app::types::Commit {
                                    repo,
                                    id: id.clone(),
                                };
                                let v =
                                    commit::fetch_commit(ui.ctx(), &self.data.api_addr, &commit);
                                self.data.fetched_commit_metadata.insert(id, v);
                            }
                        }
                        let Some(id2) = id2 else {
                            continue;
                        };
                        if max_time - forth_timestamp < self.data.max_fetch
                            && self.data.fetched_commit_metadata.is_absent(&id2)
                        {
                            let id = id2;
                            if self.data.fetched_commit_metadata.is_waiting(&id) {
                                if self.data.fetched_commit_metadata.try_poll_with(&id, |x| {
                                    x.map(|x| poll_md_with_pr2(x, repo_id, &mut commit_slice))
                                }) {
                                    let repo = r.clone();
                                    let commit = crate::app::types::Commit {
                                        repo,
                                        id: id.clone(),
                                    };
                                    log::debug!("fetch_merge_pr");
                                    let waiting = commit::fetch_merge_pr(
                                        ui.ctx(),
                                        &self.data.api_addr,
                                        &commit,
                                        md.clone(),
                                        repo_id,
                                    );
                                    self.data
                                        .fetched_commit_metadata
                                        .insert(id.to_string(), waiting);
                                }
                            } else {
                                let repo = r.clone();
                                let commit = crate::app::types::Commit {
                                    repo,
                                    id: id.clone(),
                                };
                                let v =
                                    commit::fetch_commit(ui.ctx(), &self.data.api_addr, &commit);
                                self.data.fetched_commit_metadata.insert(id, v);
                            }
                        }
                        let repo = r.clone();
                        let commit = crate::app::types::Commit {
                            repo,
                            id: id.clone(),
                        };
                        log::debug!("fetch_merge_pr");
                        let waiting = commit::fetch_merge_pr(
                            ui.ctx(),
                            &self.data.api_addr,
                            &commit,
                            md.clone(),
                            repo_id,
                        );
                        self.data
                            .fetched_commit_metadata
                            .insert(id.to_string(), waiting);
                    }
                }
            }
        }
        for repo_id in caches_to_clear {
            let Some((r, mut c)) = self.data.selected_code_data.get_mut(repo_id) else {
                continue;
            };
            let commit_slice = &mut c.iter_mut();
            let Some(branch) = commit_slice
                .next()
                .map(|c| (format!("{}/{}", r.user, r.name), c.clone()))
            else {
                continue;
            };
            if let Some(r) = self
                .data
                .queries_results
                .iter()
                .find(|x| x.project == repo_id)
            {
                let qid = r.query;
                if let Some(Ok(r)) = r.content.get() {
                    let key = { (repo_id, qid, r.rows.lock().unwrap().0) };
                    res_per_commit.remove(key);
                }
            }
            layout_cache.remove((
                &branch.0,
                &branch.1,
                ready_count, // TODO find something more reliable
            ));
        }
        ui.add_space(20.0);
        res_per_commit.evice_cache();
        layout_cache.evice_cache();
    }
}

enum GraphInteration {
    None,
    ClickCommit(usize),
    ClickChange(usize, usize),
    ClickErrorFetch(Vec<usize>),
}

const CUSTOM_LABEL_FORMAT_MARK_WITH_DATA: &str = "_d";
const CUSTOM_LABEL_FORMAT_MARK_NO_DATA: &str = "_";

fn show_commit_graph_timed_egui_plot<'a>(
    ui: &mut egui::Ui,
    max_fetch: i64,
    fetched_commit_metadata: &CommitMdStore,
    results_per_commit: Option<&super::ResultsPerCommit>,
    cached: &'a commit::CommitsLayoutTimed,
    repo_id: ProjectId,
    to_fetch: &mut Vec<&'a String>,
    to_poll: &mut Vec<&'a String>,
) -> egui_plot::PlotResponse<GraphInteration> {
    const DIFF_VALS: bool = true;
    const LEFT_VALS: bool = false;
    const RIGHT_VALS: bool = false;
    let diff_val_col = if ui.visuals().dark_mode {
        egui::Color32::YELLOW
    } else {
        egui::Color32::RED
    };
    egui::Frame::NONE
        .inner_margin(egui::vec2(50.0, 10.0))
        .show(ui, |ui| {
            // TODO now use egui_plot, it will handle interation properly and should not be difficult to migrate I think.
            use egui_plot::*;
            let resp = Plot::new(repo_id)
                .view_aspect(8.0)
                .show_axes([true, false])
                .allow_zoom([true, false])
                .x_axis_formatter(|m, _range| {
                    let v = m.value as i64 - cached.max_time;
                    if v == 0 {
                        format!("0")
                    } else if m.step_size as i64 > 60 * 60 * 24 * 364 {
                        format!("{:+}y", v / (60 * 60 * 24 * 364))
                    } else if m.step_size as i64 > 60 * 60 * 24 * 20 {
                        format!("{:+}M", v / (60 * 60 * 24 * 30))
                    } else if m.step_size as i64 > 60 * 60 * 24 * 6 {
                        format!("{:+}w", v / (60 * 60 * 24 * 7))
                    } else if m.step_size as i64 > 60 * 60 * 20 {
                        format!("{:+}d", v / (60 * 60 * 24))
                    } else {
                        format!("{:+}h", v / (60 * 60))
                    }
                })
                // .x_axis_label("time")
                .x_grid_spacer(|i| with_egui_plot::compute_multi_x_marks(i, cached.max_time))
                .show_y(false)
                .y_axis_formatter(|m, _| Default::default())
                .show_grid([true, false])
                .set_margin_fraction(egui::vec2(0.1, 0.3))
                .allow_scroll([true, false])
                // .coordinates_formatter(
                //     Corner::RightBottom,
                //     CoordinatesFormatter::new(|p, b| format!("42")),
                // )
                .label_formatter(|name, value| {
                    fn msg(x: Option<&Result<commit::CommitMetadata, String>>) -> Option<&str> {
                        x?.as_ref().ok()?.message.as_ref().map(|s| s.as_str())
                    }
                    if name == CUSTOM_LABEL_FORMAT_MARK_WITH_DATA {
                        let i = value.x.to_bits() as usize;
                        let c = &cached.commits[i];
                        let s = results_per_commit
                            .and_then(|x| x.offset(c.as_str()).map(|o| x.vals_to_string(o)));

                        format!(
                            "{}\n{}\n\n{}",
                            &c[..6],
                            s.unwrap_or_default(),
                            msg(fetched_commit_metadata.get(c)).unwrap_or_default()
                        )
                    } else if name == CUSTOM_LABEL_FORMAT_MARK_NO_DATA {
                        let i = value.x.to_bits() as usize;
                        let c = &cached.commits[i];
                        format!(
                            "{}\n\n{}",
                            &c[..6],
                            msg(fetched_commit_metadata.get(c)).unwrap_or_default()
                        )
                    } else {
                        name.to_string()
                    }
                })
                .show(ui, |plot_ui| {
                    let mut ouput = GraphInteration::None;
                    let mut offsets = vec![];
                    let mut offsets2 = vec![];
                    let mut points_with_data = vec![];
                    let mut points = vec![];
                    'subs: for sub @ commit::SubsTimed {
                        prev,
                        prev_sub,
                        start,
                        end,
                        succ,
                        succ_sub,
                        delta_time,
                    } in &cached.subs
                    {
                        const CORNER: bool = true;
                        let mut line = vec![];

                        let prev_p = [
                            if cached.times[*prev] == -1 {
                                -1
                            } else {
                                let t = cached.times[*prev];
                                if cached.max_time - t > max_fetch {
                                    continue 'subs;
                                }
                                t
                            },
                            with_egui_plot::transform_y(cached.subs[*prev_sub].delta_time),
                        ];
                        line.push(prev_p.map(|x| x as f64));
                        for i in *start..*end {
                            let t = cached.times[i];
                            if t == -1 {
                                if fetched_commit_metadata.is_absent(&cached.commits[i]) {
                                    to_fetch.push(&cached.commits[i]);
                                } else if let Some(a) =
                                    fetched_commit_metadata.get(&cached.commits[i])
                                {
                                    match a {
                                        Err(e) => {
                                            to_poll.push(&cached.commits[i]);
                                            let plot_point = [
                                                cached.times[*prev] as f64,
                                                with_egui_plot::transform_y(*delta_time) as f64
                                                    + 30.0,
                                            ];
                                            if plot_ui.response().clicked() {
                                                let point = plot_ui.response().hover_pos().unwrap();
                                                let pos = plot_ui
                                                    .transform()
                                                    .position_from_point(&plot_point.into());
                                                let dist_sq = point.distance_sq(pos);
                                                if dist_sq < 100.0 {
                                                    log::error!("should reload");
                                                    if let GraphInteration::None = ouput {
                                                        ouput =
                                                            GraphInteration::ClickErrorFetch(vec![
                                                                i,
                                                            ]);
                                                    } else if let GraphInteration::ClickErrorFetch(
                                                        v,
                                                    ) = &mut ouput
                                                    {
                                                        v.push(i)
                                                    }
                                                }
                                            }
                                            let series: Vec<[f64; 2]> = vec![plot_point];
                                            let points = Points::new("error", series)
                                                .radius(4.0)
                                                .color(egui::Color32::RED)
                                                .name(format!(
                                                    "Error getting {}:\n{e}",
                                                    cached.commits[i]
                                                ));
                                            plot_ui.add(points);
                                            // to_fetch.push(cached.commits[i]);
                                        }
                                        _ => (),
                                    }
                                } else {
                                    to_poll.push(&cached.commits[i]);
                                }
                                break;
                            }
                            let commit = &cached.commits[i];

                            let before = if i != *start {
                                Some(cached.commits[i - 1].as_str())
                            } else if *prev != usize::MAX {
                                Some(cached.commits[*prev].as_str())
                            } else {
                                None
                            };
                            let after = if i + 1 < *end {
                                Some(cached.commits[i + 1].as_str())
                            } else if *succ != usize::MAX {
                                Some(cached.commits[*succ].as_str())
                            } else {
                                None
                            };
                            let diff = results_per_commit
                                .zip(before)
                                .and_then(|(x, c1)| x.try_diff_as_string(c1, commit));

                            let y = with_egui_plot::transform_y(*delta_time);
                            let mut p = [t, y];
                            if *start == i {
                                let corner = [
                                    (p[0] as f64).max(
                                        line.last().unwrap()[0]
                                            - plot_ui.transform().dvalue_dpos()[0] * 10.0,
                                    ),
                                    p[1] as f64,
                                ];
                                line.push(corner);
                                if let Some(text) = DIFF_VALS.then_some(()).and(diff) {
                                    plot_ui.text(
                                        Text::new("diff", corner.into(), text)
                                            .anchor(egui::Align2::RIGHT_BOTTOM)
                                            .color(diff_val_col),
                                    );

                                    if plot_ui.response().clicked() {
                                        let point = plot_ui.response().hover_pos().unwrap();
                                        let plot_point = PlotPoint::new(corner[0], corner[1]);
                                        let pos =
                                            plot_ui.transform().position_from_point(&plot_point);
                                        let dist_sq = point.distance_sq(pos);
                                        if dist_sq < 100.0 {
                                            log::error!("clicked");
                                            ouput = GraphInteration::ClickChange(i, *prev);
                                        }
                                    }
                                }
                            } else {
                                if t > cached.times[i - 1] {
                                    p[1] += 100;
                                }
                                let a = line.last().unwrap().clone();
                                let b = p.map(|x| x as f64);
                                let position = with_egui_plot::center(a, b);
                                if let Some(text) = DIFF_VALS.then_some(()).and(diff) {
                                    plot_ui.text(
                                        Text::new("diff val", position, text)
                                            .anchor(egui::Align2::RIGHT_BOTTOM)
                                            .color(diff_val_col),
                                    );
                                    if plot_ui.response().clicked() {
                                        let point = plot_ui.response().hover_pos().unwrap();
                                        let pos =
                                            plot_ui.transform().position_from_point(&position);
                                        let dist_sq = point.distance_sq(pos);
                                        if dist_sq < 100.0 {
                                            log::debug!("clicked");
                                            ouput = GraphInteration::ClickChange(i, i - 1);
                                        }
                                    }
                                }
                            }
                            line.push(p.map(|x| x as f64));

                            // stop rendering when reached limit
                            if cached.max_time - t > max_fetch {
                                let line = Line::new("last line", line).allow_hover(false);
                                plot_ui.line(line);
                                continue 'subs;
                            }

                            if i == 1 {
                                let text = results_per_commit.and_then(|x| {
                                    x.offset(commit).map(|offset| x.vals_to_string(offset))
                                });
                                if let Some(text) = text {
                                    plot_ui.text(
                                        Text::new("text last", p.map(|x| x as f64).into(), text)
                                            .anchor(egui::Align2::RIGHT_BOTTOM)
                                            .color(egui::Color32::GRAY),
                                    );
                                }
                            }

                            let vals_offset = results_per_commit.and_then(|x| {
                                x.offset_with_variation(
                                    commit.as_str(),
                                    before,
                                    Some(commit.as_str()),
                                )
                            });
                            if let Some(offset) = LEFT_VALS.then_some(()).and(vals_offset) {
                                let text = results_per_commit.unwrap().vals_to_string(offset);
                                plot_ui.text(
                                    Text::new("left vals", p.map(|x| x as f64).into(), text)
                                        .anchor(egui::Align2::RIGHT_BOTTOM)
                                        .color(egui::Color32::GRAY),
                                );
                            }

                            if results_per_commit
                                .and_then(|x| x._get_offset(commit))
                                .is_some()
                            {
                                points_with_data.push(p.map(|x| x as f64));
                                offsets.push(i as u32);
                            } else {
                                points.push(p.map(|x| x as f64));
                                offsets2.push(i as u32);
                            }
                        }

                        if *succ < usize::MAX && cached.times[*succ] != -1 {
                            let y = cached.subs[*succ_sub].delta_time;
                            let y = with_egui_plot::transform_y(y);
                            let x = cached.times[*succ];
                            let position: PlotPoint;
                            let p = [x, y].map(|x| x as f64);
                            if CORNER {
                                let prev = line.last().unwrap();
                                let x = p[0];
                                let x = x + plot_ui.transform().dvalue_dpos()[0] * 10.0;
                                let x = prev[0].min(x);
                                // let y = prev[1];
                                let y = prev[1];
                                let y = if (y - p[1]).abs() < 1.0 {
                                    y - 1000.0 // + plot_ui.transform().dvalue_dpos()[1] * 2.0 * (y - p[1]).signum()
                                } else if y < p[1] {
                                    y - plot_ui.transform().dvalue_dpos()[1] * 5.0
                                } else {
                                    y + plot_ui.transform().dvalue_dpos()[1] * 5.0
                                };
                                // let y = p[1].min(y);
                                // let y = if (p[1] - prev[1]).abs() < 1.0 {
                                // } else {
                                //     prev[1] + plot_ui.transform().dvalue_dpos()[1] * 10.0
                                // };
                                let corner = [x, y];
                                position = corner.into();
                                line.push(corner);
                                line.push(p);
                            } else {
                                position = with_egui_plot::center(*line.last().unwrap(), p);
                                line.push(p);
                            }

                            let c1 = if start == end { *prev } else { *end - 1 };
                            let diff = results_per_commit.and_then(|x| {
                                x.try_diff_as_string(&cached.commits[c1], &cached.commits[*succ])
                            });
                            if let Some(text) = DIFF_VALS.then_some(()).and(diff) {
                                plot_ui.text(
                                    Text::new("diff vals", position, text)
                                        .anchor(egui::Align2::RIGHT_BOTTOM)
                                        .color(egui::Color32::RED),
                                );

                                if plot_ui.response().clicked() {
                                    let point = plot_ui.response().hover_pos().unwrap();
                                    let plot_point = position;
                                    let pos = plot_ui.transform().position_from_point(&plot_point);
                                    let dist_sq = point.distance_sq(pos);
                                    if dist_sq < 100.0 {
                                        log::error!("clicked");
                                        log::error!(
                                            "{} {} {} {}\n{} {} {} {}",
                                            cached.commits[*prev],
                                            cached.commits[end - 1],
                                            cached.commits[*start],
                                            cached.commits[*succ],
                                            prev,
                                            end,
                                            start,
                                            succ,
                                        );
                                        ouput = GraphInteration::ClickChange(*succ, c1);
                                    }
                                }
                            }
                        }

                        let line = Line::new("line", line).allow_hover(false);
                        plot_ui.line(line);
                    }

                    let points = Points::new("commit", points)
                        .radius(2.0)
                        .color(egui::Color32::GREEN)
                        .name("Commit");

                    let item = with_egui_plot::CommitPoints {
                        offsets: offsets2,
                        points,
                        with_data: false,
                    };
                    let has_any_click = plot_ui
                        .response()
                        .flags
                        .contains(egui::response::Flags::CLICKED);
                    if has_any_click {
                        if let Some(x) = item.find_closest(
                            plot_ui.response().hover_pos().unwrap(),
                            plot_ui.transform(),
                        ) {
                            if x.dist_sq < 10.0 {
                                let i = item.offsets[x.index] as usize;
                                ouput = GraphInteration::ClickCommit(i);
                                // *selected_commit = Some((repo_id, cached.commits[i].to_string()));
                                // plot_ui
                                //     .ctx()
                                //     .output_mut(|r| r.copied_text = cached.commits[i].to_string());
                            }
                        }
                    }
                    plot_ui.add(item);
                    let points = Points::new("commit with data", points_with_data)
                        .radius(2.0)
                        .color(egui::Color32::DARK_GREEN)
                        .name("Commit with data");
                    let item = with_egui_plot::CommitPoints {
                        offsets,
                        points,
                        with_data: true,
                    };
                    if has_any_click {
                        if let Some(x) = item.find_closest(
                            plot_ui.response().hover_pos().unwrap(),
                            plot_ui.transform(),
                        ) {
                            if x.dist_sq < 10.0 {
                                let i = item.offsets[x.index] as usize;
                                ouput = GraphInteration::ClickCommit(i);
                                // *selected_commit = Some((repo_id, cached.commits[i].to_string()));
                                // plot_ui
                                //     .ctx()
                                //     .output_mut(|r| r.copied_text = cached.commits[i].to_string());
                            }
                        }
                    }
                    plot_ui.add(item);

                    for &b in &cached.branches {
                        let b = cached.subs[b].prev;
                        let y = cached.subs[b].delta_time;
                        let y = with_egui_plot::transform_y(y);
                        let position = [cached.times[b] as f64, y as f64].into();
                        let text = &cached.commits[b];
                        let text =
                            Text::new("branch name", position, text).anchor(egui::Align2::LEFT_TOP);
                        plot_ui.text(text);
                    }
                    ouput
                });
            if resp.response.secondary_clicked() {
                log::error!("secondary_clicked");
            }
            if resp.response.clicked() {
                log::error!("clicked");
            }
            if resp.response.double_clicked() {
                log::error!("double_clicked");
            }
            // if let Some(id) = &resp.hovered_plot_item {
            //     if resp.response.clicked() {}
            // }
            resp
        })
        .inner
}

fn show_commit_graph_timed_custom<'a>(
    ui: &mut egui::Ui,
    offset_fetch: i64,
    max_fetch: i64,
    fetched_commit_metadata: &CommitMdStore,
    results_per_commit: Option<&super::ResultsPerCommit>,
    cached: &'a commit::CommitsLayoutTimed,
    repo_id: ProjectId,
    to_fetch: &mut Vec<&'a String>,
    to_poll: &mut Vec<&'a String>,
    selected_commit: &mut Option<(ProjectId, String)>,
) {
    let parent_rel_color = if ui.visuals().dark_mode {
        egui::Color32::WHITE
    } else {
        egui::Color32::BLACK
    };
    let diff_val_col = if ui.visuals().dark_mode {
        egui::Color32::YELLOW
    } else {
        egui::Color32::RED
    };
    let max_time = cached.min_time.max(cached.max_time - offset_fetch);
    let min_time = cached.min_time.max(max_time - max_fetch);
    let cached = cached;
    let width = ui.available_width() - 120.0;
    let _time_norm = |t: i64| t as f32 / (max_time - min_time).max(1) as f32;
    let _time_scaling = |t: i64| _time_norm(t) * width;
    assert!(cached.max_delta >= 0);
    let time_scaling = |t: i64| _time_scaling(t - min_time);
    let time_scaling_y = |t: i64| {
        _time_norm(t)
            .clamp(0.0, 1.0)
            .sqrt()
            .mul((width / 6.0).min(200.0))
            .max(0.0)
    };

    assert!(cached.max_delta < i64::MAX);
    let desired_size = egui::vec2(
        ui.available_width(),
        if cached.max_delta <= 0 {
            0.0
        } else {
            time_scaling_y(cached.max_delta)
        },
    ) + egui::vec2(0.0, 60.0);
    let (_rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let rect = _rect.shrink(30.0);
    let min = rect.min;

    // background
    ui.painter().rect_filled(
        _rect
            .shrink2(egui::vec2(55.0, 0.0))
            .translate(egui::vec2(-30.0, 0.0)),
        ui.visuals().window_corner_radius,
        ui.visuals().extreme_bg_color,
    );

    // region:   --- Ticks
    const SECONDS_PER_DAY: i64 = 60 * 60 * 24;
    let day_count = (max_time - min_time) / (SECONDS_PER_DAY);
    let day_width = _time_scaling(SECONDS_PER_DAY);
    let day_offset = min.x + _time_scaling(min_time % (SECONDS_PER_DAY));
    for d in 0..day_count {
        let x = day_offset + d as f32 * day_width;
        ui.painter().line_segment(
            [egui::pos2(x, _rect.min.y), egui::pos2(x, _rect.min.y + 5.0)],
            egui::Stroke::new(2.0, egui::Color32::GRAY),
        );
    }
    // endregion --- Ticks

    assert!(cached.branches.len() <= 1);
    'subs: for commit::SubsTimed {
        prev,
        prev_sub,
        start,
        end,
        succ,
        succ_sub,
        delta_time,
    } in &cached.subs
    {
        let mut prev_p = min
            + egui::vec2(
                if cached.times[*prev] == -1 {
                    0.0
                } else {
                    time_scaling(cached.times[*prev])
                },
                time_scaling_y(cached.subs[*prev_sub].delta_time),
            );
        let curr_y = min.y + time_scaling_y(*delta_time);
        for i in *start..*end {
            let t = cached.times[i];
            if t == -1 {
                if fetched_commit_metadata.is_absent(&cached.commits[i]) {
                    to_fetch.push(&cached.commits[i]);
                } else if fetched_commit_metadata.get(&cached.commits[i]).is_none() {
                    to_poll.push(&cached.commits[i]);
                }
                break;
            }
            let center = egui::pos2(min.x + time_scaling(t), curr_y);

            let commit = &cached.commits[i];

            // region: painter queried values when there is a change
            let before = if i != *start {
                Some(cached.commits[i - 1].as_str())
            } else if *prev != usize::MAX {
                Some(cached.commits[*prev].as_str())
            } else {
                None
            };
            let after = if i + 1 < *end {
                Some(cached.commits[i + 1].as_str())
            } else if *succ != usize::MAX {
                Some(cached.commits[*succ].as_str())
            } else {
                None
            };

            let circle_fill_color = if results_per_commit
                .and_then(|x| x._get_offset(commit))
                .is_some()
            {
                egui::Color32::DARK_GREEN
            } else {
                egui::Color32::GREEN
            };

            let diff = results_per_commit
                .zip(before)
                .and_then(|(x, c2)| x.try_diff_as_string(commit, c2));

            if i == *start {
                let corner_p = egui::pos2(prev_p.x.max(center.x - 10.0), center.y);
                let parent_rel_color = egui::Color32::LIGHT_GRAY;
                let stroke = egui::Stroke::new(2.0, parent_rel_color);
                ui.painter().line_segment([prev_p, corner_p], stroke);
                ui.painter().line_segment([corner_p, center], stroke);
                if let Some(text) = diff {
                    let pos = corner_p;
                    let font_id = egui::TextStyle::Body.resolve(ui.style());
                    let anchor = egui::Align2::RIGHT_BOTTOM;
                    ui.painter().text(pos, anchor, text, font_id, diff_val_col);
                }
            } else {
                ui.painter()
                    .line_segment([prev_p, center], egui::Stroke::new(2.0, parent_rel_color));
                if let Some(text) = diff {
                    let pos = egui::Rect::from_min_max(prev_p, center).center();
                    let font_id = egui::TextStyle::Body.resolve(ui.style());
                    let anchor = egui::Align2::RIGHT_BOTTOM;
                    ui.painter().text(pos, anchor, text, font_id, diff_val_col);
                }
            }

            // stop rendering when reached limit
            if max_time - t > max_fetch {
                let fill_color = egui::Color32::RED;
                ui.painter().circle_filled(center, 10.0, fill_color);
                continue 'subs;
            }

            ui.painter().circle_filled(center, 4.0, circle_fill_color);

            let resp = ui.interact(
                egui::Rect::from_center_size(center, egui::Vec2::splat(8.0)),
                ui.id().with(&cached.commits[i]),
                egui::Sense::click(),
            );
            let vals_offset = results_per_commit
                .and_then(|x| x.offset_with_variation(commit.as_str(), before, after));
            if let Some(offset) = vals_offset {
                let pos = center + (0.0, -10.0).into();
                let text_color = ui.style().visuals.text_color();
                let text = results_per_commit.unwrap().vals_to_string(offset);
                let font_id = egui::TextStyle::Body.resolve(ui.style());
                let anchor = egui::Align2::RIGHT_BOTTOM;
                // let rect = egui::Align2::RIGHT_BOTTOM.anchor_size(pos, galley.size());
                // ui.painter().galley(rect.min, galley.clone(), text_color);
                ui.painter().text(pos, anchor, text, font_id, text_color);
            }
            // endregion

            // similar to a tooltip
            let resp = resp.on_hover_ui(|ui| {
                let vals_offset = results_per_commit.and_then(|x| x.offset(commit.as_str()));
                let text = if let Some(v) = vals_offset {
                    format!(
                        "{}\n{}",
                        commit,
                        results_per_commit.unwrap().vals_to_string(v)
                    )
                } else {
                    format!("{}", commit)
                };
                ui.label(text);
            });
            const SC_COPY: egui::KeyboardShortcut =
                egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::C);
            if resp.hovered() {
                if ui.input_mut(|mem| mem.consume_shortcut(&SC_COPY)) {
                    ui.ctx().copy_text(commit.to_string());
                }
            }
            if resp.clicked() {
                log::debug!("");
                *selected_commit = Some((repo_id, commit.to_string()));
            }
            // if let Some(pos) = resp.hover_pos() {
            //     let vals_offset =
            //         results_per_commit.and_then(|x| x.offset(commit.as_str()));
            //     let text = if let Some(v) = vals_offset {
            //         format!(
            //             "{}\n{}",
            //             commit,
            //             results_per_commit.unwrap().vals_to_string(v)
            //         )
            //     } else {
            //         format!("{}", commit)
            //     };
            //     let font_id = egui::TextStyle::Button.resolve(ui.style());
            //     ui.painter().text(
            //         pos + (20.0, 0.0).into(),
            //         egui::Align2::RIGHT_BOTTOM,
            //         text,
            //         font_id,
            //         ui.style().visuals.text_color(),
            //     );
            // }
            prev_p = center;
        }

        if *succ < usize::MAX {
            let y = min.y + time_scaling_y(cached.subs[*succ_sub].delta_time);
            let x = min.x + time_scaling(cached.times[*succ]);
            let succ_p = egui::pos2(x, y);
            let mid_p = egui::pos2(prev_p.x.min(x + 5.0), prev_p.y);
            ui.painter()
                .line_segment([prev_p, mid_p], egui::Stroke::new(2.0, parent_rel_color));
            ui.painter()
                .line_segment([mid_p, succ_p], egui::Stroke::new(2.0, parent_rel_color));
        }
    }
    for &b in &cached.branches {
        let b = cached.subs[b].prev;
        let p = (
            cached.times[b],
            min.y + time_scaling_y(cached.subs[b].delta_time),
        );
        let center = egui::pos2(time_scaling(p.0), p.1);
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        let pos = center + (70.0, -10.0).into();
        let text = &cached.commits[b];
        let text_color = ui.style().visuals.text_color();
        let galley = ui
            .painter()
            .layout(text.to_string(), font_id, text_color, 100.0);
        let angle = 0.9;
        let rect = egui::Rect::from_min_size(pos, galley.size());
        if !galley.is_empty() {
            let shape = epaint::TextShape::new(rect.min, galley, text_color);
            ui.painter().add(shape.with_angle(angle));
        };
    }
    if let Some(pos) = resp.hover_pos() {
        let top = egui::pos2(pos.x, _rect.top());
        let bot = egui::pos2(pos.x, _rect.bottom());
        ui.painter()
            .line_segment([top, bot], egui::Stroke::new(2.0, parent_rel_color));
        let Some(x_ratio) = egui::emath::inverse_lerp(rect.x_range().into(), pos.x) else {
            panic!("TODO just continue with next plot");
        };
        if min_time == i64::MAX {
            panic!("TODO just continue with next plot");
        }
        let timestamp = min_time + (x_ratio * (max_time - min_time) as f32) as i64;
        let Some(naive_datetime) = chrono::DateTime::from_timestamp(timestamp, 0) else {
            panic!("TODO just continue with next plot");
        };
        let datetime_again = naive_datetime.to_utc();
        let text = format!("{}", datetime_again);
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        ui.painter().text(
            pos + (20.0, 0.0).into(),
            egui::Align2::LEFT_TOP,
            text,
            font_id,
            ui.style().visuals.text_color(),
        );
    }
}

// Non timed
impl crate::HyperApp {
    pub(crate) fn print_commit_graph(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let id = ui.make_persistent_id("bottom_cache_layout");
        let cached = ctx.data_mut(|mem| {
            let _v = (
                self.data.selected_code_data.len(), // TODO use something better
                self.data.fetched_commit_metadata.len_local(),
                commit::CommitsLayout::default(),
            );
            let v = mem.get_temp_mut_or_insert_with(id, || _v.clone());
            if &_v == v {
                return v.2.clone();
            }

            let layout = commit::compute_commit_layout(
                |id| {
                    self.data
                        .fetched_commit_metadata
                        .get(id)
                        .and_then(|x| x.as_ref().ok())
                        .map(|x| x.clone())
                },
                self.data.selected_code_data.project_ids().filter_map(|i| {
                    let (r, mut c) = self.data.selected_code_data.get_mut(i)?;
                    let commit_slice = &mut c.iter_mut();
                    commit_slice
                        .next()
                        .map(|c| (format!("b:{}/{}", r.user, r.name), c.clone()))
                }),
            );
            v.2 = layout;

            v.2.clone()
        });
        let min = ui.available_rect_before_wrap().min.to_vec2() + egui::vec2(20.0, 20.0);
        let rect = &(cached.rect * 3.0 + egui::Margin::same(20)).translate(min);
        ui.painter()
            .debug_rect(*rect, egui::Color32::RED, "cached rect");
        let desired_size = rect.size();
        ui.allocate_exact_size(desired_size, egui::Sense::click());

        // log::debug!("{:?}", cached.subs);
        let parent_rel_color = if ui.visuals().dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };
        for commit::Subs {
            prev: child_id,
            start,
            end,
            succ,
        } in &cached.subs
        {
            let mut prev_p = cached.pos[*child_id] * 3.0 + min;
            for i in *start..*end {
                let p = cached.pos[i];
                let center = p * 3.0 + min;
                ui.painter()
                    .line_segment([prev_p, center], egui::Stroke::new(2.0, parent_rel_color));
                let fill_color = if cached.commits[i].starts_with("m:") {
                    egui::Color32::DARK_GREEN
                } else {
                    egui::Color32::GREEN
                };
                ui.painter().circle_filled(center, 6.0, fill_color);
                prev_p = center;
            }

            if *succ < usize::MAX {
                let p = cached.pos[*succ];
                let succ_p = p * 3.0 + min;
                ui.painter()
                    .line_segment([prev_p, succ_p], egui::Stroke::new(2.0, parent_rel_color));
            }
        }
        for b in cached.branches {
            let b = cached.subs[b].prev;
            let p = cached.pos[b];
            let center = p * 3.0 + min - egui::vec2(2.0, 6.0) * 3.0;
            ui.painter().debug_rect(
                egui::Rect::from_center_size(center, egui::vec2(7.0, 4.0) * 3.0),
                egui::Color32::KHAKI,
                &cached.commits[b],
            );
        }
    }
}

fn update_results_per_commit(
    results_per_commit: &mut super::ResultsPerCommit,
    r: &StreamedDataTable<
        Vec<std::string::String>,
        std::result::Result<ComputeResultIdentified, MatchingError>,
    >,
) {
    let header = &r.head; //.results.iter().find(|x| x.is_ok());
    // let Some(header) = header.as_ref() else {
    //     wasm_rs_dbg::dbg!("issue with header");
    //     panic!("issue with header");
    // };
    // let font_id = egui::TextStyle::Body.resolve(ui.style());
    // let text_color = ui.style().visuals.text_color();
    // let header = header.as_ref().unwrap();
    // let h =
    // header
    //     .inner
    //     .result
    //     .as_array()
    //     .unwrap()
    //     .into_iter()
    //     .enumerate()
    //     .map(|(i, h)| i.to_string())
    //     .collect();
    let mut vals = vec![0; header.len()];
    results_per_commit.set_cols(header);
    for r in &r.rows.lock().unwrap().1 {
        if let Ok(r) = r {
            // let c: [u8;8] = r.commit.as_bytes().as_chunks::<8>().0[0];
            // let mut c: [u8; 8] = [0; 8];
            // c.copy_from_slice(
            //     &r.commit.as_bytes()[..8],
            // );
            // panic!("{} {}",r.inner.result.is_object(),r.inner.result.is_array());
            for (i, r) in r.inner.result.as_array().unwrap().into_iter().enumerate() {
                if i > 2 {
                    break;
                }
                vals[i] = r.as_i64().unwrap() as i32;
            }

            results_per_commit.insert(
                &r.commit,
                // || {
                //     let text = crate::app::utils::join(vals.iter(), "\n").to_string();
                //     ui.painter()
                //         .layout_no_wrap(text, font_id.clone(), text_color)
                //         .into()
                // },
                r.inner.compute_time as f32,
                &[],
                &vals,
            );
        }
    }
    log::debug!("{:?}", results_per_commit);
}

mod with_egui_plot {
    use egui_plot::*;

    pub(crate) fn transform_y(y: i64) -> i64 {
        assert_ne!(y, i64::MIN);
        assert_ne!(y, i64::MAX);
        assert!(y > -1);
        if y == 0 {
            0
        } else {
            -((y as f64).sqrt() as i64) - 1
        }
    }

    pub fn center(a: [f64; 2], b: [f64; 2]) -> PlotPoint {
        fn apply(a: [f64; 2], b: [f64; 2], f: impl Fn(f64, f64) -> f64) -> [f64; 2] {
            [f(a[0], b[0]), f(a[1], b[1])]
        }
        let position = apply(a, b, |a, b| (a + b) / 2.0).into();
        position
    }

    /// from egui_plot
    /// Fill in all values between [min, max] which are a multiple of `step_size`
    fn fill_marks_between(
        step_size: f64,
        (min, max): (f64, f64),
        ori: i64,
    ) -> impl Iterator<Item = GridMark> {
        debug_assert!(min <= max, "Bad plot bounds: min: {min}, max: {max}");
        let (min, max) = (min - ori as f64, max - ori as f64);
        let first = (min / step_size).ceil() as i64;
        let last = (max / step_size).ceil() as i64;

        (first..last).map(move |i| {
            let value = (i as f64) * step_size + ori as f64;
            GridMark { value, step_size }
        })
    }

    pub(crate) fn compute_multi_x_marks(i: GridInput, ori: i64) -> Vec<GridMark> {
        // TODO use proper rounded year convention
        let year = 60 * 60 * 24 * 365 + 60 * 60 * 6;
        let years = fill_marks_between(year as f64, i.bounds, ori);
        let month = 60 * 60 * 24 * 30;
        let months = fill_marks_between(month as f64, i.bounds, ori);
        let week = 60 * 60 * 24 * 7;
        let weeks = fill_marks_between(week as f64, i.bounds, ori);
        let day = 60 * 60 * 24;
        let days = fill_marks_between(day as f64, i.bounds, ori);
        years.chain(months).chain(weeks).chain(days).collect()
    }

    pub struct CommitPoints<'a> {
        pub offsets: Vec<u32>,
        pub points: Points<'a>,
        pub with_data: bool,
    }

    impl<'a> PlotItem for CommitPoints<'a> {
        fn shapes(&self, ui: &egui::Ui, transform: &PlotTransform, shapes: &mut Vec<egui::Shape>) {
            self.points.shapes(ui, transform, shapes)
        }

        fn initialize(&mut self, x_range: std::ops::RangeInclusive<f64>) {
            self.points.initialize(x_range)
        }

        fn name(&self) -> &str {
            PlotItem::name(&self.points)
        }

        fn color(&self) -> egui::Color32 {
            PlotItem::color(&self.points)
        }

        fn highlight(&mut self) {
            PlotItem::highlight(&mut self.points)
        }

        fn highlighted(&self) -> bool {
            self.points.highlighted()
        }

        fn allow_hover(&self) -> bool {
            PlotItem::allow_hover(&self.points)
        }

        fn geometry(&self) -> PlotGeometry<'_> {
            self.points.geometry()
        }

        fn bounds(&self) -> PlotBounds {
            self.points.bounds()
        }

        fn id(&self) -> egui::Id {
            PlotItem::id(&self.points)
        }

        fn on_hover(
            &self,
            elem: ClosestElem,
            shapes: &mut Vec<egui::Shape>,
            cursors: &mut Vec<Cursor>,
            plot: &PlotConfig<'_>,
            label_formatter: &LabelFormatter<'_>,
        ) {
            let points = match self.geometry() {
                PlotGeometry::Points(points) => points,
                PlotGeometry::None => {
                    panic!("If the PlotItem has no geometry, on_hover() must not be called")
                }
                PlotGeometry::Rects => {
                    panic!("If the PlotItem is made of rects, it should implement on_hover()")
                }
            };

            // let line_color = if plot.ui.visuals().dark_mode {
            //     Color32::from_gray(100).additive()
            // } else {
            //     Color32::from_black_alpha(180)
            // };

            // this method is only called, if the value is in the result set of find_closest()
            let value = points[elem.index];
            let pointer = plot.transform.position_from_point(&value);
            // shapes.push(Shape::circle_filled(pointer, 3.0, line_color));

            let offset = self.offsets[elem.index];

            // rulers_at_value(
            //     pointer,
            //     value,
            //     self.name(),
            //     plot,
            //     shapes,
            //     cursors,
            //     label_formatter,
            // );
            let font_id = egui::TextStyle::Body.resolve(plot.ui.style());
            // WARN big hack passing an index as a f64...
            let mark = if self.with_data {
                super::CUSTOM_LABEL_FORMAT_MARK_WITH_DATA
            } else {
                super::CUSTOM_LABEL_FORMAT_MARK_NO_DATA
            };
            let text = label_formatter.as_ref().unwrap()(
                mark,
                &PlotPoint {
                    x: f64::from_bits(offset as u64),
                    y: 0.0,
                },
            );
            plot.ui.painter().text(
                pointer + egui::vec2(3.0, -2.0),
                egui::Align2::LEFT_BOTTOM,
                text,
                font_id,
                plot.ui.visuals().text_color(),
            );
            log::debug!("{}", label_formatter.is_some());
        }

        fn base(&self) -> &egui_plot::PlotItemBase {
            self.points.base()
        }

        fn base_mut(&mut self) -> &mut egui_plot::PlotItemBase {
            self.points.base_mut()
        }
    }
}
