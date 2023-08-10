use tauri_egui::egui::{util::History, Ui};

pub struct FramesPerSecond {
    frame_times: History<f32>,
}

impl Default for FramesPerSecond {
    fn default() -> Self {
        let max_age: f32 = 1.0;
        let max_len = (max_age * 300.0).round() as usize;
        Self {
            frame_times: History::new(0..max_len, max_age),
        }
    }
}

impl FramesPerSecond {
    // Called first
    pub fn on_new_frame(&mut self, now: f64, previous_frame_time: Option<f32>) {
        let previous_frame_time = previous_frame_time.unwrap_or_default();
        if let Some(latest) = self.frame_times.latest_mut() {
            *latest = previous_frame_time; // rewrite history now that we know
        }
        self.frame_times.add(now, previous_frame_time); // projected
    }

    fn mean_frame_time(&self) -> f32 {
        self.frame_times.average().unwrap_or_default()
    }

    fn fps(&self) -> f32 {
        1.0 / self.frame_times.mean_time_interval().unwrap_or_default()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        ui.label(format!(
            "FPS: {:.1} ({:.2} ms / frame)",
            self.fps(),
            1e3 * self.mean_frame_time(),
        ))
        .on_hover_text(
            "Includes egui layout and tessellation time.\n\
            Does not include GPU usage, nor overhead for sending data to GPU.",
        );
    }
}
