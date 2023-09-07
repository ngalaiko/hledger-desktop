use std::path;

use poll_promise::Promise;
use tauri_egui::egui::{Align, CentralPanel, Hyperlink, Layout, Response, SidePanel, Ui};

use crate::hledger::{self, Manager};

use super::{
    accounts_tree::AccountsTree,
    new_transaction_modal::{NewTransactionModal, Suggestions},
    state::State,
    transactions_list,
};

pub struct Tab {
    state: State,

    accounts_tree: AccountsTree,
    add_transaction_modal: NewTransactionModal,

    client: Promise<Result<hledger::Client, hledger::Error>>,
    transactions: Option<Promise<Result<Vec<hledger::Transaction>, hledger::Error>>>,
    visible_transactions: Option<Vec<hledger::Transaction>>,
    add_transaction_modal_suggestions: Option<Suggestions>,
}

impl Tab {
    pub fn new(manager: &Manager, file_path: path::PathBuf, state: State) -> Self {
        Self {
            client: Promise::spawn_async({
                let manager = manager.to_owned();
                let file_path = file_path.to_owned();
                async move { manager.client(file_path).await }
            }),
            transactions: None,
            add_transaction_modal_suggestions: None,
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
        match self.client.ready() {
            None => {
                ui.vertical_centered(|ui| {
                    ui.ctx().request_repaint();
                    ui.spinner()
                })
                .inner
            }
            Some(Err(hledger::Error::Process(hledger::ProcessError::NotFound))) => {
                ui.vertical_centered(|ui| {
                    ui.heading("Hledger is not installed");
                    ui.label("Follow official installation instructions to continue:");
                    ui.add(Hyperlink::from_label_and_url(
                        "Hledger documentation",
                        "https://hledger.org/install.html",
                    ))
                })
                .inner
            }
            Some(Err(err)) => {
                ui.vertical_centered(|ui| {
                    ui.heading("Failed to start hledger");
                    ui.label(err.to_string())
                })
                .inner
            }
            Some(Ok(client)) => {
                let tx_created = if let Some(tx) = self.visible_transactions.as_ref() {
                    let suggestions = self
                        .add_transaction_modal_suggestions
                        .get_or_insert_with(|| Suggestions::from(tx.as_ref()));
                    self.add_transaction_modal.ui(ui, suggestions)
                } else {
                    self.add_transaction_modal.ui(ui, &Suggestions::default())
                };

                if tx_created {
                    self.transactions = Some(Promise::spawn_async({
                        let client = client.clone();
                        async move { client.transactions().await }
                    }));
                    self.visible_transactions = None;
                    self.add_transaction_modal_suggestions = None;
                }

                let transactions = self.transactions.get_or_insert_with(|| {
                    Promise::spawn_async({
                        let client = client.clone();
                        async move { client.transactions().await }
                    })
                });

                let accounts_response = SidePanel::left("accounts_tree")
                    .show(ui.ctx(), |ui| {
                        let response = self.accounts_tree.ui(ui);

                        if response.changed() {
                            self.state.tree = self.accounts_tree.state().clone();
                            self.visible_transactions =
                                transactions.ready().and_then(|transactions| {
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

                        match transactions.ready() {
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
