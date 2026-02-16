mod app;
mod camera;
#[allow(dead_code)]
mod image_utils;
mod ml;
mod state;
mod translation;

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    js_sys::eval(
        r#"
        if ('serviceWorker' in navigator) {
            navigator.serviceWorker.register('/service-worker.js')
                .catch(function(err) { console.warn('SW registration failed:', err); });
        }
        "#,
    )
    .ok();

    dioxus::launch(app::App);
}
