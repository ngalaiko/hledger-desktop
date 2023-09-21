use tauri::AppHandle;
use tauri_egui::{
    eframe::CreationContext,
    egui::{Context, FontDefinitions},
};

use crate::{state::State, ui::show};

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

        let state = State::from(&handle);
        cc.egui_ctx.set_visuals(state.theme().into());
        Self { state, handle }
    }
}

impl tauri_egui::eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut tauri_egui::eframe::Frame) {
        // update fps counter
        self.state
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);

        let updates = show(ctx, &self.state);

        self.state.apply_updates(ctx, &self.handle, &updates);
    }
}
