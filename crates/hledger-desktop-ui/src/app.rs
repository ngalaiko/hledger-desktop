pub mod window;

use std::sync::Arc;

use eframe::egui::{Context, FontDefinitions};
use eframe::{self, CreationContext};
use smol_macros::Executor;

use crate::window_info::WindowInfo;

pub use self::window::State;

pub struct App {
    state: State,
    executor: Arc<Executor<'static>>,
}

impl App {
    #[must_use]
    pub fn new(cc: &CreationContext<'_>, executor: Arc<Executor<'static>>, state: State) -> Self {
        let mut fonts = FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        cc.egui_ctx.set_fonts(fonts);

        Self { state, executor }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let now = ctx.input(|i| i.time);
        let previous_time_frame = frame.info().cpu_usage;
        self.state.frames.on_new_frame(now, previous_time_frame);

        self.state.set_window_info(ctx.input(|i| {
            let viewport = i.viewport();
            let mut window_info = WindowInfo::default();
            if let Some(maximized) = viewport.maximized {
                window_info.maximized = maximized;
            }
            if let Some(fullscreen) = viewport.fullscreen {
                window_info.fullscreen = fullscreen;
            }
            if let Some(size) = viewport.inner_rect {
                window_info.size = [size.size().x, size.size().y];
            }
            window_info.position = viewport.outer_rect.map(|rect| [rect.min.x, rect.min.y]);
            window_info
        }));

        window::render(ctx, &self.executor, &mut self.state);

        self.state.save();
    }
}
