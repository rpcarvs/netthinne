use image::{imageops, RgbImage};

const IMG_SIZE: usize = 224;
const MEAN: f32 = 0.5;
const STD: f32 = 0.5;

/// Converts raw RGBA pixels to a float32 NCHW tensor for MobileNetV2.
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

    let resized = imageops::resize(
        &img,
        IMG_SIZE as u32,
        IMG_SIZE as u32,
        imageops::FilterType::Triangle,
    );

    let pixels = IMG_SIZE * IMG_SIZE;
    let mut out = vec![0.0f32; 3 * pixels];

    for (i, pixel) in resized.pixels().enumerate() {
        out[i] = (pixel[0] as f32 / 255.0 - MEAN) / STD;
        out[pixels + i] = (pixel[1] as f32 / 255.0 - MEAN) / STD;
        out[2 * pixels + i] = (pixel[2] as f32 / 255.0 - MEAN) / STD;
    }

    Ok(out)
}
