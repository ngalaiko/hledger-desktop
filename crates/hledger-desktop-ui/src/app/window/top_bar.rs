use eframe::egui::{vec2, Align, Button, Layout, Sense, Ui};

use crate::app::State;
use crate::theme::Theme;
use crate::Command;

use super::tab;

pub fn ui(ui: &mut Ui, state: &State) -> Command<State> {
    ui.horizontal(|ui| {
        if cfg!(target_os = "macos") {
            macos_traffic_lights_box_ui(ui);
            ui.separator();
        }

        let tab_action = tabs_list(ui, state);

        let theme_switch_action = ui
            .with_layout(Layout::right_to_left(Align::Center), |ui| {
                let action = dark_light_mode_switch_ui(ui, state);
                ui.separator();
                action
            })
            .inner;

        tab_action.and_then(theme_switch_action)
    })
    .inner
}

fn tabs_list(ui: &mut Ui, state: &State) -> Command<State> {
    ui.horizontal(|ui| {
        let mut action = Command::none();
        let mut new_selected = None;
        let mut deleted = vec![];

        for (tab_index, tab) in state.tabs.iter().enumerate() {
            let is_selected = state.active_tab_index == Some(tab_index);
            if ui
                .selectable_label(is_selected, tab.name())
                .context_menu(|ui| {
                    if ui.button("Close").clicked() {
                        deleted.push(tab_index);
                        ui.close_menu();
                    }
                })
                .is_some()
            {
                new_selected.replace(tab_index);
            }
        }

        if !state.tabs.is_empty()
            && ui
                .button(egui_phosphor::regular::PLUS)
                .on_hover_text("Open new file")
                .clicked()
        {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                action = action.and_then(Command::<State>::persistent(move |state| {
                    let tab = tab::State::new(file_path.clone());
                    state.tabs.push(tab);
                    state.active_tab_index.replace(state.tabs.len() - 1);
                }));
            }
        }

        if let Some(index) = new_selected {
            action = action.and_then(Command::<State>::persistent(move |state| {
                state.active_tab_index.replace(index);
            }));
        }

        for index in deleted.drain(..) {
            action = action.and_then(Command::<State>::persistent(move |state| {
                state.tabs.remove(index);
                if state.tabs.is_empty() {
                    state.active_tab_index.take();
                } else {
                    state.active_tab_index = state.active_tab_index.map(|i| i.saturating_sub(1));
                }
            }));
        }

        action
    })
    .inner
}

fn macos_traffic_lights_box_ui(ui: &mut Ui) {
    ui.allocate_exact_size(vec2(50.0, 25.0), Sense::click());
}

fn dark_light_mode_switch_ui(ui: &mut Ui, state: &State) -> Command<State> {
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
        Command::<State>::persistent(move |state| state.theme = theme)
    } else {
        Command::none()
    }
}
