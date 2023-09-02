/// TODO:
/// - validate if posting is balanced
use std::{collections::HashSet, path};

use chrono::NaiveDate;
use egui_autocomplete::AutoCompleteTextEdit;
use egui_extras::DatePickerButton;
use egui_modal::Modal as EguiModal;
use poll_promise::Promise;
use tauri_egui::egui::{Align, Button, ComboBox, Label, Layout, RichText, TextEdit, Ui};

use crate::hledger::{self, AccountName, Amount, Manager, ParseAccountNameError, ParseAmountError};

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
    input_postings: PostingsInput,
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
            input_postings: PostingsInput::new(),
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

            self.input_postings.ui(ui, is_loading, suggestions);

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
                        match self.input_postings.value() {
                            Err(PostingError::InvalidPostings) => {
                                ui.add_enabled(false, Button::new("add"));
                            },
                            Ok(postings) => {
                                if ui.button("add").clicked() {
                                    self.creating = Some(Promise::spawn_async({
                                        let manager = self.manager.clone();
                                        let file_path = self.input_destination.as_ref().unwrap().clone();
                                        let tx = hledger::Transaction {
                                            date: self.input_date,
                                            description: self.input_description.clone(),
                                            postings,
                                            ..Default::default()
                                        };
                                        async move { manager.client(file_path).await?.add(&tx).await }
                                    }));
                                }
                            }
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
        self.input_postings.clear();
    }

    pub fn open(&self) {
        if let Some(ref modal) = self.modal {
            modal.open()
        }
    }
}

struct AmountInput {
    input_text: String,
    parsed: Result<Amount, ParseAmountError>,
}

impl AmountInput {
    pub fn new() -> Self {
        Self {
            input_text: String::new(),
            parsed: Ok(Amount::default()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.input_text.is_empty()
    }

    pub fn value(&self) -> Result<Amount, ParseAmountError> {
        self.parsed.clone()
    }

    pub fn ui(&mut self, ui: &mut Ui, interactive: bool, hint: &str) {
        if ui
            .add(
                TextEdit::singleline(&mut self.input_text)
                    .interactive(interactive)
                    .hint_text(hint)
                    .text_color(if self.parsed.is_ok() {
                        ui.visuals().widgets.inactive.text_color()
                    } else {
                        ui.style().visuals.error_fg_color
                    }),
            )
            .changed()
        {
            self.parsed = if self.input_text.is_empty() {
                Ok(Amount::default())
            } else {
                self.input_text.parse()
            };
        }
    }
}

struct AccountInput {
    input_text: String,
    parsed: Result<AccountName, ParseAccountNameError>,
}

impl AccountInput {
    pub fn new() -> Self {
        Self {
            input_text: String::new(),
            parsed: Ok(AccountName::default()),
        }
    }

    pub fn value(&self) -> Result<AccountName, ParseAccountNameError> {
        self.parsed.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.input_text.is_empty()
    }

    pub fn ui(&mut self, ui: &mut Ui, interactive: bool, hint: &str, suggestions: &[String]) {
        let hint = hint.to_owned();
        ui.add(
            AutoCompleteTextEdit::new(&mut self.input_text, suggestions)
                .highlight_matches(true)
                .max_suggestions(5)
                .set_text_edit_properties(move |text_edit| {
                    text_edit.hint_text(hint).interactive(interactive)
                }),
        );

        self.parsed = if self.input_text.is_empty() {
            Ok(AccountName::default())
        } else {
            self.input_text.parse()
        };
    }
}

struct PostingsInput {
    input_postings: Vec<(AccountInput, AmountInput)>,
}

#[derive(Debug, thiserror::Error)]
enum PostingError {
    #[error("one or more postings are invalid")]
    InvalidPostings,
}

impl PostingsInput {
    pub fn new() -> Self {
        Self {
            input_postings: vec![(AccountInput::new(), AmountInput::new())],
        }
    }

    pub fn value(&self) -> Result<Vec<hledger::Posting>, PostingError> {
        if self
            .input_postings
            .iter()
            .filter(|(account_name, amount)| !account_name.is_empty() || !amount.is_empty())
            .any(|(account_name, amount)| account_name.value().is_err() || amount.value().is_err())
        {
            return Err(PostingError::InvalidPostings);
        };

        let postings = self
            .input_postings
            .iter()
            .filter(|(account_name, amount)| !account_name.is_empty() && !amount.is_empty())
            .map(|(account_name, amount)| hledger::Posting {
                account: account_name.value().unwrap(),
                amount: vec![amount.value().unwrap()],
                ..Default::default()
            })
            .collect();

        Ok(postings)
    }

    pub fn clear(&mut self) {
        *self = Self::new()
    }

    pub fn ui(&mut self, ui: &mut Ui, is_loading: bool, suggestions: &Suggestions) {
        let empty_input_postings = self
            .input_postings
            .iter()
            .filter(|(account_name, amount)| account_name.is_empty() && amount.is_empty())
            .count();

        if empty_input_postings == 0 {
            self.input_postings
                .push((AccountInput::new(), AmountInput::new()))
        }

        self.input_postings
            .iter_mut()
            .enumerate()
            .for_each(|(i, (account_name, amount))| {
                ui.horizontal(|ui| {
                    account_name.ui(
                        ui,
                        !is_loading,
                        &format!("account {}", i + 1),
                        &suggestions.account_names,
                    );
                    amount.ui(ui, !is_loading, &format!("amount {}", i + 1));
                });

                if let Err(error) = amount.value() {
                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        ui.add(Label::new(
                            RichText::new(format!("invalid amount: {}", error))
                                .small()
                                .color(ui.style().visuals.error_fg_color),
                        ));
                    });
                }
            });
    }
}
