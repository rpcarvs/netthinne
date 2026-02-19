use image::{imageops, RgbImage};

mod model_config {
    include!(concat!(env!("OUT_DIR"), "/ml/model_config.rs"));
}
use model_config::{IMG_H, IMG_W, IS_BGR, IS_NHWC, MEAN, STD};

/// Converts raw RGBA pixels to a float32 tensor matching the active model's expected format.
/// Layout (NCHW or NHWC), channel order (RGB or BGR), and normalization are all config-driven.
pub fn preprocess_for_model(
    rgba_bytes: &[u8],
    source_width: u32,
    source_height: u32,
) -> Result<Vec<f32>, String> {
    let rgb_bytes: Vec<u8> = rgba_bytes
        .chunks(4)
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect();

    let img = RgbImage::from_raw(source_width, source_height, rgb_bytes)
        .ok_or("Failed to create image from raw bytes")?;

    let resized = imageops::resize(&img, IMG_W as u32, IMG_H as u32, imageops::FilterType::Triangle);

    let total = IMG_W * IMG_H * 3;
    let mut out = vec![0.0f32; total];

    for (i, pixel) in resized.pixels().enumerate() {
        // Normalize each channel then optionally swap to BGR order
        let vals = if IS_BGR {
            [
                (pixel[2] as f32 / 255.0 - MEAN[0]) / STD[0], // B → channel 0
                (pixel[1] as f32 / 255.0 - MEAN[1]) / STD[1], // G → channel 1
                (pixel[0] as f32 / 255.0 - MEAN[2]) / STD[2], // R → channel 2
            ]
        } else {
            [
                (pixel[0] as f32 / 255.0 - MEAN[0]) / STD[0],
                (pixel[1] as f32 / 255.0 - MEAN[1]) / STD[1],
                (pixel[2] as f32 / 255.0 - MEAN[2]) / STD[2],
            ]
        };

        if IS_NHWC {
            out[i * 3 + 0] = vals[0];
            out[i * 3 + 1] = vals[1];
            out[i * 3 + 2] = vals[2];
        } else {
            out[0 * IMG_H * IMG_W + i] = vals[0];
            out[1 * IMG_H * IMG_W + i] = vals[1];
            out[2 * IMG_H * IMG_W + i] = vals[2];
        }
    }

    Ok(out)
}
