#[cfg(target_arch = "wasm32")]
pub(crate) fn file_save(content: &str) {
    use wasm_bindgen::prelude::wasm_bindgen;
    #[wasm_bindgen]
    extern "C" {
        fn alert(s: &str);
    }
    eframe::web_sys::console::log_1(&content.into());
    alert("(WIP) Look a the debug console to copy the file :)");
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn file_save(_content: &str) {
    // TODO
    println!("TODO save file")
}
