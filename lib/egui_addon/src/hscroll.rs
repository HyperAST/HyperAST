use egui::*;

pub fn hscroll_many_columns<R>(
    ui: &mut Ui,
    width: f32,
    total_cols: usize,
    mut add_contents: impl FnMut(&mut Ui, usize) -> R,
) {
    let spacing = ui.spacing().item_spacing;
    // the width of each column with space
    let with_spacing = width + spacing.x;
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show_viewport(ui, |ui, viewport| {
            // use egui::NumExt;
            // ui.set_height((with_spacing * total_cols as f32 - spacing.x).at_least(0.0));

            let mut min_col = (viewport.min.x / with_spacing).floor() as usize;
            let mut max_col = (viewport.max.x / with_spacing).ceil() as usize + 1;
            if max_col > total_cols {
                let diff = max_col.saturating_sub(min_col);
                max_col = total_cols;
                min_col = total_cols.saturating_sub(diff);
            }

            let x_min = ui.max_rect().left() + min_col as f32 * with_spacing;
            let x_max = ui.max_rect().left() + max_col as f32 * with_spacing;

            let rect = egui::Rect::from_x_y_ranges(x_min..=x_max, ui.max_rect().y_range());

            let cols = min_col..max_col;
            ui.allocate_new_ui(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::left_to_right(egui::Align::TOP)),
                |ui| {
                    ui.skip_ahead_auto_ids(min_col); // Make sure we get consistent IDs.
                    for i in cols {
                        ui.allocate_ui(Vec2::new(with_spacing, ui.clip_rect().height()), |ui| {
                            ui.set_max_width(width);
                            egui::ScrollArea::vertical()
                                .id_salt(i)
                                .auto_shrink([false, false])
                                .show(ui, |ui| ui.vertical(|ui| add_contents(ui, i)));
                        });
                    }
                },
            )
            .inner
        });
}

#[allow(unused)]
const ELE: &str = r#"adfwregwr
adfwregwradfwregwradfwregwr
adfwregwr
adfwregwr

adfwregwradfwregwradfwregwradfwregwr
adfwregwradfwregwr
adfwregwr
adfwregwradfwregwradfwregwr
adfwregwradfwregwr
adfwregwr
adfwregwr
adfwregwr
adfwregwr
adfwregwradfwregwr



adfwregwradfwregwr
adfwregwr
adfwregwradfwregwr
"#;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "eframe")]
#[allow(unused)]
fn main() {
    let cols = vec![ELE; 20];
    let total_cols = cols.len();
    eframe::run_simple_native(
        "horizontal scroll with culling",
        eframe::NativeOptions::default(),
        move |ctx, _frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                hscroll_many_columns(ui, 300.0, total_cols, |ui: &mut Ui, i| {
                    ui.label(format!("{}", cols[i]))
                });
            });
        },
    )
    .unwrap();
}
