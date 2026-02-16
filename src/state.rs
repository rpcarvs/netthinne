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
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Camera,
            detected_label: None,
            translated_label: None,
            error: None,
        }
    }
}
