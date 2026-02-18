use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, HtmlCanvasElement, HtmlVideoElement, MediaStreamConstraints, MediaStreamTrack,
};

/// Starts the camera and attaches the stream to a video element.
pub async fn start_camera(video_id: &str) -> Result<(), String> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    let navigator = window.navigator();
    let media_devices = navigator
        .media_devices()
        .map_err(|e| format!("no media devices: {:?}", e))?;

    let constraints = MediaStreamConstraints::new();
    constraints.set_audio(&JsValue::FALSE);

    let video_constraints = js_sys::Object::new();
    js_sys::Reflect::set(
        &video_constraints,
        &"facingMode".into(),
        &"environment".into(),
    )
    .map_err(|e| format!("failed to set facingMode: {:?}", e))?;
    constraints.set_video(&video_constraints);

    let promise = media_devices
        .get_user_media_with_constraints(&constraints)
        .map_err(|e| format!("getUserMedia failed: {:?}", e))?;

    let stream = JsFuture::from(promise)
        .await
        .map_err(|e| format!("camera stream error: {:?}", e))?;

    let video_el: HtmlVideoElement = document
        .get_element_by_id(video_id)
        .ok_or("video element not found")?
        .dyn_into()
        .map_err(|_| "element is not a video")?;

    video_el.set_src_object(Some(
        &stream
            .dyn_into()
            .map_err(|_| "stream is not a MediaStream")?,
    ));

    Ok(())
}

/// Captures the current frame from the video element as RGBA pixel data.
/// Returns (rgba_bytes, width, height).
pub fn capture_frame(video_id: &str) -> Result<(Vec<u8>, u32, u32), String> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    let video_el: HtmlVideoElement = document
        .get_element_by_id(video_id)
        .ok_or("video element not found")?
        .dyn_into()
        .map_err(|_| "element is not a video")?;

    let width = video_el.video_width();
    let height = video_el.video_height();

    if width == 0 || height == 0 {
        return Err("video not ready".to_string());
    }

    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .map_err(|e| format!("failed to create canvas: {:?}", e))?
        .dyn_into()
        .map_err(|_| "element is not a canvas")?;

    canvas.set_width(width);
    canvas.set_height(height);

    let ctx = canvas
        .get_context("2d")
        .map_err(|e| format!("failed to get 2d context: {:?}", e))?
        .ok_or("no 2d context")?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| "context is not CanvasRenderingContext2d")?;

    ctx.draw_image_with_html_video_element(&video_el, 0.0, 0.0)
        .map_err(|e| format!("drawImage failed: {:?}", e))?;

    let image_data = ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .map_err(|e| format!("getImageData failed: {:?}", e))?;

    Ok((image_data.data().to_vec(), width, height))
}

/// Stops all camera tracks and clears the video source.
pub fn stop_camera(video_id: &str) -> Result<(), String> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;

    let video_el: HtmlVideoElement = document
        .get_element_by_id(video_id)
        .ok_or("video element not found")?
        .dyn_into()
        .map_err(|_| "element is not a video")?;

    if let Some(stream) = video_el.src_object() {
        let media_stream: web_sys::MediaStream = stream
            .dyn_into()
            .map_err(|_| "srcObject is not a MediaStream")?;

        let tracks = media_stream.get_tracks();
        for i in 0..tracks.length() {
            if let Some(track) = tracks.get(i).dyn_ref::<MediaStreamTrack>() {
                track.stop();
            }
        }
    }

    video_el.set_src_object(None);
    Ok(())
}
