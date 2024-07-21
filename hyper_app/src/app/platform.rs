#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
pub(crate) mod macos;
#[cfg(all(not(target_arch = "wasm32"), target_os = "macos"))]
pub(crate) use macos::*;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "macos")))]
pub(crate) fn init_nat_menu() {}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "macos")))]
pub(crate) fn show_nat_menu(ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // no File->Quit on web pages!
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    _frame.close();
                }
            });
        });
    });
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn show_nat_menu(ctx: &egui::Context, _frame: &mut eframe::Frame) {}

#[cfg(target_arch = "wasm32")]
pub(crate) fn init_nat_menu() {}
