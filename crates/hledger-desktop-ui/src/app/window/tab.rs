use eframe::egui::Ui;
use smol_macros::Executor;

use crate::{journal, widgets, Command};

pub struct State {
    pub file_path: std::path::PathBuf,
    watcher: journal::Watcher,
}

impl State {
    #[must_use]
    pub fn name(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    }

    pub fn new<P: AsRef<std::path::Path>>(executor: &Executor<'static>, path: P) -> Self {
        let path = path.as_ref();
        let path_clone = path.to_path_buf();
        Self {
            file_path: path.to_path_buf(),
            watcher: journal::Watcher::watch(executor, path_clone),
        }
    }
}

pub fn ui<'frame>(ui: &mut Ui, state: &State) -> Command<'frame, State> {
    ui.label(format!("i am {}", state.name()));
    let journal_guard = state.watcher.journal();
    let error_guard = state.watcher.error();
    if !error_guard.is_empty() {
        ui.label("error");
    } else if let Some(journal) = journal_guard.as_ref() {
        let count = journal.transactions().count();
        ui.label(format!("{count} transactions"));
    } else {
        widgets::spinner_ui(ui);
    }
    Command::none()
}
