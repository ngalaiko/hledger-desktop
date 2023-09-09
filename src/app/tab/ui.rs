/// TODO: 
/// * remember amount styles for each commodity
/// * allow configuring valuation date for conversion
use std::path;

use poll_promise::Promise;
use tauri_egui::egui::{Align, CentralPanel, ComboBox, Hyperlink, Layout, Response, SidePanel, Ui};

use crate::hledger::{self, Amount, Commodity, Manager, Posting, Transaction};

use super::{
    accounts_tree::AccountsTree,
    converter::Converter,
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
    commodities: Option<Vec<Commodity>>,
    prices: Option<Promise<Result<Vec<hledger::Price>, hledger::Error>>>,
    converter: Option<Converter>,
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
            prices: None,
            commodities: None,
            converter: None,
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
                let tx_created = if let Some(transactions) = self.visible_transactions.as_ref() {
                    let suggestions = self
                        .add_transaction_modal_suggestions
                        .get_or_insert_with(|| Suggestions::from(transactions.as_ref()));
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

                let prices = self.prices.get_or_insert_with(|| {
                    Promise::spawn_async({
                        let client = client.clone();
                        async move { client.prices().await }
                    })
                });

                let mut accounts_response = SidePanel::left("accounts_tree")
                    .show(ui.ctx(), |ui| self.accounts_tree.ui(ui))
                    .inner;

                if accounts_response.changed() {
                    self.state.tree = self.accounts_tree.state().clone();
                    self.visible_transactions = None;
                }

                CentralPanel::default().show(ui.ctx(), |ui| {
                    match (transactions.ready(), prices.ready()) {
                        (None, _) | (_, None) => {
                            ui.vertical_centered(|ui| {
                                ui.ctx().request_repaint();
                                ui.spinner()
                            });
                        }
                        (Some(Err(err)), _) => {
                            ui.vertical_centered(|ui| {
                                ui.heading("Failed to load transactions");
                                ui.label(err.to_string());
                            });
                        }
                        (_, Some(Err(err))) => {
                            ui.vertical_centered(|ui| {
                                ui.heading("Failed to load prices");
                                ui.label(err.to_string());
                            });
                        }
                        (Some(Ok(transactions)), Some(Ok(prices))) => {
                            let commodities = self.commodities.get_or_insert_with(|| {
                                let mut commodities = transactions
                                    .iter()
                                    .flat_map(|tx| tx.postings.iter())
                                    .flat_map(|posting| posting.amount.iter())
                                    .map(|amount| amount.commodity.clone())
                                    .collect::<Vec<_>>();
                                commodities.sort();
                                commodities.dedup();
                                commodities
                            });

                            let mut display_commodity = self
                                .state
                                .display_commodity
                                .as_ref()
                                .map(|c| c.to_string())
                                .unwrap_or("-".to_string());

                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                if ui.button("add transaction").clicked() {
                                    self.add_transaction_modal.open();
                                }

                                if self.state.display_commodity.is_some()
                                    && ui.button("x").clicked()
                                {
                                    self.state.display_commodity = None;
                                    self.visible_transactions = None;
                                    accounts_response.mark_changed();
                                }

                                ComboBox::from_id_source("display commodity")
                                    .selected_text(display_commodity.to_string())
                                    .show_ui(ui, |ui| {
                                        for commodity in commodities {
                                            if ui
                                                .selectable_value(
                                                    &mut display_commodity,
                                                    commodity.to_string(),
                                                    commodity.to_string(),
                                                )
                                                .changed()
                                            {
                                                self.state.display_commodity =
                                                    Some(commodity.to_string());
                                                self.visible_transactions = None;
                                                accounts_response.mark_changed();
                                            }
                                        }
                                    });
                            });

                            ui.separator();

                            let converter =
                                self.converter.get_or_insert_with(|| Converter::new(prices));

                            let visible_transactions =
                                self.visible_transactions.get_or_insert_with(|| {
                                    let transactions = filter_visible(transactions, |account| {
                                        self.state.tree.checked.contains(account)
                                    });
                                    if let Some(commodity) = self.state.display_commodity.as_ref() {
                                        convert_amounts(converter, &transactions, commodity)
                                    } else {
                                        transactions.to_vec()
                                    }
                                });

                            transactions_list::ui(ui, visible_transactions);
                        }
                    }
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

fn convert_amounts(
    converter: &Converter,
    transactions: &[hledger::Transaction],
    target: &Commodity,
) -> Vec<hledger::Transaction> {
    transactions
        .iter()
        .map(|transaction| Transaction {
            postings: transaction
                .postings
                .iter()
                .map(|posting| Posting {
                    amount: posting
                        .amount
                        .iter()
                        .map(|amount| {
                            // TODO: try to use amount's price here
                            if let Ok(quantity) = converter.convert(
                                (&amount.quantity, &amount.commodity),
                                target,
                                &transaction.date,
                            ) {
                                Amount {
                                    commodity: target.clone(),
                                    quantity,
                                    ..amount.clone()
                                }
                            } else {
                                amount.clone()
                            }
                        })
                        .collect::<Vec<_>>(),
                    ..posting.clone()
                })
                .collect::<Vec<_>>(),
            ..transaction.clone()
        })
        .collect::<Vec<_>>()
}
