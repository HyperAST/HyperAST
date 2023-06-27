#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).

    use egui_addon::Lang;
    tracing_subscriber::fmt::init();

    let lang = tree_sitter_javascript::language().into();
    let name = "JavaScript".to_string();
    let mut languages: hyper_app::Languages = Default::default();
    languages.insert(name.clone(), Lang { name, lang });
    // let mut parser = tree_sitter::Parser::new().unwrap();
    // parser.set_language(&lang.into()).expect("Error loading Java grammar");
    // let parsed = parser.parse("function f() {}", None).unwrap().unwrap();
    // parsed.walk().node().kind();
    dbg!();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "HyperAST",
        native_options,
        Box::new(move |cc| Box::new(hyper_app::HyperApp::new(cc, languages))),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    use egui_addon::Lang;
    use wasm_bindgen::prelude::*;
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        // use eframe::web_sys::console;
        tree_sitter::TreeSitter::init()
            .await
            .map_err(JsValue::from)
            .unwrap();
        // let mut parser = tree_sitter::Parser::new().unwrap();
        let lang = web_tree_sitter_sg::Language::load_path("./tree-sitter-javascript.wasm")
            .await
            .unwrap()
            .into();
        let name = "JavaScript".to_string();
        let mut languages: hyper_app::Languages = Default::default();
        languages.insert(name.clone(), Lang { name, lang });
        // panic!("lang");
        // parser.set_language(&lang.into()).expect("Error loading Java grammar");
        // let parsed = parser.parse("function f() {}", None).unwrap().unwrap();
        // console::log_1(&"42".into());
        // console::log_1(&parsed.walk().node().kind().as_ref().into());
        // dbg!("{:?}", parsed);

        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(move |cc| Box::new(hyper_app::HyperApp::new(cc, languages))),
        )
        .await
        .expect("failed to start eframe");
    });
}
