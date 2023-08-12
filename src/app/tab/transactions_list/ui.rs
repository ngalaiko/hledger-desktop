/// TODO:
/// - display rolling balance
/// - do not wrap text inside table
/// - wrap account names like a:b:account, similar to how hledger does it
use egui_extras::{Column, TableBuilder};
use tauri_egui::egui::{TextStyle, Ui};

use crate::hledger;

fn height(ui: &Ui, transaction: &hledger::Transaction) -> f32 {
    let row_height = ui.text_style_height(&TextStyle::Monospace);
    let inter_height = ui.spacing().item_spacing.y;
    row_height * transaction.postings.len() as f32
        + inter_height * (transaction.postings.len() - 1) as f32
}

pub fn ui(ui: &mut Ui, transactions: &[hledger::Transaction]) {
    let heights = transactions
        .iter()
        .map(|t| height(ui, t))
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
            body.heterogeneous_rows(heights.into_iter(), |row_index, mut row| {
                let transaction = &transactions[row_index];
                row.col(|ui| {
                    ui.monospace(&transaction.date.to_string());
                });
                row.col(|ui| {
                    ui.monospace(&transaction.description);
                });
                row.col(|ui| {
                    ui.vertical(|ui| {
                        transaction.postings.iter().for_each(|posting| {
                            posting.amount.iter().for_each(|_| {
                                ui.monospace(&posting.account.to_string());
                            });
                        });
                    });
                });
                row.col(|ui| {
                    ui.vertical(|ui| {
                        transaction.postings.iter().for_each(|posting| {
                            posting.amount.iter().for_each(|amount| {
                                ui.monospace(amount.to_string());
                            });
                        });
                    });
                });
            });
        });
}
