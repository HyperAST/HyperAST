use std::ops::Range;

use egui::{NumExt, Id, CollapsingResponse};

use super::{types, code_editor::generic_text_buffer::char_index_from_byte_index2};

pub(crate) fn paint_cursor_selection(
    ui: &mut egui::Ui,
    painter: &egui::Painter,
    pos: egui::Pos2,
    galley: &egui::Galley,
    [min, max]: &[epaint::text::cursor::RCursor; 2],
    color: egui::Color32,
) {
    if min == max {
        return;
    }

    // We paint the cursor selection on top of the text, so make it transparent:
    // let [min, max] = cursor_range.sorted_cursors();
    // let min = min.rcursor;
    // let max = max.rcursor;

    for ri in min.row..=max.row {
        let row = &galley.rows[ri];
        let left = if ri == min.row {
            row.x_offset(min.column)
        } else {
            row.rect.left()
        };
        let right = if ri == max.row {
            row.x_offset(max.column)
        } else {
            let newline_size = if row.ends_with_newline {
                row.height() / 2.0 // visualize that we select the newline
            } else {
                0.0
            };
            row.rect.right() + newline_size
        };
        let rect = egui::Rect::from_min_max(
            pos + egui::vec2(left, row.min_y()),
            pos + egui::vec2(right, row.max_y()),
        );
        painter.rect_filled(rect, 0.0, color);
    }
}

pub(crate) fn paint_cursor_end(
    ui: &mut egui::Ui,
    row_height: f32,
    painter: &egui::Painter,
    pos: egui::Pos2,
    galley: &egui::Galley,
    cursor: &epaint::text::cursor::Cursor,
) -> egui::Rect {
    let stroke = ui.visuals().selection.stroke;
    let mut cursor_pos = galley.pos_from_cursor(cursor).translate(pos.to_vec2());
    cursor_pos.max.y = cursor_pos.max.y.at_least(cursor_pos.min.y + row_height); // Handle completely empty galleys
    cursor_pos = cursor_pos.expand(1.5); // slightly above/below row

    let top = cursor_pos.center_top();
    let bottom = cursor_pos.center_bottom();

    painter.line_segment(
        [top, bottom],
        (ui.visuals().text_cursor_width, stroke.color),
    );

    if false {
        // Roof/floor:
        let extrusion = 3.0;
        let width = 1.0;
        painter.line_segment(
            [
                top - egui::vec2(extrusion, 0.0),
                top + egui::vec2(extrusion, 0.0),
            ],
            (width, stroke.color),
        );
        painter.line_segment(
            [
                bottom - egui::vec2(extrusion, 0.0),
                bottom + egui::vec2(extrusion, 0.0),
            ],
            (width, stroke.color),
        );
    }

    cursor_pos
}

pub(crate) fn show_wip(ui: &mut egui::Ui, short: Option<&str>) {
    ui.vertical_centered(|ui| {
        if let Some(short) = short {
            ui.small(format!("(WIP {})", short))
        } else {
            ui.small("(WIP)")
        }
    });
}

// TODO Generalize selected config stuff
pub(crate) fn radio_collapsing<R>(
    ui: &mut egui::Ui,
    id: Id,
    title: &str,
    selected: &mut types::SelectedConfig,
    wanted: types::SelectedConfig,
    add_body: impl FnOnce(&mut egui::Ui) -> R,
) -> CollapsingResponse<R> {
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
    let title = egui::RichText::new(title).heading();
    let header_response = ui.horizontal(|ui| {
        let mut val = *selected == wanted;
        if ui.radio_value(&mut val, true, title).clicked() {
            *selected = wanted
        }
    });
    state.set_open(*selected == wanted);
    let header_response = header_response.response;
    let ret_response = state.show_body_indented(&header_response, ui, add_body);

    if header_response.clicked() {
        *selected = wanted
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


pub(crate) fn highlight_byte_range(
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

    let mut p = ui.painter().clone();
    // p.set_clip_rect(aa.inner_rect);
    paint_cursor_selection(ui, &p, te.text_draw_pos, &te.galley, &cursor_range.map(|x|x.rcursor), color);
    bounding_rect
}