/// TODO:
/// - validate if posting is balanced
use std::{collections::HashSet, path};

use chrono::NaiveDate;
use egui_autocomplete::AutoCompleteTextEdit;
use egui_extras::DatePickerButton;
use egui_modal::Modal as EguiModal;
use poll_promise::Promise;
use tauri_egui::egui::{Align, ComboBox, Layout, TextEdit, Ui, Widget};

use crate::hledger::{self, Amount, Manager};

#[derive(Default)]
pub struct Suggestions {
    descriptions: Vec<String>,
    account_names: Vec<String>,
    destinations: Vec<path::PathBuf>,
}

impl From<&[hledger::Transaction]> for Suggestions {
    fn from(transactions: &[hledger::Transaction]) -> Self {
        let mut destinations = transactions
            .iter()
            .map(|a| a.source_position.0.file_name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        destinations.sort();
        let mut descriptions = transactions
            .iter()
            .map(|a| a.description.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        descriptions.sort();
        let mut account_names = transactions
            .iter()
            .flat_map(|a| a.postings.iter().map(|p| p.account.to_string().clone()))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        account_names.sort();
        Suggestions {
            destinations,
            descriptions,
            account_names,
        }
    }
}

pub struct NewTransactionModal {
    modal: Option<EguiModal>,
    manager: Manager,

    creating: Option<Promise<Result<(), hledger::Error>>>,

    input_date: NaiveDate,
    input_description: String,
    input_postings: Vec<(String, String)>,
    input_destination: Option<path::PathBuf>,
}

impl NewTransactionModal {
    pub fn new(manager: &Manager) -> Self {
        Self {
            creating: None,
            manager: manager.to_owned(),
            modal: None,
            input_date: chrono::offset::Local::now().date_naive(),
            input_description: String::new(),
            input_postings: vec![(String::new(), String::new())],
            input_destination: None,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, suggestions: &Suggestions) -> bool {
        let modal = self.modal.get_or_insert_with(|| {
            EguiModal::new(ui.ctx(), "new tranaction modal").with_close_on_outside_click(true)
        });

        let is_loading = self
            .creating
            .as_ref()
            .map(|p| p.ready().is_none())
            .unwrap_or(false);

        modal.show(|ui| {
            ui.horizontal(|ui| {
                ui.add(DatePickerButton::new(&mut self.input_date).calendar_week(true));

                ui.add(
                    AutoCompleteTextEdit::new(
                        &mut self.input_description,
                        &suggestions.descriptions,
                    )
                    .highlight_matches(true)
                    .set_text_edit_properties(|text_edit| text_edit.hint_text("description")),
                );
            });

            let empty_input_postings = self
                .input_postings
                .iter()
                .filter(|(account_name, amount)| account_name.is_empty() && amount.is_empty())
                .count();

            if empty_input_postings == 0 {
                self.input_postings.push((String::new(), String::new()))
            }

            self.input_postings
                .iter_mut()
                .enumerate()
                .for_each(|(i, (account_name, amount))| {
                    ui.horizontal(|ui| {
                        ui.add(
                            AutoCompleteTextEdit::new(account_name, &suggestions.account_names)
                                .highlight_matches(true)
                                .set_text_edit_properties(move |text_edit| {
                                    text_edit
                                        .hint_text(format!("account {}", i + 1))
                                        .interactive(!is_loading)
                                }),
                        );

                        let is_valid_amount = amount.parse::<Amount>().is_ok();
                        TextEdit::singleline(amount)
                            .interactive(!is_loading)
                            .hint_text(format!("amount {}", i + 1).as_str())
                            .text_color(if is_valid_amount {
                                ui.visuals().widgets.inactive.text_color()
                            } else {
                                ui.style().visuals.error_fg_color
                            })
                            .ui(ui);
                    });
                });

            ui.separator();

            ui.horizontal(|ui| {
                {
                    let mut selected = self
                        .input_destination
                        .as_ref()
                        .unwrap_or(suggestions.destinations.first().unwrap());
                    ComboBox::from_id_source("destination file")
                        .selected_text(selected.file_name().unwrap().to_str().unwrap().to_string())
                        .show_ui(ui, |ui| {
                            for destination in &suggestions.destinations {
                                ui.selectable_value(
                                    &mut selected,
                                    destination,
                                    destination
                                        .file_name()
                                        .unwrap()
                                        .to_str()
                                        .unwrap()
                                        .to_string(),
                                );
                            }
                        });
                    self.input_destination.replace(selected.to_owned());
                }

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if is_loading {
                        ui.spinner();
                    } else if let Some(Ok(())) = self.creating.as_ref().and_then(|p| p.ready()) {
                        modal.close();
                    } else {
                        let button_response = ui.button("add");
                        if button_response.clicked() {
                            self.creating = Some(Promise::spawn_async({
                                let manager = self.manager.clone();
                                let file_path = self.input_destination.as_ref().unwrap().clone();
                                let tx = hledger::Transaction {
                                    date: self.input_date,
                                    description: self.input_description.clone(),
                                    postings: self
                                        .input_postings
                                        .iter()
                                        .filter(|(account_name, amount)| {
                                            !account_name.is_empty() && !amount.is_empty()
                                        })
                                        .map(|(account_name, amount)| hledger::Posting {
                                            account: account_name.parse().unwrap(),
                                            amount: vec![amount.parse().unwrap()],
                                            ..Default::default()
                                        })
                                        .collect(),
                                    ..Default::default()
                                };
                                async move { manager.client(file_path).await?.add(&tx).await }
                            }));
                        }
                    }
                });
            });
        });

        let is_success = self
            .creating
            .as_ref()
            .and_then(|p| p.ready().map(|r| r.is_ok()))
            .unwrap_or(false);

        if !modal.is_open() {
            self.clear();
        }

        is_success
    }

    fn clear(&mut self) {
        self.creating = None;
        self.input_description.clear();
        self.input_postings = vec![(String::new(), String::new())];
    }

    pub fn open(&self) {
        if let Some(ref modal) = self.modal {
            modal.open()
        }
    }
}
