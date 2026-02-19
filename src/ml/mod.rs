mod recognition;

use crate::image_utils::preprocess_for_model;

/// Processes raw RGBA camera data through the ML pipeline.
/// Returns (english_label, norwegian_label).
pub fn process_image(rgba_bytes: &[u8], width: u32, height: u32) -> (String, String) {
    let float_data = match preprocess_for_model(rgba_bytes, width, height) {
        Ok(d) => d,
        Err(e) => {
            log::error!("Preprocessing failed: {e}");
            return ("unknown".to_string(), "ukjent".to_string());
        }
    };
    recognition::recognize(float_data)
}
