use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub fn run(canvas: HtmlCanvasElement) {
    particle_dance::run_web(canvas)
}
