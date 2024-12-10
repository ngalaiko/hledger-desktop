use eframe::egui::{ScrollArea, Ui};
use egui_virtual_list::VirtualList;
use smol_macros::Executor;

use crate::{journal, widgets, Command};

pub struct State {
    pub file_path: std::path::PathBuf,
    watcher: journal::Watcher,
    list: VirtualList,
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
            list: VirtualList::new(),
        }
    }
}

pub fn ui<'frame>(ui: &mut Ui, state: &mut State) -> Command<'frame, State> {
    let journal_guard = state.watcher.journal();
    let error_guard = state.watcher.error();
    if !error_guard.is_empty() {
        ui.label("error");
        Command::none()
    } else if let Some(journal) = journal_guard.as_ref() {
        transactions_list_ui(
            ui,
            journal.transactions().collect::<Vec<_>>().as_slice(),
            state,
        )
    } else {
        widgets::spinner_ui(ui);
        Command::none()
    }
}

fn transactions_list_ui<'frame>(
    ui: &mut Ui,
    transactions: &[&hledger_journal::Transaction],
    state: &mut State,
) -> Command<'frame, State> {
    ScrollArea::vertical().show(ui, |ui| {
        ui.set_width(ui.available_width());
        state
            .list
            .ui_custom_layout(ui, transactions.len(), |ui, index| {
                let transaction = &transactions[index];
                ui.label(transaction.payee.clone());
                1
            })
    });
    Command::none()
}
