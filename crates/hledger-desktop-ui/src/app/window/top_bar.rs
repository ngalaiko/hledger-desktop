use eframe::egui::{menu, vec2, Align, Button, Layout, Sense, Ui};
use smol_macros::Executor;

use crate::app::State;
use crate::theme::Theme;

pub fn ui(ui: &mut Ui, executor: &Executor<'static>, state: &mut State) {
    menu::bar(ui, |ui| {
        if cfg!(target_os = "macos") {
            macos_traffic_lights_box_ui(ui);
            ui.separator();
        }
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                    state.open_file(executor, file_path);
                    ui.close_menu();
                }
            }

            if ui
                .add_enabled(state.file.is_some(), Button::new("Close"))
                .clicked()
            {
                state.close_file();
                ui.close_menu();
            }
        });

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
