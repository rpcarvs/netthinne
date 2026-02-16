use image::{ImageBuffer, Rgba, imageops::FilterType};

/// Preprocesses raw RGBA pixel data for model input.
/// Resizes to 224x224, converts to RGB, normalizes to [0.0, 1.0] in CHW format.
pub fn preprocess_for_model(rgba_bytes: &[u8], width: u32, height: u32) -> Vec<f32> {
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width, height, rgba_bytes.to_vec())
            .expect("failed to create image buffer from raw RGBA data");

    let resized = image::imageops::resize(&img, 224, 224, FilterType::Triangle);

    let mut chw = vec![0.0f32; 3 * 224 * 224];
    for y in 0..224u32 {
        for x in 0..224u32 {
            let pixel = resized.get_pixel(x, y);
            let idx = (y * 224 + x) as usize;
            chw[idx] = pixel[0] as f32 / 255.0;             // R
            chw[224 * 224 + idx] = pixel[1] as f32 / 255.0;  // G
            chw[2 * 224 * 224 + idx] = pixel[2] as f32 / 255.0; // B
        }
    }

    chw
}
