#[derive(Clone, Debug, PartialEq)]
pub enum Screen {
    Camera,
    Processing,
    Result,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub screen: Screen,
    pub detected_label: Option<String>,
    pub translated_label: Option<String>,
    pub error: Option<String>,
    pub captured_pixels: Option<(Vec<u8>, u32, u32)>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Camera,
            detected_label: None,
            translated_label: None,
            error: None,
            captured_pixels: None,
        }
    }
}
