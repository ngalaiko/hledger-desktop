mod bottom_bar;
pub mod tab;
mod top_bar;

use eframe::egui::{CentralPanel, Context, TopBottomPanel, Ui};
use smol_macros::Executor;

use crate::{frames::Frames, render_mode::RenderMode, theme::Theme};

#[derive(Default)]
pub struct State {
    pub tabs: Vec<tab::State>,
    pub active_tab_index: Option<usize>,

    pub theme: Theme,
    pub frames: Frames,
    pub render_mode: RenderMode,

    // if should_save is true, state will be flushed on disk after rendering current frame.
    pub should_save: bool,
}

impl State {
    pub fn open_tab<P: AsRef<std::path::Path>>(
        &mut self,
        executor: &Executor<'static>,
        file_path: P,
    ) {
        let tab = tab::State::new(executor, file_path);
        self.tabs.push(tab);
        self.active_tab_index.replace(self.tabs.len() - 1);
        self.should_save = true;
    }

    pub fn set_active_tab(&mut self, index: usize) {
        self.active_tab_index.replace(index);
        self.should_save = true;
    }

    pub fn close_tab(&mut self, index: usize) {
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            self.active_tab_index.take();
        } else {
            self.active_tab_index = self.active_tab_index.map(|i| i.saturating_sub(1));
        }
        self.should_save = true;
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.should_save = true;
    }

    pub fn set_render_mode(&mut self, render_mode: RenderMode) {
        self.render_mode = render_mode;
    }
}

pub fn render(ctx: &Context, executor: &Executor<'static>, state: &mut State) {
    TopBottomPanel::top("top_bar").show(ctx, |ui| top_bar::ui(ui, executor, state));
    TopBottomPanel::bottom("botttom_bar").show(ctx, |ui| bottom_bar::ui(ui, state));
    CentralPanel::default().show(ctx, |ui| central_pane_ui(ui, executor, state));
}

fn central_pane_ui(ui: &mut Ui, executor: &Executor<'static>, state: &mut State) {
    if let Some(active_tab_index) = state.active_tab_index {
        let active_tab = state
            .tabs
            .get_mut(active_tab_index)
            .expect("active tab index is valid");
        tab::ui(ui, active_tab);
    } else {
        welcome_screen_ui(ui, executor, state);
    }
}

fn welcome_screen_ui(ui: &mut Ui, executor: &Executor<'static>, state: &mut State) {
    ui.vertical_centered(|ui| {
        ui.heading("Welcome to hledger-desktop");
        if ui.button("Open a new file...").clicked() {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                state.open_tab(executor, file_path);
            }
        }

        let default_file = std::env::var("LEDGER_FILE")
            .map(std::path::PathBuf::from)
            .ok();

        if let Some(default_file) = default_file {
            let default_file_name = default_file.file_name().unwrap().to_str().unwrap();
            if ui.button(format!("Open {default_file_name}")).clicked() {
                state.open_tab(executor, default_file);
            }
        }
    });
}
