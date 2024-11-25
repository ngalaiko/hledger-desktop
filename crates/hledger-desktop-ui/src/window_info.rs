use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowInfo {
    pub position: Option<[f32; 2]>,
    pub size: [f32; 2],
    pub fullscreen: bool,
    pub maximized: bool,
}

impl Default for WindowInfo {
    fn default() -> Self {
        Self {
            position: None,
            size: [800.0, 600.0],
            fullscreen: false,
            maximized: false,
        }
    }
}
