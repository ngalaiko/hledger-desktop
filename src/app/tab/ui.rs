use std::path;

use tauri_egui::egui::{Align, CentralPanel, Layout, Response, SidePanel, Ui};

use crate::hledger::Manager;

use super::{
    accounts_tree::AccountsTree, new_transaction_modal::NewTransactionModal, state::State,
    transactions_list::TransactionsList,
};

pub struct Tab {
    state: State,

    transactions_list: TransactionsList,
    accounts_tree: AccountsTree,
    add_transaction_modal: NewTransactionModal,
}

impl Tab {
    pub fn new(manager: &Manager, file_path: path::PathBuf, state: State) -> Self {
        Self {
            accounts_tree: AccountsTree::new(manager, &file_path, &state.tree),
            transactions_list: TransactionsList::new(manager, &file_path),
            add_transaction_modal: NewTransactionModal::new(manager, &file_path),
            state,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        self.add_transaction_modal.ui(ui);

        let accounts_response = SidePanel::left("accounts_tree")
            .show(ui.ctx(), |ui| {
                let response = self.accounts_tree.ui(ui);

                if response.changed() {
                    self.state.tree = self.accounts_tree.state().clone();
                    self.transactions_list.filter_accounts(|account_name| {
                        self.state.tree.checked.contains(account_name)
                    });
                }

                response
            })
            .inner;

        CentralPanel::default().show(ui.ctx(), |ui| {
            ui.vertical(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if ui.button("add transaction").clicked() {
                        self.add_transaction_modal.open();
                    }
                });

                ui.separator();

                self.transactions_list.ui(ui);
            });
        });

        accounts_response
    }
}
