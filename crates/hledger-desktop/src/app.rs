pub mod window;

use std::sync::Arc;

use eframe::egui::{Context, FontDefinitions};
use eframe::{self, CreationContext};
use smol_macros::Executor;

use crate::persistance;

pub use self::window::State;

pub struct App {
    state: State,
    executor: Arc<Executor<'static>>,
}

impl App {
    #[must_use]
    pub fn new(cc: &CreationContext<'_>, executor: Arc<Executor<'static>>) -> Self {
        let mut fonts = FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        let state = if let Some(storage) = cc.storage {
            persistance::load_state(storage, executor.clone())
                .map_err(|error| tracing::error!(%error, "failed to load state"))
                .unwrap_or_default()
        } else {
            State::default()
        };

        cc.egui_ctx.set_fonts(fonts);

        Self { state, executor }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        let previous_time_frame = frame.info().cpu_usage;
        self.state.frames.on_new_frame(now, previous_time_frame);

        window::render(ctx, self.executor.clone(), &mut self.state);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.state.should_save {
            if let Err(error) = persistance::save_state(storage, &self.state) {
                tracing::error!("failed to save state: {}", error);
            }
            self.state.should_save = false;
        }
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}
