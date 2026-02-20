use dioxus::prelude::*;

use crate::camera;
use crate::ml;
use crate::state::{AppState, Screen};

const VIDEO_ID: &str = "camera-preview";

#[component]
pub fn App() -> Element {
    let state = use_signal(AppState::default);

    rsx! {
        document::Stylesheet { href: asset!("/assets/main.css") }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "anonymous",
        }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=SN+Pro:ital,wght@0,200..900;1,200..900&display=swap",
        }
        div { class: "app",
            match state.read().screen {
                Screen::Camera => rsx! { CameraScreen { state } },
                Screen::Processing => rsx! { ProcessingScreen { state } },
                Screen::Result => rsx! { ResultScreen { state } },
            }
        }
    }
}

#[component]
fn CameraScreen(state: Signal<AppState>) -> Element {
    use_future(move || async move {
        if let Err(e) = camera::start_camera(VIDEO_ID).await {
            log::error!("Camera error: {}", e);
            state.write().error = Some(e);
        }
    });

    rsx! {
        div { class: "camera-screen",
            h1 { class: "app-title", "Netthinne" }
            div { class: "camera-viewport",
                video {
                    id: VIDEO_ID,
                    autoplay: true,
                    playsinline: true,
                    class: "camera-preview",
                }
                button {
                    class: "capture-btn",
                    onclick: move |_| {
                        match camera::capture_frame(VIDEO_ID) {
                            Ok((pixels, w, h)) => {
                                let _ = camera::stop_camera(VIDEO_ID);
                                let mut s = state.write();
                                s.captured_pixels = Some((pixels, w, h));
                                s.screen = Screen::Processing;
                            }
                            Err(e) => {
                                log::error!("Capture error: {}", e);
                                state.write().error = Some(e);
                            }
                        }
                    },
                }
            }
            if let Some(ref err) = state.read().error {
                p { class: "error-text", "{err}" }
            }
        }
    }
}

/// Mounts while inference runs. use_future fires once on mount, runs inference,
/// then navigates to ResultScreen. ProcessingScreen stays alive for the full duration
/// so the task is never cancelled by scope drops.
#[component]
fn ProcessingScreen(state: Signal<AppState>) -> Element {
    use_future(move || async move {
        let data = state.read().captured_pixels.clone();
        match data {
            Some((pixels, w, h)) => {
                let (english, norwegian) = ml::process_image(&pixels, w, h);
                let mut s = state.write();
                s.detected_label = Some(english);
                s.translated_label = Some(norwegian);
                s.captured_pixels = None;
                s.screen = Screen::Result;
            }
            None => {
                log::error!("ProcessingScreen mounted with no captured pixels");
                state.write().screen = Screen::Camera;
            }
        }
    });

    rsx! {
        div { class: "processing-screen",
            p { class: "processing-text", "Analyzing..." }
        }
    }
}

#[component]
fn ResultScreen(state: Signal<AppState>) -> Element {
    let detected = state.read().detected_label.clone().unwrap_or_default();
    let translated = state.read().translated_label.clone().unwrap_or_default();

    rsx! {
        div { class: "result-screen",
            p { class: "label-english", "{detected}" }
            p { class: "label-norwegian", "{translated}" }
            button {
                class: "new-scan-btn",
                onclick: move |_| {
                    *state.write() = AppState::default();
                },
                "New Scan"
            }
        }
    }
}
