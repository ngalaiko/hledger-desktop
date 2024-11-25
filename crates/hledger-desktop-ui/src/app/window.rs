mod bottom_bar;
pub mod tab;
mod top_bar;

use eframe::egui::{CentralPanel, Context, TopBottomPanel, Ui};

use crate::action::Action;
use crate::{frames::Frames, render_mode::RenderMode, theme::Theme, window_info::WindowInfo};

#[derive(Default)]
pub struct State {
    pub tabs: Vec<tab::State>,
    pub active_tab_index: Option<usize>,

    pub theme: Theme,
    pub window: WindowInfo,
    pub frames: Frames,
    pub render_mode: RenderMode,
}

#[must_use]
pub fn render(ctx: &Context, state: &State) -> Action<State> {
    let top_bar_action = TopBottomPanel::top("top_bar")
        .show(ctx, |ui| top_bar::ui(ui, state))
        .inner;

    let bottom_bar_action = TopBottomPanel::bottom("botttom_bar")
        .show(ctx, |ui| bottom_bar::ui(ui, state))
        .inner;

    let central_panel_action = CentralPanel::default()
        .show(ctx, |ui| central_pane_ui(ui, state))
        .inner;

    top_bar_action
        .and_then(bottom_bar_action)
        .and_then(central_panel_action)
}

fn central_pane_ui(ui: &mut Ui, _state: &State) -> Action<State> {
    welcome_screen_ui(ui)
}

fn welcome_screen_ui(ui: &mut Ui) -> Action<State> {
    ui.vertical_centered(|ui| {
        let mut action = Action::noop();

        ui.heading("Welcome to hledger-desktop");
        if ui.button("Open a new file...").clicked() {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                action = action.and_then(Action::<State>::Persistent(Box::new(move |state| {
                    let tab = tab::State::new(file_path.clone());
                    state.tabs.push(tab);
                    state.active_tab_index.replace(state.tabs.len() - 1);
                })));
            }
        }

        let default_file = std::env::var("LEDGER_FILE")
            .map(std::path::PathBuf::from)
            .ok();
        if let Some(default_file) = default_file {
            let default_file_name = default_file.file_name().unwrap().to_str().unwrap();
            if ui.button(format!("Open {default_file_name}")).clicked() {
                action = action.and_then(Action::<State>::Persistent(Box::new(move |state| {
                    let tab = tab::State::new(default_file.clone());
                    state.tabs.push(tab);
                    state.active_tab_index.replace(state.tabs.len() - 1);
                })));
            }
        }

        action
    })
    .inner
}
