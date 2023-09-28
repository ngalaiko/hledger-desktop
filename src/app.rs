use tauri::AppHandle;
use tauri_egui::{
    eframe::CreationContext,
    egui::{Context, FontDefinitions},
};

use crate::frame::{actions::app::Action, render, state::app::State};

pub struct App {
    handle: AppHandle,
    state: State,
}

impl App {
    pub fn new(cc: &CreationContext<'_>, handle: AppHandle) -> Self {
        // setup phosphor icons
        let mut fonts = FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        let state = State::try_from(&handle).unwrap_or_default();
        cc.egui_ctx.set_visuals(state.theme.into());
        Self { state, handle }
    }
}

impl tauri_egui::eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut tauri_egui::eframe::Frame) {
        let before_render_updates = &[Action::frame_history(
            ctx.input(|i| i.time),
            frame.info().cpu_usage,
        )];
        let render_updates = render(ctx, &self.state);

        let all_updates = before_render_updates
            .iter()
            .chain(render_updates.iter())
            .collect::<Vec<_>>();

        let should_save = all_updates
            .iter()
            .fold(false, |should_save, update| match update {
                Action::Persistent(update) => {
                    update(&self.handle, &mut self.state);
                    true
                }
                Action::Ephemeral(update) => {
                    update(&self.handle, &mut self.state);
                    should_save
                }
            });

        if should_save {
            if let Err(error) = self.state.save(&self.handle) {
                tracing::error!("failed to save state: {}", error);
            }
        }
    }
}
