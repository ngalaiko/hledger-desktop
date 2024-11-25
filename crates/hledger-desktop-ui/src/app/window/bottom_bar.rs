use eframe::egui::{Align, Button, Layout, Ui};

use crate::action::Action;

use crate::app::State;
use crate::render_mode::RenderMode;

pub fn ui(ui: &mut Ui, state: &State) -> Action<State> {
    ui.horizontal(|ui| {
        if cfg!(debug_assertions) {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let action = render_mode_ui(ui, state);

                frames_per_second_ui(ui, state);

                ui.separator();
                action
            })
            .inner
        } else {
            Action::noop()
        }
    })
    .inner
}

fn frames_per_second_ui(ui: &mut Ui, state: &State) {
    ui.label(format!(
        "FPS: {:.1} ({:.2} ms / frame)",
        state.frames.per_second(),
        1e3 * state.frames.mean_time(),
    ))
    .on_hover_text(
        "Includes egui layout and tessellation time.\n\
            Does not include GPU usage, nor overhead for sending data to GPU.",
    );
}

fn render_mode_ui(ui: &mut Ui, state: &State) -> Action<State> {
    match state.render_mode {
        RenderMode::Continious => {
            ui.ctx().request_repaint();
            if ui
                .add(Button::new(egui_phosphor::regular::PLAY).frame(false))
                .on_hover_text("Switch to reactive rendering")
                .clicked()
            {
                Action::Ephemeral(Box::new(move |state| {
                    state.render_mode = RenderMode::Reactive;
                }))
            } else {
                Action::noop()
            }
        }
        RenderMode::Reactive => {
            if ui
                .add(Button::new(egui_phosphor::regular::PAUSE).frame(false))
                .on_hover_text("Switch to continious rendering")
                .clicked()
            {
                Action::Ephemeral(Box::new(move |state| {
                    state.render_mode = RenderMode::Continious;
                }))
            } else {
                Action::noop()
            }
        }
    }
}
