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
        if let Some(msg) = &self.message {
            ui.text_edit_multiline(&mut msg.to_string());
        }
        ui.label(format!("Parents: {}", self.parents.join(" + ")));
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