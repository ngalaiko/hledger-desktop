use tauri_egui::egui::{Button, Response, Ui, Visuals};

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Into<Visuals> for Theme {
    fn into(self) -> Visuals {
        match self {
            Theme::Light => Visuals::light(),
            Theme::Dark => Visuals::dark(),
        }
    }
}

pub fn ui(ui: &mut Ui, current_value: &mut Theme) -> Response {
    match current_value {
        Theme::Light => {
            let response = ui
                .add(Button::new("🌙").frame(false))
                .on_hover_text("Switch to dark mode");
            if response.clicked() && !matches!(current_value, Theme::Dark) {
                *current_value = Theme::Dark;
                ui.ctx().set_visuals(Theme::Dark.into());
            }
            response
        }
        Theme::Dark => {
            let response = ui
                .add(Button::new("☀").frame(false))
                .on_hover_text("Switch to light mode");
            if response.clicked() && !matches!(current_value, Theme::Light) {
                *current_value = Theme::Light;
                ui.ctx().set_visuals(Theme::Light.into());
            }
            response
        }
    }
}
