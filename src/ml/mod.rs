mod recognition;

use crate::translation;

/// Processes raw image data through the ML pipeline.
/// Returns (english_label, norwegian_label).
pub fn process_image(image_data: &[u8]) -> (String, String) {
    let english = recognition::recognize(image_data);
    let norwegian = translation::translate(&english);
    (english, norwegian)
}
