use std::path;

use egui_extras::{Column, TableBuilder, TableRow};
use poll_promise::Promise;
use tauri_egui::egui::{TextStyle, Ui};

use crate::hledger;

#[derive(Clone)]
pub struct Posting {
    account_name: hledger::AccountName,
    amount: String,
}

impl Posting {
    pub fn new(posting: &hledger::Posting) -> Vec<Self> {
        posting
            .amount
            .iter()
            .map(|amount| Self {
                account_name: posting.account.clone(),
                amount: amount.to_string(),
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct Transaction {
    date: String,
    description: String,
    postings: Vec<(Posting, bool)>,
}

impl Transaction {
    pub fn new(transaction: &hledger::Transaction) -> Self {
        Self {
            date: transaction.date.to_string(),
            description: transaction.description.to_string(),
            postings: transaction
                .postings
                .iter()
                .flat_map(Posting::new)
                .map(|posting| (posting, true))
                .collect(),
        }
    }

    fn is_visible(&self) -> bool {
        self.postings.iter().any(|(_, visible)| *visible)
    }

    pub fn height(&self, ui: &Ui) -> f32 {
        let visible_postings_count = self.postings.iter().filter(|(_, visible)| *visible).count();
        if visible_postings_count == 0 {
            0.0
        } else {
            let row_height = ui.text_style_height(&TextStyle::Monospace);
            let inter_height = ui.spacing().item_spacing.y;
            row_height * visible_postings_count as f32
                + inter_height * (visible_postings_count - 1) as f32
        }
    }

    fn row(&self, ui: &mut TableRow) {
        ui.col(|ui| {
            ui.monospace(self.date.clone());
        });
        ui.col(|ui| {
            ui.monospace(self.description.clone());
        });
        ui.col(|ui| {
            ui.vertical(|ui| {
                for (posting, visible) in &self.postings {
                    if *visible {
                        ui.monospace(posting.account_name.to_string());
                    }
                }
            });
        });
        ui.col(|ui| {
            ui.vertical(|ui| {
                for (posting, visible) in &self.postings {
                    if *visible {
                        ui.monospace(&posting.amount);
                    }
                }
            });
        });
    }
}

pub struct TransactionsList {
    loaded_transactions: Promise<Result<Vec<hledger::Transaction>, hledger::Error>>,
    transactions: Option<Vec<Transaction>>,
}

impl TransactionsList {
    pub fn new(manager: &hledger::Manager, file_path: &path::Path) -> Self {
        Self {
            loaded_transactions: Promise::spawn_async({
                let manager = manager.to_owned();
                let file_path = file_path.to_owned();
                async move {
                    let client = manager.client(file_path).await?;
                    client.transactions().await
                }
            }),
            transactions: None,
        }
    }

    pub fn filter_accounts(&mut self, filter: impl Fn(&hledger::AccountName) -> bool) {
        self.transactions = self.transactions.take().map(|mut transactions| {
            transactions.iter_mut().for_each(|transaction| {
                transaction
                    .postings
                    .iter_mut()
                    .for_each(|(posting, visible)| {
                        *visible = filter(&posting.account_name);
                    });
            });
            transactions
        });
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        match self.loaded_transactions.ready() {
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
                let visible_transactions = self
                    .transactions
                    .get_or_insert_with(|| {
                        transactions
                            .iter()
                            .map(Transaction::new)
                            .collect::<Vec<_>>()
                    })
                    .iter()
                    .filter(|transaction| transaction.is_visible())
                    .collect::<Vec<_>>();
                let heights = visible_transactions
                    .iter()
                    .map(|t| t.height(ui))
                    .collect::<Vec<_>>();

                let date_width = 75.0;
                let width_without_date = ui.available_width() - date_width;
                let price_width = width_without_date * 0.25;
                let left_width = width_without_date - price_width;

                TableBuilder::new(ui)
                    .striped(true)
                    .stick_to_bottom(true)
                    .column(Column::exact(date_width))
                    .columns(Column::exact(left_width / 2.0), 2)
                    .column(Column::exact(price_width))
                    .body(|body| {
                        body.heterogeneous_rows(heights.iter().copied(), |row_index, mut row| {
                            visible_transactions[row_index].row(&mut row);
                        });
                    });
            }
        }
    }
}
