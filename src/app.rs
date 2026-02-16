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
        document::Link { rel: "manifest", href: "/manifest.json" }
        document::Meta { name: "theme-color", content: "#1a4a58" }
        document::Meta { name: "apple-mobile-web-app-capable", content: "yes" }
        document::Meta {
            name: "apple-mobile-web-app-status-bar-style",
            content: "black-translucent",
        }


        div { class: "app",
            match state.read().screen {
                Screen::Camera => rsx! { CameraScreen { state } },
                Screen::Processing => rsx! { ProcessingScreen {} },
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
            video {
                id: VIDEO_ID,
                autoplay: true,
                playsinline: true,
                class: "camera-preview",
            }
            if let Some(ref err) = state.read().error {
                p { class: "error-text", "{err}" }
            }
            button {
                class: "capture-btn",
                onclick: move |_| {
                    let mut state = state;
                    spawn(async move {
                        state.write().screen = Screen::Processing;

                        match camera::capture_frame(VIDEO_ID) {
                            Ok((pixels, _w, _h)) => {
                                let _ = camera::stop_camera(VIDEO_ID);
                                let (english, norwegian) = ml::process_image(&pixels);
                                let mut s = state.write();
                                s.detected_label = Some(english);
                                s.translated_label = Some(norwegian);
                                s.screen = Screen::Result;
                            }
                            Err(e) => {
                                log::error!("Capture error: {}", e);
                                let mut s = state.write();
                                s.error = Some(e);
                                s.screen = Screen::Camera;
                            }
                        }
                    });
                },
            }
        }
    }
}

#[component]
fn ProcessingScreen() -> Element {
    rsx! {
        div { class: "processing-screen",
            p { class: "processing-text", "Analyzing..." }
        }
    }
}

#[component]
fn ResultScreen(state: Signal<AppState>) -> Element {
    let detected = state
        .read()
        .detected_label
        .clone()
        .unwrap_or_default();
    let translated = state
        .read()
        .translated_label
        .clone()
        .unwrap_or_default();

    let mut state = state;

    rsx! {
        div { class: "result-screen",
            p { class: "label-english", "{detected}" }
            p { class: "label-norwegian", "{translated}" }
            button {
                class: "try-again-btn",
                onclick: move |_| {
                    *state.write() = AppState::default();
                },
                "Try Again"
            }
        }
    }
}
