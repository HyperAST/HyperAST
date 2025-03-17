#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

const ADDR: &str = "127.0.0.1:8888";

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    use std::ops::Not;
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();
    let args = std::env::args()
        .collect::<Vec<_>>();
    log::trace!("{:?}",args);
    let api_addr = args
        .get(1)
        .and_then(|x| x.is_empty().not().then(|| x))
        .map_or(ADDR, |x| x)
        .to_string();

    let languages = hyper_app::Languages::default();
    static ICON: &[u8] = include_bytes!("coevolution.png");
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("hyperast")
            // .with_maximized(true)
            .with_icon(eframe::icon_data::from_png_bytes(ICON).unwrap())
            .with_decorations(!re_ui::CUSTOM_WINDOW_DECORATIONS) // Maybe hide the OS-specific "chrome" around the window
            .with_fullsize_content_view(re_ui::FULLSIZE_CONTENT)
            .with_inner_size([1200.0, 800.0])
            .with_title_shown(!re_ui::FULLSIZE_CONTENT)
            .with_titlebar_buttons_shown(!re_ui::CUSTOM_WINDOW_DECORATIONS)
            .with_titlebar_shown(!re_ui::FULLSIZE_CONTENT)
            .with_transparent(re_ui::CUSTOM_WINDOW_DECORATIONS), // To have rounded corners without decorations we need transparency


        ..Default::default()
    };
    eframe::run_native(
        "HyperAST",
        native_options,
        Box::new(move |cc| {
            cc.egui_ctx.options_mut(|opt| {
                opt.theme_preference = egui::ThemePreference::Dark
            });
            re_ui::apply_style_and_install_loaders(&cc.egui_ctx);
            Ok(Box::new(hyper_app::HyperApp::new(cc, languages, api_addr)))
        }),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    use egui_addon::Lang;
    use wasm_bindgen::prelude::*;
    let api_addr = None;
    // let api_addr = std::env::args()
    //     .collect::<Vec<_>>()
    //     .get(0)
    //     .and_then(|x| x.is_empty().then(|| x.to_string()));
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        #[cfg(feature = "ts_highlight")]
        tree_sitter::TreeSitter::init()
            .await
            .map_err(JsValue::from)
            .unwrap();
        let mut languages: hyper_app::Languages = Default::default();
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");
        
        let start_result = eframe::WebRunner::new()
            .start(
                canvas, // hardcode it
                web_options,
                Box::new(move |cc| {
                    re_ui::apply_style_and_install_loaders(&cc.egui_ctx);
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
