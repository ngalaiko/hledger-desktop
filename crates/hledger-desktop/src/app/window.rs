mod bottom_bar;
pub mod file;
mod top_bar;

use std::sync::Arc;

use eframe::egui::{CentralPanel, Context, TopBottomPanel, Ui};
use smol_macros::Executor;

use crate::{frames::Frames, render_mode::RenderMode, theme::Theme};

#[derive(Default)]
pub struct State {
    pub file: Option<file::File>,

    pub theme: Theme,
    pub frames: Frames,
    pub render_mode: RenderMode,

    // if should_save is true, state will be flushed on disk after rendering current frame.
    pub should_save: bool,
}

impl State {
    pub fn open_file<P: AsRef<std::path::Path>>(
        &mut self,
        executor: Arc<Executor<'static>>,
        file_path: P,
    ) {
        self.file = Some(file::File::new(executor, file_path));
        self.should_save = true;
    }

    pub fn close_file(&mut self) {
        self.file = None;
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

pub fn render(ctx: &Context, executor: Arc<Executor<'static>>, state: &mut State) {
    TopBottomPanel::top("top_bar").show(ctx, |ui| top_bar::ui(ui, executor.clone(), state));
    TopBottomPanel::bottom("botttom_bar").show(ctx, |ui| bottom_bar::ui(ui, state));
    CentralPanel::default().show(ctx, |ui| central_pane_ui(ui, executor, state));
}

fn central_pane_ui(ui: &mut Ui, executor: Arc<Executor<'static>>, state: &mut State) {
    if let Some(file) = &mut state.file {
        file::ui(ui, file);
    } else {
        welcome_screen_ui(ui, executor, state);
    }
}

fn welcome_screen_ui(ui: &mut Ui, executor: Arc<Executor<'static>>, state: &mut State) {
    ui.vertical_centered(|ui| {
        ui.heading("Welcome to hledger-desktop");
        if ui.button("Open a new file...").clicked() {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                state.open_file(executor.clone(), file_path);
            }
        }

        let default_file = std::env::var("LEDGER_FILE")
            .map(std::path::PathBuf::from)
            .ok();

        if let Some(default_file) = default_file {
            let default_file_name = default_file.file_name().unwrap().to_str().unwrap();
            if ui.button(format!("Open {default_file_name}")).clicked() {
                state.open_file(executor, default_file);
            }
        }
    });
}
