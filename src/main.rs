mod app;
mod camera;
#[allow(dead_code)]
mod image_utils;
mod ml;
mod state;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    dioxus::launch(app::App);
}
