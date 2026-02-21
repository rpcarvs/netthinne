use crate::ml::DetectedObject;

#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    Camera,
    Processing,
    Result,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub screen: Screen,
    pub detections: Vec<DetectedObject>,
    pub error: Option<String>,
    pub captured_pixels: Option<(Vec<u8>, u32, u32)>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Camera,
            detections: Vec::new(),
            error: None,
            captured_pixels: None,
        }
    }
}
