#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

const ADDR: &str = "127.0.0.1:8888";

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    let api_addr = std::env::args()
        .collect::<Vec<_>>()
        .get(0)
        .and_then(|x| x.is_empty().then(|| x))
        .map_or(ADDR, |x| x)
        .to_string();
    tracing_subscriber::fmt::init();

    let languages = hyper_app::Languages::default();
    let mut native_options = eframe::NativeOptions::default();
    native_options.follow_system_theme = true;
    static ICON: &[u8] = include_bytes!("coevolution.png");
    native_options.viewport = native_options
        .viewport
        .with_maximized(true)
        .with_icon(eframe::icon_data::from_png_bytes(ICON).unwrap());
    eframe::run_native(
        "HyperAST",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(hyper_app::HyperApp::new(cc, languages, api_addr)))
        }),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    use egui_addon::Lang;
    use wasm_bindgen::prelude::*;
    let api_addr = std::env::args()
        .collect::<Vec<_>>()
        .get(0)
        .and_then(|x| x.is_empty().then(|| x.to_string()));
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        tree_sitter::TreeSitter::init()
            .await
            .map_err(JsValue::from)
            .unwrap();
        let mut languages: hyper_app::Languages = Default::default();
        let start_result = eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(move |cc| {
                    Ok(Box::new(hyper_app::HyperApp::new(
                        cc, languages, api_addr, ADDR,
                    )))
                }),
            )
            .await;
        let loading_text = eframe::web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("loading_text"));
        match start_result {
            Ok(_) => {
                loading_text.map(|e| e.remove());
            }
            Err(e) => {
                loading_text.map(|e| {
                    e.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    )
                });
                panic!("failed to start eframe: {e:?}");
            }
        }
    });
}
