use std::{collections::HashSet, path};

use chrono::NaiveDate;
use egui_extras::DatePickerButton;
use egui_modal::Modal as EguiModal;
use poll_promise::Promise;
use tauri_egui::egui::{Align, Button, ComboBox, Layout, TextEdit, Ui, Widget};

use crate::hledger::{self, Amount, Manager};

struct Suggestion {
    destinations: Vec<path::PathBuf>,
}

pub struct NewTransactionModal {
    modal: Option<EguiModal>,
    manager: Manager,

    suggestion: Promise<Result<Suggestion, hledger::Error>>,
    creating: Option<Promise<Result<(), hledger::Error>>>,

    input_date: NaiveDate,
    input_description: String,
    input_postings: Vec<(String, String)>,
    input_destination: Option<path::PathBuf>,
}

impl NewTransactionModal {
    pub fn new(manager: &Manager, file_path: &path::Path) -> Self {
        Self {
            creating: None,
            suggestion: Promise::spawn_async({
                let manager = manager.to_owned();
                let file_path = file_path.to_owned();
                async move {
                    let client = manager.client(file_path).await?;
                    let mut destinations = client
                        .transactions()
                        .await?
                        .into_iter()
                        .map(|a| a.source_position.0.file_name)
                        .collect::<HashSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>();
                    destinations.sort();
                    Ok(Suggestion { destinations })
                }
            }),
            manager: manager.to_owned(),
            modal: None,
            input_date: chrono::offset::Local::now().date_naive(),
            input_description: String::new(),
            input_postings: Vec::new(),
            input_destination: None,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        match self.suggestion.ready_mut() {
            None => {
                ui.spinner();
            }
            Some(Err(err)) => {
                ui.vertical_centered(|ui| {
                    ui.heading("Failed to load transactions");
                    ui.label(err.to_string());
                });
            }
            Some(Ok(ref mut suggestions)) => {
                let modal = self.modal.get_or_insert_with(|| {
                    EguiModal::new(ui.ctx(), "new tranaction modal")
                        .with_close_on_outside_click(true)
                });

                let is_loading = self
                    .creating
                    .as_ref()
                    .map(|p| p.ready().is_none())
                    .unwrap_or(false);

                modal.show(|ui| {
                    modal.frame(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                DatePickerButton::new(&mut self.input_date)
                                    .calendar_week(true)
                                    .ui(ui);

                                TextEdit::singleline(&mut self.input_description)
                                    .hint_text("description")
                                    .ui(ui);
                            });

                            self.input_postings = self
                                .input_postings
                                .iter_mut()
                                .enumerate()
                                .filter_map(|(i, (account_name, amount))| {
                                    ui.horizontal(|ui| {
                                        TextEdit::singleline(account_name)
                                            .interactive(!is_loading)
                                            .hint_text(format!("account {}", i + 1).as_str())
                                            .ui(ui);

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

                                        if !is_loading {
                                            if Button::new("❌").ui(ui).clicked() {
                                                None
                                            } else {
                                                Some((account_name.clone(), amount.clone()))
                                            }
                                        } else {
                                            Some((account_name.clone(), amount.clone()))
                                        }
                                    })
                                    .inner
                                })
                                .collect::<Vec<_>>();

                            let new_posting_response = ui
                                .horizontal(|ui| {
                                    let mut new_account = String::new();
                                    let mut new_amount = String::new();
                                    let new_account_response =
                                        TextEdit::singleline(&mut new_account)
                                            .interactive(!is_loading)
                                            .hint_text(
                                                format!(
                                                    "account {}",
                                                    self.input_postings.len() + 1
                                                )
                                                .as_str(),
                                            )
                                            .ui(ui);
                                    let new_amount_response = TextEdit::singleline(&mut new_amount)
                                        .interactive(!is_loading)
                                        .hint_text(
                                            format!("amount {}", self.input_postings.len() + 1)
                                                .as_str(),
                                        )
                                        .ui(ui);

                                    if new_account_response.union(new_amount_response).changed() {
                                        Some((new_account.clone(), new_amount))
                                    } else {
                                        None
                                    }
                                })
                                .inner;

                            if let Some(posting) = new_posting_response {
                                self.input_postings.push(posting);
                            }

                            if self.input_destination.is_none() {
                                self.input_destination = suggestions.destinations.first().cloned();
                            }
                            let mut selected = self.input_destination.as_ref().unwrap();
                            ComboBox::from_id_source("destination file")
                                .selected_text(format!(
                                    "{}",
                                    self.input_destination.as_ref().unwrap().display()
                                ))
                                .show_ui(ui, |ui| {
                                    for destination in &suggestions.destinations {
                                        ui.selectable_value(
                                            &mut selected,
                                            destination,
                                            format!("{}", destination.display()),
                                        );
                                    }
                                });
                            self.input_destination.replace(selected.to_owned());
                        });
                    });

                    ui.separator();

                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        if is_loading {
                            ui.spinner();
                        } else if let Some(Ok(())) = self.creating.as_ref().and_then(|p| p.ready())
                        {
                            modal.close();
                        } else {
                            let button_response = ui.button("add");
                            if button_response.clicked() {
                                self.creating = Some(Promise::spawn_async({
                                    let manager = self.manager.clone();
                                    let file_path =
                                        self.input_destination.as_ref().unwrap().clone();
                                    let tx = hledger::Transaction {
                                        date: self.input_date,
                                        description: self.input_description.clone(),
                                        postings: self
                                            .input_postings
                                            .iter()
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

                if !modal.is_open() {
                    self.clear();
                }
            }
        }
    }

    fn clear(&mut self) {
        self.input_description.clear();
        self.input_postings.clear();
    }

    pub fn open(&self) {
        if let Some(ref modal) = self.modal {
            modal.open()
        }
    }
}
