use crate::code_editor::generic_text_buffer::char_index_from_byte_index2;
use std::ops::Range;

mod paint_cursor;

pub fn highlight_byte_range(
    ui: &mut egui::Ui,
    te: &egui::text_edit::TextEditOutput,
    selected_node: &Range<usize>,
    color: egui::Color32,
) -> egui::Rect {
    highlight_byte_range_aux(ui, &te.galley, te.galley_pos, selected_node, color)
}

pub fn highlight_byte_range_aux(
    ui: &mut egui::Ui,
    galley: &egui::Galley,
    galley_pos: egui::Pos2,
    selected_node: &Range<usize>,
    color: egui::Color32,
) -> egui::Rect {
    let cursor_range = compute_cursor_range(galley, selected_node);
    let bounding_rect = compute_bounding_rect(galley, cursor_range);

    let p = ui.painter().clone();
    // p.set_clip_rect(aa.inner_rect);
    paint_cursor_selection2(
        ui,
        &p,
        galley_pos,
        &galley,
        &cursor_range.map(|x| x.rcursor),
        color,
    );
    bounding_rect
}

pub fn compute_bounding_rect(
    galley: &egui::Galley,
    cursor_range: [epaint::text::cursor::Cursor; 2],
) -> egui::Rect {
    let row_range = cursor_range.map(|c| c.rcursor.row);
    compute_bounding_rect_from_row_range(galley, row_range)
}

pub fn compute2_bounding_rect(
    galley: &egui::Galley,
    cursor_range: [epaint::text::cursor::Cursor; 2],
) -> egui::Rect {
    let range = cursor_range.map(|c| c.rcursor);
    let mut bounding_rect = galley.rows[range[0].row].rect;
    for x in &galley.rows[range[0].row + 1..=range[1].row] {
        // let rect =
        bounding_rect = bounding_rect.union(x.rect);
    }
    bounding_rect.min.x = galley.rows[range[0].row].x_offset(range[0].column);
    // bounding_rect.max.x = galley.rows[range[1].row].x_offset(range[1].column);
    bounding_rect
}

pub fn compute_bounding_rect_from_row_range(
    galley: &egui::Galley,
    row_range: [usize; 2],
) -> egui::Rect {
    let mut bounding_rect = galley.rows[row_range[0]].rect;
    for x in &galley.rows[row_range[0] + 1..=row_range[1]] {
        bounding_rect = bounding_rect.union(x.rect);
    }
    bounding_rect
}

pub fn compute2_bounding_rect_from_row_range(
    galley: &egui::Galley,
    row_range: [usize; 2],
) -> egui::Rect {
    let mut bounding_rect = galley.rows[row_range[0]].rect;
    if let Some(x) = first_ws_x(&galley.rows[row_range[0]]) {
        bounding_rect.min.x = x;
    } else {
        bounding_rect.min.x = f32::MAX;
    }
    for x in &galley.rows[row_range[0] + 1..=row_range[1]] {
        let mut other = x.rect;
        if let Some(x) = first_ws_x(x) {
            other.min.x = x;
            if bounding_rect.min.x > bounding_rect.max.x {
                bounding_rect.min.x = x
            }
        } else {
            other.min.x = bounding_rect.max.x;
        }
        bounding_rect = bounding_rect.union(other);
    }
    bounding_rect
}

pub fn first_ws_x(row: &epaint::text::Row) -> Option<f32> {
    row.glyphs
        .iter()
        .find(|x| !x.chr.is_ascii_whitespace())
        .map(|g| {
            dbg!(g);
            g.pos.x})
}

pub fn compute_cursor_range(
    galley: &egui::Galley,
    selected_node: &Range<usize>,
) -> [epaint::text::cursor::Cursor; 2] {
    let (a, b) = char_index_from_byte_index2(galley.text(), selected_node.start, selected_node.end);
    [
        galley.from_ccursor(epaint::text::cursor::CCursor::new(a)),
        galley.from_ccursor(epaint::text::cursor::CCursor::new(b)),
    ]
}

use egui::{CollapsingResponse, Id};
use wasm_rs_dbg::dbg;

use self::paint_cursor::paint_cursor_selection2;

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
