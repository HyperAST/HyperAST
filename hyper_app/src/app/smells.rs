//! Smell detection / Code quality issue finder / Code Quality evaluation assistant
//!
//! This HyperAST extension is aimed at assisting developers that want to improve the code quality of one of their project.
//! 1) The developer select/provide examples comming from a patch that improves code quality
//! 2) The developer select a commit in his project.
//! 3) Then a latice of queries is computed corrresponding to the examples:
//!     1) these queries are run on the provided commit
//!     2) the results are used to filter out extreme queries and rank them
//!     3) the developper select the query he prefer
//! 4) the selected query can then be executed on multiple commits or up to their resolution
//! 5) the results are diplayed to the developer and can be explored
//! 6) given feedbacks from the developper alternative queries are suggested
//! 7) go back to 4 by selecting a query
//! 8) generate an issue summarising the possible quality/consitency improvements
//!
//!
//! This tool can also be used by researchers on a more meta level,
//! such as finding how popular/widespread/prevalent/pervasive is a particular code pattern on productivity
//! or evaluating the impact of a particular code pattern on productivity
//!
//! Note: Smell are particularly interesting for their complexiity and context dependant nature,
//! as well as their low frequency (per pattern (not per category)).
//!
//! Note: The temporal analysis (~ searching for the patterns throughout the history of dev)
//! provide confidence and facts about the significance of code quality issues
//!
//!
//! https://github.com/INRIA/spoon/commit/8f967893e5441dbf95b842350234c3185bcaeed7
//! test: Migrate support and testing tests to Junit5
//! @Test(expected = ...)
//!
//! https://github.com/google/gson/commit/99cc4cb11f73a6d672aa6381013d651b7921e00f
//! more specifically:
//! https://github.com/Marcono1234/gson/commit/3d241ca0a6435cbf1fa1cdaed2af8480b99fecde
//! about fixing try catches in tests

use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Range, SubAssign},
};

use egui_addon::{
    interactive_split::interactive_splitter::InteractiveSplitter,
    multi_split::{
        multi_splitter::MultiSplitter, multi_splitter_orientation::MultiSplitterOrientation,
    },
};
use wasm_rs_dbg::dbg;

use crate::app::utils_edition::MakeHighlights;

use super::{
    code_tracking::RemoteFile,
    show_repo_menu,
    types::{self, CodeRange, Commit, SelectedConfig},
};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub(super) struct ComputeConfigQuery {
    commit: Commit,
    /// the query configuring the query generation from examples
    /// eg. `(identifier) @label ["{" ";" "." "try" "(" ")" "}" "catch" "import"] @skip (block ["{" "}"] @show) (block) @imm`
    /// eg. `(identifier) (type_identifier)` same as `(identifier) @label (type_identifier) @label`
    meta_gen: String,
    /// the query configuring the query simplification/generalization
    /// eg. `(predicate (identifier) (#EQ? "EQ") (parameters (string) @label )) @pred`
    meta_simp: String,
    config: super::types::Config,
    len: usize,
    simple_matching: bool,
    prepro_matching: bool,

    // filterings
    wanted_matches: std::ops::Range<usize>,

    // just ui stuff, might do better
    advanced_open: bool,
}

impl Default for ComputeConfigQuery {
    fn default() -> Self {
        use super::types::*;
        Self {
            //google/gson/commit/99cc4cb11f73a6d672aa6381013d651b7921e00f
            //Marcono1234/gson/commit/3d241ca0a6435cbf1fa1cdaed2af8480b99fecde
            commit: Commit {
                repo: Repo {
                    user: "Marcono1234".into(),
                    name: "gson".into(),
                },
                id: "3d241ca0a6435cbf1fa1cdaed2af8480b99fecde".into(),
            },
            meta_gen: r#"(identifier) @label
["{" ";" "." "try" "(" ")" "}" "catch" "import"] @skip"#
                .into(),
            meta_simp: r#"(predicate
    (identifier) (#EQ? "EQ")
    (parameters
        (string) @label
    )
) @pred
(_
    (named_node
        (identifier) (#EQ "expression_statement")
    ) @rm
    .
)
(_
    (named_node
        (identifier) (#EQ "expression_statement")
    ) @rm
    .
    (named_node)
)
(_
    (named_node
        (identifier) (#EQ "expression_statement")
    ) @rm
    .
    (anonymous_node)
)"#
            .into(),
            config: Config::MavenJava,
            len: 1,
            simple_matching: true,
            prepro_matching: true,
            wanted_matches: Default::default(),
            advanced_open: false,
        }
    }
}

// pub(crate) type Config = Sharing<ComputeConfigQuery>;
#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct Config {
    pub(crate) commits: Option<ComputeConfigQuery>,
    pub(crate) diffs: Option<ExamplesValues>,
    pub(crate) queries: Option<SearchResults>,
    pub(crate) stats: Option<Vec<(types::CodeRange, types::CodeRange)>>,
    pub(crate) bad_matches_bounds: std::ops::RangeInclusive<usize>,
    pub(crate) bads: Option<Vec<usize>>,
}
impl Default for Config {
    fn default() -> Self {
        let compute_config_query: ComputeConfigQuery = Default::default();
        Self {
            commits: Some(compute_config_query),
            diffs: None,
            queries: None,
            stats: None,
            bad_matches_bounds: std::ops::RangeInclusive::new(0, 0),
            bads: Default::default(),
        }
    }
}

pub(crate) type RemoteResult =
    super::utils_results_batched::Remote<Result<SearchResults, SmellsError>>;
pub(crate) type RemoteResultGenQuery =
    super::utils_results_batched::Remote<Result<QueryGenResults, SmellsError>>;
pub(crate) type RemoteResultDiffs =
    super::utils_results_batched::Remote<Result<ExamplesValues, DiffsError>>;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct QueryGenResults {
    pub prepare_time: f64,
    pub results: Vec<Result<String, String>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct SearchResults {
    pub prepare_time: f64,
    pub search_time: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Default::default")]
    pub bad: Vec<SearchResult>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Default::default")]
    pub good: Vec<SearchResult>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ExamplesValues {
    pub(crate) examples: Vec<ExamplesValue>,
    moves: Vec<(CodeRange, CodeRange)>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ExamplesValue {
    pub(crate) before: CodeRange,
    pub(crate) after: CodeRange,
    pub(crate) inserts: Vec<Range<usize>>,
    pub(crate) deletes: Vec<Range<usize>>,
    pub(crate) moves: Vec<(Range<usize>, Range<usize>)>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct SearchResult {
    pub query: String,
    // the correspondin
    pub examples: Vec<usize>,
    //stats
    pub matches: usize,
}

pub struct SearchResults2 {
    pub prepare_time: f64,
    pub search_time: f64,
    pub examples: Vec<(CodeRange, CodeRange)>,
    pub bad: Vec<SearchResult>,
    pub good: Vec<SearchResult>,
}

// WIP
pub(crate) type Context = ();

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum SmellsError {
    MissingLanguage(String),
    QueryParsing(String),
    MissingExamples(String),
}
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum DiffsError {
    Error(String),
}

pub(crate) const WANTED: SelectedConfig = SelectedConfig::Smells;

pub(crate) fn show_config(ui: &mut egui::Ui, smells: &mut Config) {
    use super::utils_egui::MyUiExt;
    match &mut smells.commits {
        Some(conf) => {
            ui.label("Source of inital Examples:");
            show_repo_menu(ui, &mut conf.commit.repo);
            ui.push_id(ui.id().with("commit"), |ui| {
                egui::TextEdit::singleline(&mut conf.commit.id)
                    .clip_text(true)
                    .desired_width(150.0)
                    .desired_rows(1)
                    .hint_text("commit")
                    .interactive(true)
                    .show(ui)
            });

            ui.add_enabled_ui(true, |ui| {
                ui.add(
                    egui::Slider::new(&mut conf.len, 1..=200)
                        .text("commits")
                        .clamping(egui::SliderClamping::Always)
                        .integer()
                        .logarithmic(true),
                );
                // show_wip(ui, Some("only process one commit"));
            });
            let selected = &mut conf.config;
            selected.show_combo_box(ui, "Repo Config");

            if ui
                .add(egui::Button::new("🗖 Open Advanced Settings").selected(conf.advanced_open))
                .clicked()
            {
                conf.advanced_open ^= true;
            }

            egui::Window::new("Interactive Finder's Advanced Settings")
                .open(&mut conf.advanced_open)
                .show(ui.ctx(), |ui| {
                    ui.label("Query Generation:");
                    egui::TextEdit::multiline(&mut conf.meta_gen)
                        // .clip_text(true)
                        // .desired_width(150.0)
                        .desired_rows(1)
                        .hint_text("the query configuring the query generation")
                        .interactive(true)
                        .show(ui);

                    ui.label("Query Simplification:");
                    egui::TextEdit::multiline(&mut conf.meta_simp)
                        // .clip_text(true)
                        // .desired_width(150.0)
                        .desired_rows(1)
                        .hint_text(
                            "the query used to direct the simplification of generated queries",
                        )
                        .interactive(true)
                        .show(ui);

                    ui.checkbox(&mut conf.simple_matching, "Simple Matching");
                    ui.checkbox(&mut conf.prepro_matching, "Incr. Matching");
                });

            ui.checkbox(&mut conf.simple_matching, "Simple Matching");
            ui.checkbox(&mut conf.prepro_matching, "Incr. Matching");

            ui.label("#matches on entire commit:");
            if ui
                .double_ended_slider(
                    &mut conf.wanted_matches.start,
                    &mut conf.wanted_matches.end,
                    smells.bad_matches_bounds.clone(),
                )
                .on_hover_text("displays only queries in the given range")
                .changed()
            {
                smells.bads = None
            };
        }
        None => (),
    }
}

pub(super) fn show_result(
    ui: &mut egui::Ui,
    api_addr: &str,
    smells: &mut ComputeConfigQuery,
    examples: &ExamplesValues,
    promise: &mut Option<RemoteResult>,
    _cols: std::ops::Range<usize>,
    _id: egui::Id,
) {
    let Some(promise) = promise else {
        ui.spinner();
        ui.spinner();
        *promise = Some(fetch_results(ui.ctx(), api_addr, smells, examples));
        return;
    };
    let Some(result) = promise.ready() else {
        ui.spinner();
        return;
    };
    match result {
        Ok(resource) => match &resource.content {
            Some(Ok(content)) => {
                // show_long_result_success(ui, content);
                let len = content.bad.len();
                if len == 0 {
                    ui.label("no queries found");
                    return;
                }
                // loop {
                //     let i = cols.start;
                // ui.push_id(id.with(-(i as isize + 1)), |ui| {
                // });
                // }
                // MultiSplitter::with_orientation(MultiSplitterOrientation::Horizontal)
                //     .ratios(vec![0.1 * 2.0; len - 1])
                //     .show(ui, |uis| {
                //         for (i, ui) in uis.iter_mut().enumerate() {
                //             let c = &content.bad[i];
                //             ui.label(format!(
                //                 "query[{}] prep={:3} search={:3} matches={}",
                //                 i, content.prepare_time, content.search_time, c.matches
                //             ));
                //             ui.text_edit_multiline(&mut c.query.clone());
                //         }
                //     });
                todo!()
            }
            Some(Err(error)) => {
                dbg!(&error);
                ui.label("Error");
            }
            _ => (),
        },
        Err(error) => {
            wasm_rs_dbg::dbg!();
            // This should only happen if the fetch API isn't available or something similar.
            ui.colored_label(
                ui.visuals().error_fg_color,
                if error.is_empty() { "Error" } else { error },
            );
        }
    }
}

pub(super) fn show_central_panel(
    ui: &mut egui::Ui,
    api_addr: &str,
    smells: &mut Config,
    _smells_editors: &mut Context,
    _trigger_compute: &mut bool,
    smells_result: &mut Option<RemoteResult>,
    smells_diffs_result: &mut Option<RemoteResultDiffs>,
    fetched_files: &mut HashMap<types::FileIdentifier, RemoteFile>,
) {
    if let Some(_x) = &mut smells.stats {
        todo!();
    }
    if let Some(_examples) = &mut smells.queries {
        todo!();
    }
    if let Some(promise) = smells_result {
        let Some(result) = promise.ready() else {
            let center = ui.available_rect_before_wrap().center();
            let conf = smells.commits.as_mut().unwrap();
            let examples = smells.diffs.as_mut().unwrap();
            show_examples(ui, api_addr, examples, fetched_files);
            egui::Window::new("Actions")
                .default_pos(center)
                .pivot(egui::Align2::CENTER_CENTER)
                .show(ui.ctx(), |ui| {
                    if ui.button("Compute Queries").clicked() {
                        *smells_result = Some(fetch_results(ui.ctx(), api_addr, conf, &examples));
                    }
                    ui.spinner();
                });
            smells.bad_matches_bounds = std::ops::RangeInclusive::new(0, 0);
            return;
        };
        match result {
            Ok(resource) => match &resource.content {
                Some(Ok(queries)) => {
                    let conf = smells.commits.as_mut().unwrap();
                    let examples = smells.diffs.as_mut().unwrap();
                    let center = ui.available_rect_before_wrap().center();
                    let id = ui.id();
                    let tot_len = queries.bad.len();
                    if smells.bads.is_none() {
                        smells.bads = Some(
                            (0..tot_len)
                                .filter(|i| conf.wanted_matches.contains(&queries.bad[*i].matches))
                                .collect(),
                        )
                    }
                    let bads = smells.bads.as_ref().unwrap();
                    let len = bads.len();
                    if !queries.good.is_empty() {
                        todo!("handle the queries matching the fixes")
                    }

                    if *smells.bad_matches_bounds.end() == 0 {
                        let start = queries
                            .bad
                            .iter()
                            .map(|x| x.matches)
                            .min()
                            .unwrap_or_default();
                        let end = queries
                            .bad
                            .iter()
                            .map(|x| x.matches)
                            .max()
                            .unwrap_or_default();
                        smells.bad_matches_bounds = std::ops::RangeInclusive::new(start, end);
                    }
                    if tot_len == 0 {
                        ui.colored_label(ui.visuals().error_fg_color, "No queries found");
                        return;
                    }
                    if len == 0 {
                        ui.colored_label(
                            ui.visuals().error_fg_color,
                            "No queries selected, change the hyperparameters",
                        );
                        return;
                    }
                    egui::ScrollArea::vertical()
                        .scroll_bar_visibility(
                            egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                        )
                        .show_rows(ui, H, len, |ui, rows| {
                            let (mut rect, _) = ui.allocate_exact_size(
                                egui::Vec2::new(
                                    ui.available_width(),
                                    H, // * (rows.end - rows.start) as f32,
                                ),
                                egui::Sense::hover(),
                            );
                            let top = rect.top();
                            for i in rows.start..rows.end - 1 {
                                let mut rect = {
                                    let (t, b) = rect.split_top_bottom_at_y(
                                        top + H * (i - rows.start + 1) as f32,
                                    );
                                    rect = b;
                                    t
                                };
                                rect.bottom_mut().sub_assign(B);

                                let line_pos_1 = egui::emath::GuiRounding::round_to_pixels(
                                    rect.left_bottom(),
                                    ui.pixels_per_point(),
                                );
                                let line_pos_2 = egui::emath::GuiRounding::round_to_pixels(
                                    rect.right_bottom(),
                                    ui.pixels_per_point(),
                                );
                                ui.painter().line_segment(
                                    [line_pos_1, line_pos_2],
                                    ui.visuals().window_stroke(),
                                );
                                rect.bottom_mut().sub_assign(B);
                                let mut ui = ui.new_child(
                                    egui::UiBuilder::new()
                                        .max_rect(rect)
                                        .layout(egui::Layout::top_down(egui::Align::Min)),
                                );
                                ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                                ui.push_id(
                                    id.with(bads[i])
                                        .with("query_with_example")
                                        .with(&resource.response.bytes),
                                    |ui| {
                                        let bad_query = &queries.bad[bads[i]];
                                        // let example = &mut examples.examples[bad_query.examples];
                                        show_query_with_example(
                                            ui,
                                            api_addr,
                                            bad_query,
                                            examples,
                                            fetched_files,
                                        );
                                    },
                                );
                            }
                        });
                    egui::Window::new("Actions")
                        .default_pos(center)
                        .pivot(egui::Align2::CENTER_CENTER)
                        .show(ui.ctx(), |ui| {
                            if ui.button("Compute Queries").clicked() {
                                *smells_result =
                                    Some(fetch_results(ui.ctx(), api_addr, conf, &examples));
                            }
                        });
                    return;
                    // smells.queries = Some(examples.clone());
                }
                Some(Err(error)) => {
                    egui::Window::new("QueryError").show(ui.ctx(), |ui| {
                        ui.label("Error");
                        ui.label(format!("{:?}", error));
                    });
                }
                _ => (),
            },
            Err(error) => {
                // This should only happen if the fetch API isn't available or something similar.
                egui::Window::new("QueryError").show(ui.ctx(), |ui| {
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        if error.is_empty() { "Error" } else { error },
                    );
                });
            }
        };
    }
    if let Some(promise) = smells_diffs_result {
        let Some(result) = promise.ready() else {
            ui.spinner();
            return;
        };
        match result {
            Ok(resource) => match &resource.content {
                Some(Ok(examples)) => {
                    smells.diffs = Some(examples.clone());
                }
                Some(Err(error)) => {
                    egui::Window::new("QueryError").show(ui.ctx(), |ui| {
                        ui.label("Error");
                        ui.label(format!("{:?}", error));
                    });
                }
                _ => (),
            },
            Err(error) => {
                // This should only happen if the fetch API isn't available or something similar.
                egui::Window::new("QueryError").show(ui.ctx(), |ui| {
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        if error.is_empty() { "Error" } else { error },
                    );
                });
            }
        };
    }
    if let Some(examples) = &mut smells.diffs {
        let len = examples.examples.len();
        if len == 0 {
            ui.colored_label(ui.visuals().error_fg_color, "No changes found");
            return;
        }
        let conf = smells.commits.as_mut().unwrap();
        let center = ui.available_rect_before_wrap().center();
        show_examples(ui, api_addr, examples, fetched_files);
        egui::Window::new("Actions")
            .default_pos(center)
            .pivot(egui::Align2::CENTER_CENTER)
            .show(ui.ctx(), |ui| {
                if ui.button("Compute Queries").clicked() {
                    *smells_result = Some(fetch_results(ui.ctx(), api_addr, conf, &examples));
                }
            });
        return;
    }
    if let Some(conf) = &mut smells.commits {
        ui.label(format!("{:?}", conf));
        ui.spinner();
        *smells_diffs_result = Some(fetch_examples_at_commits(ui.ctx(), api_addr, conf));
        // TODO display when multiple possible choices
        // egui::Window::new("Actions").show(ui.ctx(), |ui| {
        //     if ui.button("Show diffs").clicked() {
        //         *trigger_compute = true;
        //     }
        // });
        return;
    }
}

pub(crate) fn show_examples(
    ui: &mut egui::Ui,
    api_addr: &str,
    examples: &mut ExamplesValues,
    fetched_files: &mut HashMap<types::FileIdentifier, RemoteFile>,
) {
    let id = ui.id();
    let len = examples.examples.len();
    assert!(len > 0);
    egui::ScrollArea::vertical()
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .show_rows(ui, H, len, |ui, cols| {
            let (mut rect, _) = ui.allocate_exact_size(
                egui::Vec2::new(ui.available_width(), H * (cols.end - cols.start) as f32),
                egui::Sense::hover(),
            );
            let top = rect.top();
            for i in cols.clone() {
                let mut rect = {
                    let (t, b) = rect.split_top_bottom_at_y(top + H * (i - cols.start + 1) as f32);
                    rect = b;
                    t
                };
                rect.bottom_mut().sub_assign(B);

                let line_pos_1 = egui::emath::GuiRounding::round_to_pixels(
                    rect.left_bottom(),
                    ui.pixels_per_point(),
                );
                let line_pos_2 = egui::emath::GuiRounding::round_to_pixels(
                    rect.right_bottom(),
                    ui.pixels_per_point(),
                );
                ui.painter()
                    .line_segment([line_pos_1, line_pos_2], ui.visuals().window_stroke());
                rect.bottom_mut().sub_assign(B);
                let mut ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                ui.push_id(id.with(i), |ui| {
                    let example = &examples.examples[i];
                    show_diff(ui, api_addr, example, fetched_files);
                });
            }
        });
}

fn show_query_with_example(
    ui: &mut egui::Ui,
    api_addr: &str,
    bad_query: &SearchResult,
    examples: &mut ExamplesValues,
    fetched_files: &mut HashMap<types::FileIdentifier, RemoteFile>,
) {
    InteractiveSplitter::vertical()
        .ratio(0.3)
        .show(ui, |ui1, ui2| {
            ui1.push_id(
                ui1.id().with("query_bad_smell").with(&bad_query.query),
                |ui| {
                    show_query(bad_query, ui);
                },
            );
            let clip_rect = ui2.clip_rect();
            let bad_ex_cont = &bad_query.examples[..bad_query.examples.len().min(12)];
            if bad_ex_cont.is_empty() {
                ui2.label(format!("{}", bad_query.examples.len()));
                return;
            }
            MultiSplitter::with_orientation(MultiSplitterOrientation::Horizontal)
                .ratios(if bad_ex_cont.len() <= 8 {
                    vec![1.0 / bad_ex_cont.len() as f32; bad_ex_cont.len() - 1]
                } else {
                    [0.2, 0.2]
                        .into_iter()
                        .chain(
                            (0..bad_ex_cont.len() - 3)
                                .into_iter()
                                .map(|_| 0.6 / (bad_ex_cont.len() - 2) as f32),
                        )
                        .collect()
                })
                .show(ui2, |uis| {
                    for (i, ui) in uis.iter_mut().enumerate() {
                        if !clip_rect.contains_rect(ui.clip_rect()) {
                            continue;
                        }
                        let example = &examples.examples[bad_ex_cont[i]];
                        ui.push_id(ui1.id().with(i), |ui| {
                            show_diff(ui, api_addr, example, fetched_files)
                        });
                        // ui.label(format!(
                        //     "query[{}] prep={:3} search={:3} matches={}",
                        //     i, content.prepare_time, content.search_time, c.matches
                        // ));
                        // ui.text_edit_multiline(&mut c.query.clone());
                    }
                });
        });
}

pub(crate) fn show_query(
    bad_query: &SearchResult,
    ui: &mut egui::Ui,
) -> egui::scroll_area::ScrollAreaOutput<
    egui_addon::code_editor::generic_text_edit::output::TextEditOutput,
> {
    let mut code: &str = &bad_query.query;
    let language = "clojure";
    // use super::syntax_highlighting::syntax_highlighting_async as syntax_highlighter;
    // let theme = super::syntax_highlighting::syntect::CodeTheme::from_memory(ui.ctx());
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

    let mut layouter = |ui: &egui::Ui, code: &str, wrap_width: f32| {
        use egui_extras::syntax_highlighting::highlight;
        let mut layout_job = highlight(ui.ctx(), ui.style(), &theme, code, language);
        // syntax_highlighter::highlight(ui.ctx(), &theme, code, language);
        if false {
            layout_job.wrap.max_width = wrap_width;
        }
        ui.fonts(|f| f.layout_job(layout_job))
    };
    // dbg!(&code);
    let scroll_resp = egui::scroll_area::ScrollArea::both().show(ui, |ui| {
        egui_addon::code_editor::generic_text_edit::TextEdit::multiline(&mut code)
            .layouter(&mut layouter)
            .desired_width(f32::MAX)
            .show(ui)
    });
    let mut font_id = egui::TextStyle::Heading.resolve(ui.style());
    font_id.size *= 3.0;
    ui.painter().text(
        ui.available_rect_before_wrap().right_top(),
        egui::Align2::RIGHT_BOTTOM,
        bad_query.matches,
        font_id,
        matches_color(ui),
    );
    scroll_resp
}

fn matches_color(ui: &egui::Ui) -> egui::Color32 {
    if ui.visuals().dark_mode {
        egui::Color32::YELLOW
    } else {
        egui::Color32::from_rgb(255, 127, 0)
    }
}

const B: f32 = 15.;
const H: f32 = 800.;

pub(crate) fn show_diff(
    ui: &mut egui::Ui,
    api_addr: &str,
    example: &ExamplesValue,
    fetched_files: &mut HashMap<types::FileIdentifier, RemoteFile>,
) {
    let rect = ui.clip_rect();
    let mov_col = move_color(ui);
    InteractiveSplitter::vertical().show(ui, |ui1, ui2| {
        ui1.set_clip_rect(ui1.max_rect().intersect(rect));
        ui2.set_clip_rect(ui2.max_rect().intersect(rect));
        ui2.push_id(ui2.id().with("second"), |ui| {
            let color = insert_color(ui);
            let ma = MH::<false> {
                main: &example.inserts,
                col: color,
                moves: &example.moves,
                mov_col,
                hash: hash(&example.before.file),
            };
            show_either_side(ui, fetched_files, api_addr, &example.after, color, ma);
            ui.separator();
        });
        ui1.push_id(ui1.id().with("first"), |ui| {
            let color = delete_color(ui);
            let ma = MH::<true> {
                main: &example.deletes,
                col: color,
                moves: &example.moves,
                mov_col,
                hash: hash(&example.before.file),
            };
            show_either_side(ui, fetched_files, api_addr, &example.before, color, ma);
            ui.separator();
        });
    });
}

pub fn hash<T: ?Sized + Hash>(x: &T) -> u64 {
    let mut state = std::hash::DefaultHasher::default();
    x.hash(&mut state);
    use std::hash::Hasher;
    state.finish()
}

fn delete_color(ui: &mut egui::Ui) -> egui::Rgba {
    if ui.visuals().dark_mode {
        egui::Color32::from_rgb(255, 50, 50).gamma_multiply(0.01)
    } else {
        egui::Color32::from_rgb(240, 20, 20).gamma_multiply(0.03)
    }
    .into()
}

fn insert_color(ui: &mut egui::Ui) -> egui::Rgba {
    if ui.visuals().dark_mode {
        egui::Color32::from_rgb(40, 235, 40).gamma_multiply(0.01)
    } else {
        egui::Color32::from_rgb(20, 235, 20).gamma_multiply(0.03)
    }
    .into()
}

fn move_color(ui: &mut egui::Ui) -> egui::Rgba {
    if ui.visuals().dark_mode {
        egui::Color32::from_rgb(50, 50, 255).gamma_multiply(0.4)
    } else {
        egui::Color32::BLUE.gamma_multiply(0.1)
    }
    .into()
}

#[derive(Clone, Copy)]
struct MH<'a, const LEFT: bool> {
    main: &'a Vec<Range<usize>>,
    col: egui::Rgba,
    moves: &'a Vec<(Range<usize>, Range<usize>)>,
    mov_col: egui::Rgba,
    hash: u64,
}

impl<'a, const LEFT: bool> std::hash::Hash for MH<'a, LEFT> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        LEFT.hash(state);
        self.hash.hash(state);
    }
}

impl<'a, const LEFT: bool> MakeHighlights for MH<'a, LEFT> {
    const COLORS: u8 = 2;
    fn highlights(&self, col: u8) -> (egui::Rgba, impl Iterator<Item = (usize, usize)>) {
        let main: &[Range<usize>];
        let moves: &[(Range<usize>, Range<usize>)];
        let color;
        if col == 0 {
            main = &self.main;
            moves = &[];
            color = self.col;
        } else if col == 1 {
            moves = &self.moves;
            main = &[];
            color = self.mov_col;
        } else {
            unreachable!()
        };
        let main = main.iter().map(|x| (x.start, x.end));
        let moves = moves
            .iter()
            .map(|x| if LEFT { &x.0 } else { &x.1 })
            .map(|x| (x.start, x.end));
        (color, main.chain(moves))
    }
}

fn show_either_side<MH: MakeHighlights>(
    ui: &mut egui::Ui,
    fetched_files: &mut HashMap<types::FileIdentifier, RemoteFile>,
    api_addr: &str,
    code: &CodeRange,
    color: egui::Rgba,
    highlights: MH,
) {
    let file_result = fetched_files.entry(code.file.clone());
    let id_scroll = ui.id().with("off_scrolled");
    let r = super::utils_poll::try_fetch_remote_file(&file_result, |file| {
        let mut content: &str = &file.content;
        let language = "java";
        use egui::text::LayoutJob;
        use egui_addon::syntax_highlighting::syntect::CodeTheme;
        let theme = CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &egui::Ui, content: &str, _wrap_width: f32| {
            type HighlightCache =
                egui::util::cache::FrameCache<LayoutJob, crate::app::utils_edition::Highlighter0>;
            let layout_job = ui.ctx().memory_mut(|mem| {
                mem.caches.cache::<HighlightCache>().get((
                    &theme,
                    crate::app::utils_edition::FileContainer(&code.file, content),
                    language,
                ))
            });

            ui.fonts(|f| {
                let galley = f.layout_job(layout_job);
                let mut galley = galley.as_ref().clone();
                galley
                    .rows
                    .iter_mut()
                    .for_each(|row| row.glyphs.iter_mut().for_each(|g| g.line_height = 100.0));
                galley.into()
            })
        };
        let noop = ui.painter().add(egui::Shape::Noop);
        let scroll = egui::scroll_area::ScrollArea::both().show(ui, |ui| {
            egui_addon::code_editor::generic_text_edit::TextEdit::multiline(&mut content)
                .layouter(&mut layouter)
                .desired_width(f32::MAX)
                .show(ui)
        });
        (scroll, noop)
    });
    if r.is_none() {
        if let std::collections::hash_map::Entry::Vacant(_) = file_result {
            file_result.insert_entry(super::code_tracking::remote_fetch_file(
                ui.ctx(),
                &api_addr,
                &code.file.commit,
                &mut code.file.file_path.clone(),
            ));
        }
    }
    let te = match r {
        Some(Ok(r)) => Some(r),
        None => None,
        Some(Err(error)) => {
            ui.colored_label(
                ui.visuals().error_fg_color,
                if error.is_empty() { "Error" } else { &error },
            );
            None
        }
    };

    if let Some((mut aa, noop)) = te {
        if false {
            // NOTE too slow, need to hop on the galley generation or cache the rectangles
            // for (color, start, end) in highlights {
            //     let ui = &mut ui.child_ui(aa.inner_rect, *ui.layout());
            //     {
            //         let rect = aa.inner.galley.rows[0].rect;
            //         let rect = rect.translate(aa.inner.text_draw_pos.to_vec2());
            //         let stroke = egui::Stroke::new(2., egui::Color32::KHAKI);
            //         ui.painter()
            //             .rect(rect, 1., egui::Color32::KHAKI.linear_multiply(0.1), stroke);
            //     }
            //     ui.set_clip_rect(aa.inner_rect);
            //     egui_addon::egui_utils::highlight_byte_range_aux(
            //         ui,
            //         &aa.inner.galley,
            //         aa.inner.text_draw_pos,
            //         &Range { start, end },
            //         color,
            //     );
            // }
        } else {
            let galley = &aa.inner.galley;
            #[derive(Clone, Copy)]
            struct G<'a>(&'a std::sync::Arc<egui::Galley>);
            impl<'a> std::hash::Hash for G<'a> {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    self.0.hash(state);
                }
            }
            impl<'a> AsRef<std::sync::Arc<egui::Galley>> for G<'a> {
                fn as_ref(&self) -> &std::sync::Arc<egui::Galley> {
                    self.0
                }
            }

            type Type = egui::util::cache::FrameCache<
                Vec<(egui::Color32, Vec<egui::Rect>)>,
                crate::app::utils_edition::HiHighlighter2,
            >;
            let shape = ui
                .ctx()
                .memory_mut(|mem| mem.caches.cache::<Type>().get((G(galley), highlights)));
            let offset = aa.inner.text_draw_pos.to_vec2();
            let clip_rect = ui.clip_rect().translate(-offset);
            let shapes = shape
                .into_iter()
                .flat_map(|(color, rects)| {
                    rects.into_iter().filter_map(move |rect| {
                        rect.intersects(clip_rect).then(|| {
                            let mut shape = egui::Shape::rect_filled(rect, 1.0, color);
                            shape.translate(offset);
                            shape
                        })
                    })
                })
                .collect();
            ui.painter().set(noop, egui::Shape::Vec(shapes));
        }
        if let Some(selected_node) = &code.range {
            let ui = &mut ui.new_child(egui::UiBuilder::new().max_rect(aa.inner_rect));
            ui.set_clip_rect(aa.inner_rect.intersect(ui.clip_rect()));
            let mut rect = egui_addon::egui_utils::highlight_byte_range_aux(
                ui,
                &aa.inner.galley,
                aa.inner.text_draw_pos,
                selected_node,
                color.multiply(0.01).into(),
            );

            let first_there = ui.ctx().data_mut(|d| {
                let r = d.get_temp_mut_or_default::<bool>(id_scroll);
                let tmp = *r;
                *r = true;
                tmp
            });
            if !first_there {
                aa.state.offset.y =
                    rect.top() - (aa.inner_rect.height() - rect.height()).abs() / 2.0;
                aa.state.store(ui.ctx(), aa.id);
            }
            rect = rect.translate(aa.inner.text_draw_pos.to_vec2());

            let stroke = {
                let mut color = color;
                if ui.visuals().dark_mode {
                    color[0] = color[0] + 0.2;
                    color[1] = color[1] + 0.2;
                    color[2] = color[2] + 0.2;
                    color[3] = color[3] * 2.0;
                } else {
                    color = (egui::Rgba::from(color) * 10.0).into();
                };
                egui::Stroke::new(3., color)
            };
            ui.painter().rect(
                rect,
                1.,
                color.multiply(0.1),
                stroke,
                egui::StrokeKind::Inside,
            );
        }
    };
}

pub(super) fn fetch_results(
    ctx: &egui::Context,
    api_addr: &str,
    smells: &mut ComputeConfigQuery,
    examples: &ExamplesValues,
) -> RemoteResult {
    let ctx = ctx.clone();
    let (sender, promise) = poll_promise::Promise::new();
    let url = format!(
        "http://{}/smells/github/{}/{}/{}/{}",
        api_addr,
        &smells.commit.repo.user,
        &smells.commit.repo.name,
        &smells.commit.id,
        &smells.len,
    );
    #[derive(serde::Serialize)]
    struct QueryContent {
        language: String,
        query: String,
        commits: usize,
    }
    let _language = match smells.config {
        types::Config::Any => "",
        types::Config::MavenJava => "Java",
        types::Config::MakeCpp => "Cpp",
    }
    .to_string();

    #[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
    pub struct ExamplesValues<S, T, U> {
        meta_gen: S,
        meta_simp: S,
        examples: T,
        moves: U,
        simple_matching: bool,
        prepro_matching: bool,
    }
    let examples = ExamplesValues {
        meta_gen: &smells.meta_gen,
        meta_simp: &smells.meta_simp,
        examples: examples
            .examples
            .iter()
            .map(|x| ExamplesValue {
                before: x.before.clone(),
                after: x.after.clone(),
                inserts: vec![],
                deletes: vec![],
                moves: vec![],
            })
            .collect::<Vec<_>>(),
        moves: (),
        simple_matching: smells.simple_matching,
        prepro_matching: smells.prepro_matching,
    };
    let mut request = ehttp::Request::post(&url, serde_json::to_vec(&examples).unwrap());
    request.headers.insert(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // will wake up UI thread
        let resource = response.and_then(|response| {
            types::Resource::<Result<SearchResults, SmellsError>>::from_response(&ctx, response)
        });
        sender.send(resource);
    });
    promise
}

impl types::Resource<Result<SearchResults, SmellsError>> {
    pub(super) fn from_response(
        _ctx: &egui::Context,
        response: ehttp::Response,
    ) -> Result<Self, String> {
        wasm_rs_dbg::dbg!(&response);
        let content_type = response.content_type().unwrap_or_default();

        if response.status == 404 {
            let Some(text) = response.text() else {
                wasm_rs_dbg::dbg!();
                return Err("".to_string());
            };
            return Err(text.to_string());
        }
        if !content_type.starts_with("application/json") {
            let Some(text) = response.text() else {
                return Err(format!("Wrong content type: {}", content_type));
            };
            return Err(format!(
                "Wrong content type: {}\n{}",
                content_type,
                &text[..100.min(text.len())]
            ));
        }
        if response.status != 200 {
            let Some(text) = response.text() else {
                wasm_rs_dbg::dbg!();
                return Err("".to_string());
            };
            let Ok(json) = serde_json::from_str::<SmellsError>(text) else {
                wasm_rs_dbg::dbg!();
                return Err("".to_string());
            };
            return Ok(Self {
                response,
                content: Some(Err(json)),
            });
        }

        let text = response.text();
        dbg!(&text);
        let text = text.and_then(|text| {
            serde_json::from_str(text)
                .inspect_err(|err| {
                    wasm_rs_dbg::dbg!(&err);
                })
                .ok()
        });
        dbg!(&text);

        Ok(Self {
            response,
            content: text.map(|x| Ok(x)),
        })
    }
}

pub(super) fn fetch_examples_at_commits(
    ctx: &egui::Context,
    api_addr: &str,
    smells: &mut ComputeConfigQuery,
) -> RemoteResultDiffs {
    let ctx = ctx.clone();
    let (sender, promise) = poll_promise::Promise::new();
    let url = format!(
        "http://{}/smells_ex_from_diffs/github/{}/{}/{}/{}",
        api_addr,
        &smells.commit.repo.user,
        &smells.commit.repo.name,
        &smells.commit.id,
        &smells.len,
    );
    #[derive(serde::Serialize)]
    struct QueryContent {
        language: String,
        query: String,
        commits: usize,
    }
    let _language = match smells.config {
        types::Config::Any => "",
        types::Config::MavenJava => "Java",
        types::Config::MakeCpp => "Cpp",
    }
    .to_string();

    let mut request = ehttp::Request::post(&url, serde_json::to_vec(&[""]).unwrap());
    request.headers.insert(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // will wake up UI thread
        let resource = response.and_then(|response| {
            types::Resource::<Result<ExamplesValues, DiffsError>>::from_response(&ctx, response)
        });
        sender.send(resource);
    });
    promise
}

impl types::Resource<Result<ExamplesValues, DiffsError>> {
    pub(super) fn from_response(
        _ctx: &egui::Context,
        response: ehttp::Response,
    ) -> Result<Self, String> {
        wasm_rs_dbg::dbg!(&response);
        let content_type = response.content_type().unwrap_or_default();

        if response.status == 404 {
            let Some(text) = response.text() else {
                wasm_rs_dbg::dbg!();
                return Err("".to_string());
            };
            return Err(text.to_string());
        }
        if !content_type.starts_with("application/json") {
            return Err(format!("Wrong content type: {}", content_type));
        }
        if response.status != 200 {
            let Some(text) = response.text() else {
                wasm_rs_dbg::dbg!();
                return Err("".to_string());
            };
            let Ok(json) = serde_json::from_str::<DiffsError>(text) else {
                wasm_rs_dbg::dbg!();
                return Err("".to_string());
            };
            return Ok(Self {
                response,
                content: Some(Err(json)),
            });
        }

        let text = response.text();
        let text = text.and_then(|text| {
            serde_json::from_str(text)
                .inspect_err(|err| {
                    wasm_rs_dbg::dbg!(&err);
                })
                .ok()
        });

        Ok(Self {
            response,
            content: text.map(|x| Ok(x)),
        })
    }
}
