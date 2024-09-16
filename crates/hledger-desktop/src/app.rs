use eframe::egui::{Context, FontDefinitions};
use eframe::{self, CreationContext};

use crate::{
    frame::{
        actions::app::Action,
        render,
        state::app::{State, WindowInfo},
    },
    hledger,
};

pub struct App {
    manager: hledger::Manager,
    state: State,
}

impl App {
    pub fn new(cc: &CreationContext<'_>, manager: hledger::Manager, state: State) -> Self {
        // setup phosphor icons
        let mut fonts = FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        cc.egui_ctx.set_visuals(state.theme.into());
        Self { manager, state }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let mut before_render_updates = vec![Action::frame_history(
            ctx.input(|i| i.time),
            frame.info().cpu_usage,
        )];

        let window_info: WindowInfo = ctx.input(|i| {
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
        });
        if window_info != self.state.window {
            before_render_updates.push(Action::window(&window_info));
        }

        let render_updates = render(ctx, &self.state);

        let all_updates = before_render_updates
            .iter()
            .chain(render_updates.iter())
            .collect::<Vec<_>>();

        let should_save = all_updates
            .iter()
            .fold(false, |should_save, update| match update {
                Action::Persistent(update) => {
                    update(&self.manager, &mut self.state);
                    true
                }
                Action::Ephemeral(update) => {
                    update(&self.manager, &mut self.state);
                    should_save
                }
            });

        if should_save {
            if let Err(error) = self.state.save() {
                tracing::error!("failed to save state: {}", error);
            }
        }
    }

    fn on_exit(&mut self, _: Option<&eframe::glow::Context>) {
        futures::executor::block_on(self.manager.shutdown());
    }
}
