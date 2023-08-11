use std::path;

use poll_promise::Promise;
use tauri_egui::egui::{Align, CentralPanel, Layout, Response, SidePanel, Ui};

use crate::hledger::{self, Manager};

use super::{
    accounts_tree::AccountsTree, new_transaction_modal::NewTransactionModal, state::State,
    transactions_list,
};

pub struct Tab {
    state: State,

    accounts_tree: AccountsTree,
    add_transaction_modal: NewTransactionModal,

    transactions: Promise<Result<Vec<hledger::Transaction>, hledger::Error>>,
    visible_transactions: Option<Vec<hledger::Transaction>>,
}

impl Tab {
    pub fn new(manager: &Manager, file_path: path::PathBuf, state: State) -> Self {
        Self {
            transactions: Promise::spawn_async({
                let manager = manager.to_owned();
                let file_path = file_path.to_owned();
                async move {
                    let client = manager.client(file_path).await?;
                    client.transactions().await
                }
            }),
            visible_transactions: None,

            accounts_tree: AccountsTree::new(manager, &file_path, &state.tree.clone()),
            add_transaction_modal: NewTransactionModal::new(manager),
            state,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        self.add_transaction_modal.ui(ui, self.visible_transactions.as_ref().unwrap_or(&vec![]));

        let accounts_response = SidePanel::left("accounts_tree")
            .show(ui.ctx(), |ui| {
                let response = self.accounts_tree.ui(ui);

                if response.changed() {
                    self.state.tree = self.accounts_tree.state().clone();
                    self.visible_transactions =
                        self.transactions.ready().and_then(|transactions| {
                            transactions.as_ref().ok().map(|transactions| {
                                filter_visible(transactions.as_ref(), |account_name| {
                                    self.state.tree.checked.contains(account_name)
                                })
                            })
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

                match self.transactions.ready() {
                    None => {
                        ui.vertical_centered(|ui| {
                            ui.ctx().request_repaint();
                            ui.spinner()
                        });
                    }
                    Some(Err(err)) => {
                        ui.vertical_centered(|ui| {
                            ui.heading("Failed to load transactions");
                            ui.label(err.to_string());
                        });
                    }
                    Some(Ok(transactions)) => {
                        let visible_transactions =
                            self.visible_transactions.get_or_insert_with(|| {
                                filter_visible(transactions, |account| {
                                    self.state.tree.checked.contains(account)
                                })
                            });

                        transactions_list::ui(ui, visible_transactions);
                    }
                }
            });
        });

        accounts_response
    }
}

fn filter_visible(
    transactions: &[hledger::Transaction],
    is_account_visible: impl Fn(&hledger::AccountName) -> bool,
) -> Vec<hledger::Transaction> {
    transactions
        .iter()
        .filter_map(|transaction| {
            let postings = transaction
                .postings
                .iter()
                .filter(|posting| is_account_visible(&posting.account))
                .cloned()
                .collect::<Vec<_>>();
            if postings.is_empty() {
                None
            } else {
                Some(hledger::Transaction {
                    postings,
                    ..transaction.clone()
                })
            }
        })
        .collect::<Vec<_>>()
}
