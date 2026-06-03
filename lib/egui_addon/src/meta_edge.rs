use epaint::Pos2;

pub fn meta_egde(
    m_pos: Pos2,
    src_pos: Pos2,
    m_rect: epaint::Rect,
    ctrl: (Pos2, Pos2),
    src_rect: epaint::Rect,
    color: epaint::Color32,
    ui: &mut egui::Ui,
) {
    let tolerance = (m_pos.x - src_pos.x).abs() * 0.001;
    let offset = epaint::Vec2::Y * 8.0;
    let link = epaint::CubicBezierShape::from_points_stroke(
        [
            m_rect.right_top() + offset,
            ctrl.0,
            ctrl.1,
            src_rect.left_top() + offset,
        ],
        false,
        egui::Color32::TRANSPARENT,
        (5.0, color),
    );
    let up = link.flatten(Some(tolerance));
    let link = epaint::CubicBezierShape::from_points_stroke(
        [
            m_rect.right_bottom() - offset,
            ctrl.0,
            ctrl.1,
            src_rect.left_bottom() - offset,
        ],
        false,
        egui::Color32::TRANSPARENT,
        (5.0, color),
    );
    let down = link.flatten(Some(tolerance));
    let l = up.len() + down.len();
    let color_step = 255.0 / l as f32;
    let mut out = epaint::Mesh::default();
    let mut up = up.into_iter();
    let mut down = down.into_iter();
    let mut p_up = up.next().unwrap();
    let mut p_down = down.next().unwrap();
    let mut idx = 0;
    let mut color = egui::Color32::GREEN;
    out.colored_vertex(p_down, color);
    let f = |idx| {
        lerp_color_gamma(
            egui::Color32::GREEN,
            egui::Color32::RED,
            idx as f32 / l as f32,
        )
    };
    color = f(idx);
    out.colored_vertex(p_up, color);
    color = f(idx);
    loop {
        if let Some(x) = down.next() {
            p_down = x;
        } else {
            let mut i = idx;
            while let Some(x) = up.next() {
                out.colored_vertex(x, color);
                color = f(idx);
                i += 1;
                out.add_triangle(idx, i, i + 1);
            }
            break;
        };
        out.colored_vertex(p_down, color);
        color = f(idx);
        out.add_triangle(idx, idx + 1, idx + 2);
        idx += 1;
        if let Some(x) = up.next() {
            p_up = x;
        } else {
            let mut i = idx;
            while let Some(x) = down.next() {
                out.colored_vertex(x, color);
                color = f(idx);
                i += 1;
                out.add_triangle(idx, i + 1, i);
            }
            break;
        };
        out.colored_vertex(p_up, color);
        color = f(idx);
        out.add_triangle(idx + 1, idx, idx + 2);
        idx += 1;
    }
    ui.painter().add(out);
}

fn lerp_color_gamma(left: epaint::Color32, right: epaint::Color32, t: f32) -> epaint::Color32 {
    epaint::Color32::from_rgba_premultiplied(
        epaint::emath::lerp((left[0] as f32)..=(right[0] as f32), t).round() as u8,
        epaint::emath::lerp((left[1] as f32)..=(right[1] as f32), t).round() as u8,
        epaint::emath::lerp((left[2] as f32)..=(right[2] as f32), t).round() as u8,
        epaint::emath::lerp((left[3] as f32)..=(right[3] as f32), t).round() as u8,
    )
}
