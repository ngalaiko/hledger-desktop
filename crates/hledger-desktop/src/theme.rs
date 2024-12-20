use eframe::egui::Visuals;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl From<&Theme> for Visuals {
    fn from(value: &Theme) -> Self {
        match value {
            Theme::Light => Visuals::light(),
            Theme::Dark => Visuals::dark(),
        }
    }
}

impl From<Theme> for Visuals {
    fn from(value: Theme) -> Self {
        Self::from(&value)
    }
}
