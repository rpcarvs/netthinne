mod recognition;
mod segmentation;

use crate::image_utils::{crop_and_preprocess, crop_to_data_url, preprocess_for_yolo};

#[derive(Clone, Debug)]
pub struct DetectedObject {
    pub image_data_url: String,
    pub yolo_label_en: String,
    pub yolo_label_no: String,
    pub inet_label_en: String,
    pub inet_label_no: String,
}

/// Two-stage pipeline: YOLO detection then MobileNetV2 classification per crop.
pub fn process_image(rgba_bytes: &[u8], width: u32, height: u32) -> Vec<DetectedObject> {
    let yolo_input = match preprocess_for_yolo(rgba_bytes, width, height) {
        Ok(d) => d,
        Err(e) => {
            log::error!("YOLO preprocessing failed: {e}");
            return Vec::new();
        }
    };

    let detections = segmentation::detect(yolo_input, width, height);

    detections
        .into_iter()
        .filter_map(|det| {
            let image_data_url = crop_to_data_url(rgba_bytes, width, height, det.bbox)
                .map_err(|e| log::error!("Crop failed: {e}"))
                .ok()?;

            let crop_input = crop_and_preprocess(rgba_bytes, width, height, det.bbox)
                .map_err(|e| log::error!("Crop preprocess failed: {e}"))
                .ok()?;

            let (inet_en, inet_no) = recognition::recognize(crop_input);

            Some(DetectedObject {
                image_data_url,
                yolo_label_en: segmentation::label_en(det.class_idx),
                yolo_label_no: segmentation::label_no(det.class_idx),
                inet_label_en: inet_en,
                inet_label_no: inet_no,
            })
        })
        .collect()
}
