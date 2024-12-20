use eframe::emath::History;

pub struct Frames {
    times: History<f32>,
}

impl Default for Frames {
    fn default() -> Self {
        Self {
            times: History::new(2..100, 1.0),
        }
    }
}

impl Frames {
    pub fn on_new_frame(&mut self, now: f64, previous_frame_time: Option<f32>) {
        let previous_frame_time = previous_frame_time.unwrap_or_default();
        if let Some(latest) = self.times.latest_mut() {
            *latest = previous_frame_time; // rewrite history now that we know
        }
        self.times.add(now, previous_frame_time); // projected
    }

    #[must_use]
    pub fn mean_time(&self) -> f32 {
        self.times.average().unwrap_or_default()
    }

    #[must_use]
    pub fn per_second(&self) -> f32 {
        1.0 / self.times.mean_time_interval().unwrap_or_default()
    }
}
