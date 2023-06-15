use std::ops::Range;
use crate::code_editor::generic_text_buffer::char_index_from_byte_index2;

mod paint_cursor;

pub fn highlight_byte_range(
    ui: &mut egui::Ui,
    te: &egui::text_edit::TextEditOutput,
    selected_node: &Range<usize>,
    color: egui::Color32,
) -> egui::Rect {
    let cursor_range = {
        let (a, b) =
            char_index_from_byte_index2(te.galley.text(), selected_node.start, selected_node.end);
        [
            te.galley
                .from_ccursor(epaint::text::cursor::CCursor::new(a)),
            te.galley
                .from_ccursor(epaint::text::cursor::CCursor::new(b)),
        ]
    };
    let mut bounding_rect = te.galley.rows[cursor_range[0].rcursor.row].rect;
    bounding_rect.extend_with_y(te.galley.rows[cursor_range[1].rcursor.row].max_y());

    let p = ui.painter().clone();
    // p.set_clip_rect(aa.inner_rect);
    paint_cursor_selection(
        ui,
        &p,
        te.text_draw_pos,
        &te.galley,
        &cursor_range.map(|x| x.rcursor),
        color,
    );
    bounding_rect
}

use egui::{CollapsingResponse, Id};

use self::paint_cursor::paint_cursor_selection;

pub fn show_wip(ui: &mut egui::Ui, short: Option<&str>) {
    ui.vertical_centered(|ui| {
        if let Some(short) = short {
            ui.small(format!("(WIP {})", short))
        } else {
            ui.small("(WIP)")
        }
    });
}

pub fn radio_collapsing<R, S: PartialEq + Clone>(
    ui: &mut egui::Ui,
    id: Id,
    title: &str,
    selected: &mut S,
    wanted: &S,
    add_body: impl FnOnce(&mut egui::Ui) -> R,
) -> CollapsingResponse<R> {
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
    let title = egui::RichText::new(title).heading();
    let header_response = ui.horizontal(|ui| {
        let mut val = selected == wanted;
        if ui.radio_value(&mut val, true, title).clicked() {
            *selected = wanted.clone()
        }
    });
    state.set_open(selected == wanted);
    let header_response = header_response.response;
    let ret_response = state.show_body_indented(&header_response, ui, add_body);

    if header_response.clicked() {
        *selected = wanted.clone()
    };
    let openness = state.openness(ui.ctx());
    if let Some(ret_response) = ret_response {
        CollapsingResponse {
            header_response,
            body_response: Some(ret_response.response),
            body_returned: Some(ret_response.inner),
            openness,
        }
    } else {
        CollapsingResponse {
            header_response,
            body_response: None,
            body_returned: None,
            openness,
        }
    }
}
