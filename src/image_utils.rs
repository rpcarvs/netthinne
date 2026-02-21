use base64::{engine::general_purpose::STANDARD, Engine};
use image::{codecs::png::PngEncoder, imageops, ImageEncoder, RgbImage};

const MOBILENET_SIZE: usize = 224;
const MOBILENET_MEAN: f32 = 0.5;
const MOBILENET_STD: f32 = 0.5;
const YOLO_SIZE: usize = 640;

/// Converts raw RGBA pixels to a float32 NCHW tensor for YOLOv8 (640x640, /255 only).
pub fn preprocess_for_yolo(
    rgba_bytes: &[u8],
    source_width: u32,
    source_height: u32,
) -> Result<Vec<f32>, String> {
    let img = rgba_to_rgb(rgba_bytes, source_width, source_height)?;
    let resized = imageops::resize(
        &img,
        YOLO_SIZE as u32,
        YOLO_SIZE as u32,
        imageops::FilterType::Triangle,
    );
    let pixels = YOLO_SIZE * YOLO_SIZE;
    let mut out = vec![0.0f32; 3 * pixels];
    for (i, pixel) in resized.pixels().enumerate() {
        out[i] = pixel[0] as f32 / 255.0;
        out[pixels + i] = pixel[1] as f32 / 255.0;
        out[2 * pixels + i] = pixel[2] as f32 / 255.0;
    }
    Ok(out)
}

/// Crops a bounding box region from RGBA pixels and preprocesses for MobileNetV2.
pub fn crop_and_preprocess(
    rgba_bytes: &[u8],
    source_width: u32,
    source_height: u32,
    bbox: [f32; 4],
) -> Result<Vec<f32>, String> {
    let img = rgba_to_rgb(rgba_bytes, source_width, source_height)?;
    let cropped = crop_region(&img, bbox);
    let resized = imageops::resize(
        &cropped,
        MOBILENET_SIZE as u32,
        MOBILENET_SIZE as u32,
        imageops::FilterType::Triangle,
    );
    Ok(normalize_mobilenet(&resized))
}

/// Crops a bounding box region and returns a base64 PNG data URL for display.
pub fn crop_to_data_url(
    rgba_bytes: &[u8],
    source_width: u32,
    source_height: u32,
    bbox: [f32; 4],
) -> Result<String, String> {
    let img = rgba_to_rgb(rgba_bytes, source_width, source_height)?;
    let cropped = crop_region(&img, bbox);
    let mut buf = Vec::new();
    PngEncoder::new(&mut buf)
        .write_image(
            cropped.as_raw(),
            cropped.width(),
            cropped.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("PNG encode failed: {e}"))?;
    let b64 = STANDARD.encode(&buf);
    Ok(format!("data:image/png;base64,{b64}"))
}

fn rgba_to_rgb(rgba: &[u8], w: u32, h: u32) -> Result<RgbImage, String> {
    let rgb: Vec<u8> = rgba.chunks(4).flat_map(|p| [p[0], p[1], p[2]]).collect();
    RgbImage::from_raw(w, h, rgb).ok_or_else(|| "Failed to create image from raw bytes".into())
}

fn crop_region(img: &RgbImage, bbox: [f32; 4]) -> RgbImage {
    let x1 = (bbox[0].max(0.0) as u32).min(img.width().saturating_sub(1));
    let y1 = (bbox[1].max(0.0) as u32).min(img.height().saturating_sub(1));
    let x2 = (bbox[2].max(0.0).ceil() as u32).min(img.width());
    let y2 = (bbox[3].max(0.0).ceil() as u32).min(img.height());
    let w = (x2.saturating_sub(x1)).max(1);
    let h = (y2.saturating_sub(y1)).max(1);
    imageops::crop_imm(img, x1, y1, w, h).to_image()
}

fn normalize_mobilenet(img: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> Vec<f32> {
    let pixels = (img.width() * img.height()) as usize;
    let mut out = vec![0.0f32; 3 * pixels];
    for (i, pixel) in img.pixels().enumerate() {
        out[i] = (pixel[0] as f32 / 255.0 - MOBILENET_MEAN) / MOBILENET_STD;
        out[pixels + i] = (pixel[1] as f32 / 255.0 - MOBILENET_MEAN) / MOBILENET_STD;
        out[2 * pixels + i] = (pixel[2] as f32 / 255.0 - MOBILENET_MEAN) / MOBILENET_STD;
    }
    out
}
