use egui::NumExt;

pub(crate) fn paint_cursor_selection(
    _ui: &mut egui::Ui,
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

/// without visualizing newlines
pub(crate) fn paint_cursor_selection2(
    _ui: &mut egui::Ui,
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
            row.rect.right()
        };
        let rect = egui::Rect::from_min_max(
            pos + egui::vec2(left, row.min_y()),
            pos + egui::vec2(right, row.max_y()),
        );
        painter.rect_filled(rect, 0.0, color);
    }
}

pub fn paint_cursor_end(
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
        (ui.visuals().text_cursor.stroke.width, stroke.color),
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
