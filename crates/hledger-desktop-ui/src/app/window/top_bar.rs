use eframe::egui::{vec2, Align, Button, Layout, Sense, Ui};
use smol_macros::Executor;

use crate::app::State;
use crate::theme::Theme;

pub fn ui(ui: &mut Ui, _executor: &Executor<'static>, state: &mut State) {
    ui.horizontal(|ui| {
        if cfg!(target_os = "macos") {
            macos_traffic_lights_box_ui(ui);
            ui.separator();
        }

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            dark_light_mode_switch_ui(ui, state);
            ui.separator();
        });
    });
}

fn macos_traffic_lights_box_ui(ui: &mut Ui) {
    ui.allocate_exact_size(vec2(50.0, 25.0), Sense::click());
}

fn dark_light_mode_switch_ui(ui: &mut Ui, state: &mut State) {
    let new_theme = match state.theme {
        Theme::Light => {
            if ui
                .add(Button::new(egui_phosphor::regular::MOON).frame(false))
                .on_hover_text("Switch to dark mode")
                .clicked()
            {
                Some(Theme::Dark)
            } else {
                None
            }
        }
        Theme::Dark => {
            if ui
                .add(Button::new(egui_phosphor::regular::SUN).frame(false))
                .on_hover_text("Switch to light mode")
                .clicked()
            {
                Some(Theme::Light)
            } else {
                None
            }
        }
    };

    if let Some(theme) = new_theme {
        ui.ctx().set_visuals(theme.into());
        state.set_theme(theme);
    }
}
