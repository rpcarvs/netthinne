mod recognition;

use crate::image_utils::preprocess_for_model;

/// Processes raw RGBA camera data through the ML pipeline.
/// Returns (english_label, norwegian_label).
pub fn process_image(rgba_bytes: &[u8], width: u32, height: u32) -> (String, String) {
    let float_data = preprocess_for_model(rgba_bytes, width, height).unwrap_or_default();
    recognition::recognize(float_data)
}
