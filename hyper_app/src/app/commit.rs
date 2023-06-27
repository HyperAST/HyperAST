use poll_promise::Promise;

use crate::app::{types::Resource, API_URL};

use super::types::Commit;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CommitMetadata {
    /// commit message
    message: Option<String>,
    /// parents commits
    /// if multiple parents, the first one should be where the merge happends
    parents: Vec<String>,
    /// tree corresponding to version
    tree: Option<String>,
    /// offset in minutes
    timezone: i32,
    /// seconds
    time: i64,
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
            let text = self
                .parents
                .iter()
                .map(|x| &x[..8])
                .intersperse(" + ")
                .collect::<String>();
            let label = ui.label(format!("Parents: {}", text));
            if label.hovered() {
                let text = self.parents.join(" + ");
                egui::show_tooltip(ui.ctx(), label.id.with("tooltip"), |ui| {
                    ui.label(&text);
                    ui.label("CTRL+C to copy (and send in the debug console)");
                });
                const SC_COPY: egui::KeyboardShortcut =
                    egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::C);
                wasm_rs_dbg::dbg!(&text);
                if ui.input_mut(|mem| mem.consume_shortcut(&SC_COPY)) {
                    wasm_rs_dbg::dbg!(&text);
                    ui.output_mut(|mem| mem.copied_text = text.to_string());
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
                    egui::show_tooltip(ui.ctx(), label.id.with("tooltip"), |ui| {
                        ui.text_edit_multiline(&mut msg.to_string());
                    });
                }
            }
        }
    }
}

pub(super) fn fetch_commit(
    ctx: &egui::Context,
    commit: &Commit,
) -> Promise<Result<CommitMetadata, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "{}/commit/github/{}/{}/{}",
        API_URL, &commit.repo.user, &commit.repo.name, &commit.id,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        wasm_rs_dbg::dbg!(&response);
        ctx.request_repaint(); // wake up UI thread
        let resource = response
            .and_then(|response| Resource::<CommitMetadata>::from_response(&ctx, response))
            .and_then(|x| x.content.ok_or("No content".into()));
        sender.send(resource);
    });
    promise
}

impl Resource<CommitMetadata> {
    fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Result<Self, String> {
        wasm_rs_dbg::dbg!(&response);
        // let content_type = response.content_type().unwrap_or_default();

        let text = response.text();
        let text = text.ok_or("")?;
        let text = serde_json::from_str(text).map_err(|x| x.to_string())?;
        wasm_rs_dbg::dbg!(&text);

        Ok(Self {
            response,
            content: text,
        })
    }
}

pub(super) fn fetch_commit_parents(
    ctx: &egui::Context,
    commit: &Commit,
    depth: usize,
) -> Promise<Result<Vec<String>, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "{}/commit-parents/github/{}/{}/{}/{}",
        API_URL, &commit.repo.user, &commit.repo.name, &commit.id, depth
    );

    wasm_rs_dbg::dbg!(&url);
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
    fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Result<Self, String> {
        wasm_rs_dbg::dbg!(&response);
        // let content_type = response.content_type().unwrap_or_default();

        let text = response.text();
        let text = text.ok_or("")?;
        let text = serde_json::from_str(text).map_err(|x| x.to_string())?;
        wasm_rs_dbg::dbg!(&text);

        Ok(Self {
            response,
            content: text,
        })
    }
}
