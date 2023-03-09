use poll_promise::Promise;

use crate::app::API_URL;

use super::egui_utils::radio_collapsing;

use super::egui_utils::show_wip;

use super::show_repo;

use super::types;
use super::types::Resource;

pub(crate) fn show_aspects_views_menu(
    ui: &mut egui::Ui,
    selected: &mut types::SelectedConfig,
    aspects: &mut types::ComputeConfigAspectViews,
    aspects_result: &mut Option<Promise<Result<Resource<FetchedView>, String>>>,
) {
    let title = "Aspects Views";
    let wanted = (&*aspects).into();
    let id = ui.make_persistent_id(title);
    let add_body = |ui: &mut egui::Ui| {
        show_repo(ui, &mut aspects.commit.repo);
        ui.push_id(ui.id().with("commit"), |ui| {
            egui::TextEdit::singleline(&mut aspects.commit.id)
                .clip_text(true)
                .desired_width(150.0)
                .desired_rows(1)
                .hint_text("commit")
                .interactive(true)
                .show(ui)
        });
        ui.push_id(ui.id().with("path"), |ui| {
            if egui::TextEdit::singleline(&mut aspects.path)
                .clip_text(true)
                .desired_width(150.0)
                .desired_rows(1)
                .hint_text("path")
                .interactive(true)
                .show(ui)
                .response
                .changed()
            {
                *aspects_result = Some(remote_fetch_tree(ui.ctx(), &aspects.commit, &aspects.path));
            }
        });
        ui.checkbox(&mut aspects.cst, "CST");
        ui.add_enabled_ui(false, |ui| {
            ui.checkbox(&mut aspects.ast, "AST");
            show_wip(ui, Some(" soon available"));
            ui.checkbox(&mut aspects.type_decls, "Type Decls");
            show_wip(ui, Some(" soon available"));
            ui.checkbox(&mut aspects.licence, "Licence");
            show_wip(ui, Some(" soon available"));
            ui.checkbox(&mut aspects.doc, "Doc");
            show_wip(ui, Some(" soon available"));
        });
        // ui.text_edit_singleline(&mut "github.com/INRIA/spoon");
    };

    radio_collapsing(ui, id, title, selected, wanted, add_body);
}

type NodeId = u64;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FetchedView {
    label_list: Vec<String>,
    type_sys: TypeSys,
    root: NodeId,
    labeled: ViewLabeled,
    children: ViewChildren,
    both: ViewBoth,
    typed: ViewTyped,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ViewLabeled {
    ids: Vec<NodeId>,
    kinds: Vec<u16>,
    labels: Vec<u32>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ViewTyped {
    ids: Vec<NodeId>,
    kinds: Vec<u16>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ViewChildren {
    ids: Vec<NodeId>,
    kinds: Vec<u16>,
    cs_ofs: Vec<u32>,
    cs_lens: Vec<u32>,
    children: Vec<NodeId>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ViewBoth {
    ids: Vec<NodeId>,
    kinds: Vec<u16>,
    labels: Vec<u32>,
    cs_ofs: Vec<u32>,
    cs_lens: Vec<u32>,
    children: Vec<NodeId>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub(crate) struct TypeSys(Vec<String>);

pub(crate) enum Action {
    Keep,
    Delete,
}

impl FetchedView {
    pub(crate) fn show(&self, ui: &mut egui::Ui) -> Action {
        if let Some(c) = self.both.ids.iter().position(|x| x == &self.root) {
            self.ui_both_impl(ui, 0, c)
        } else if let Some(c) = self.labeled.ids.iter().position(|x| x == &self.root) {
            self.ui_labeled_impl(ui, 0, c)
        } else if let Some(c) = self.children.ids.iter().position(|x| x == &self.root) {
            self.ui_children_impl(ui, 0, c)
        } else if let Some(c) = self.typed.ids.iter().position(|x| x == &self.root) {
            self.ui_typed_impl(ui, 0, c)
        } else {
            panic!();
        }
    }
}

impl FetchedView {
    pub(crate) fn ui_both_impl(&self, ui: &mut egui::Ui, depth: usize, nid: usize) -> Action {
        let kind = &self.type_sys.0[self.both.kinds[nid] as usize];
        let label = self.both.labels[nid];
        let label = &self.label_list[label as usize];
        let o = self.both.cs_ofs[nid] as usize;
        let cs = &self.both.children[o..o + self.both.cs_lens[nid] as usize];
        // egui::CollapsingHeader::new(format!("{}: {}\t{}", kind, label, nid))
        egui::CollapsingHeader::new(format!("{}: {}", kind, label))
            .id_source(ui.next_auto_id())
            .default_open(depth < 1)
            .show(ui, |ui| self.children_ui(ui, depth, cs))
            .body_returned
            .unwrap_or(Action::Keep)
    }
    pub(crate) fn ui_children_impl(&self, ui: &mut egui::Ui, depth: usize, nid: usize) -> Action {
        let kind = &self.type_sys.0[self.children.kinds[nid] as usize];
        let o = self.children.cs_ofs[nid] as usize;
        let cs = &self.children.children[o..o + self.children.cs_lens[nid] as usize];
        // egui::CollapsingHeader::new(format!("{}\t{}", kind, nid))
        egui::CollapsingHeader::new(format!("{}", kind))
            .id_source(ui.next_auto_id())
            .default_open(depth < 1)
            .show(ui, |ui| self.children_ui(ui, depth, cs))
            .body_returned
            .unwrap_or(Action::Keep)
    }
    pub(crate) fn ui_labeled_impl(&self, ui: &mut egui::Ui, _depth: usize, nid: usize) -> Action {
        let kind = &self.type_sys.0[self.labeled.kinds[nid] as usize];
        let label = self.labeled.labels[nid];
        let label = &self.label_list[label as usize];
        let label = label
            .replace("\n", "\\n")
            .replace("\t", "\\t")
            .replace(" ", "·");
        if kind == "spaces" {
            ui.label(format!("{}: {}", kind, label));
        } else {
            ui.label(format!("{}: {}", kind, label));
        }
        Action::Keep
    }
    pub(crate) fn ui_typed_impl(&self, ui: &mut egui::Ui, _depth: usize, nid: usize) -> Action {
        let kind = &self.type_sys.0[self.typed.kinds[nid] as usize];
        // ui.label(format!("k {}\t{}", kind, nid));
        ui.label(format!("{}", kind));
        Action::Keep
    }

    pub(crate) fn children_ui(&self, ui: &mut egui::Ui, depth: usize, cs: &[u64]) -> Action {
        if depth > 20 {
            for c in cs {
                ui.label(c.to_string());
            }
            return Action::Keep;
        }
        for c in cs {
            if let Some(c) = self.both.ids.iter().position(|x| x == c) {
                self.ui_both_impl(ui, depth + 1, c);
            } else if let Some(c) = self.labeled.ids.iter().position(|x| x == c) {
                self.ui_labeled_impl(ui, depth + 1, c);
            } else if let Some(c) = self.children.ids.iter().position(|x| x == c) {
                self.ui_children_impl(ui, depth + 1, c);
            } else if let Some(c) = self.typed.ids.iter().position(|x| x == c) {
                self.ui_typed_impl(ui, depth + 1, c);
            } else {
                ui.label(format!("f {c}"));
            }
        }

        // if depth > 0
        //     && ui
        //         .button(egui::RichText::new("delete").color(ui.visuals().warn_fg_color))
        //         .clicked()
        // {
        //     return Action::Delete;
        // }

        // self.0 = std::mem::take(self)
        //     .0
        //     .into_iter()
        //     .enumerate()
        //     .filter_map(|(i, mut tree)| {
        //         if tree.ui_impl(ui, depth + 1, &format!("child #{}", i)) == Action::Keep {
        //             Some(tree)
        //         } else {
        //             None
        //         }
        //     })
        //     .collect();

        // if ui.button("+").clicked() {
        //     self.0.push(FetchedView::default());
        // }

        Action::Keep
    }
}

impl Resource<FetchedView> {
    fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Self {
        wasm_rs_dbg::dbg!(&response);
        let content_type = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|x| serde_json::from_str(x).unwrap());

        Self {
            response,
            content: text,
        }
    }
}

pub(super) type RemoteView = Promise<ehttp::Result<Resource<FetchedView>>>;

pub(super) fn remote_fetch_tree(
    ctx: &egui::Context,
    commit: &types::Commit,
    path: &str,
) -> Promise<Result<Resource<FetchedView>, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "{}/view/github/{}/{}/{}/{}",
        API_URL, &commit.repo.user, &commit.repo.name, &commit.id, &path,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource =
            response.map(|response| Resource::<FetchedView>::from_response(&ctx, response));
        sender.send(resource);
    });
    promise
}
