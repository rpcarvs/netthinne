use image::{imageops, RgbImage};

const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Converts raw RGBA pixels to a 1x3x224x224 CHW float32 tensor (flat Vec).
/// Applies ImageNet mean/std normalization as required by MobileNetV2.
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

    let resized = imageops::resize(&img, 224, 224, imageops::FilterType::Triangle);

    // CHW layout: all R, then all G, then all B
    let mut out = vec![0.0f32; 3 * 224 * 224];
    for (i, pixel) in resized.pixels().enumerate() {
        for c in 0..3 {
            out[c * 224 * 224 + i] = (pixel[c] as f32 / 255.0 - MEAN[c]) / STD[c];
        }
    }

    Ok(out)
}
