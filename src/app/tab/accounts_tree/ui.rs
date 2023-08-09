use std::{collections::HashSet, path};

use poll_promise::Promise;
use tauri_egui::egui::{collapsing_header, Align, Id, Label, Layout, Response, ScrollArea, Ui};

use crate::{
    hledger,
    widgets::{checkbox, CheckboxState},
};

use super::State;

pub struct AccountsTree {
    loading_accounts: Promise<Result<Vec<hledger::Account>, hledger::Error>>,
    trees: Option<Vec<Tree>>,
    state: State,
}

impl AccountsTree {
    pub fn new(manager: &hledger::Manager, file_path: &path::Path, state: &State) -> Self {
        Self {
            loading_accounts: Promise::spawn_async({
                let manager = manager.to_owned();
                let file_path = file_path.to_owned();
                async move {
                    let client = manager.client(file_path).await?;
                    client.accounts().await
                }
            }),
            trees: None,
            state: state.to_owned(),
        }
    }

    fn open_accounts(&self) -> HashSet<hledger::AccountName> {
        self.trees
            .as_ref()
            .map(|trees| {
                trees.iter().map(|tree| tree.open_accounts()).fold(
                    HashSet::new(),
                    |mut acc, open_accounts| {
                        acc.extend(open_accounts);
                        acc
                    },
                )
            })
            .unwrap_or_default()
    }

    fn checked_accounts(&self) -> HashSet<hledger::AccountName> {
        self.trees
            .as_ref()
            .map(|trees| {
                trees.iter().map(|tree| tree.checked_accounts()).fold(
                    HashSet::new(),
                    |mut acc, checked_accounts| {
                        acc.extend(checked_accounts);
                        acc
                    },
                )
            })
            .unwrap_or_default()
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        match self.loading_accounts.ready() {
            None => {
                ui.vertical_centered(|ui| {
                    ui.ctx().request_repaint();
                    ui.spinner()
                })
                .inner
            }
            Some(Err(err)) => {
                ui.vertical_centered(|ui| {
                    ui.heading("Failed to load accounts");
                    ui.label(err.to_string())
                })
                .inner
            }
            Some(Ok(accounts)) => {
                let trees = self.trees.get_or_insert_with(|| {
                    accounts
                        .iter()
                        .filter(|a| a.name.depth() == 1)
                        .filter(|a| a.name.to_string() != "root")
                        .map(|account| {
                            Tree::new(account, accounts, &self.state.open, &self.state.checked)
                        })
                        .collect::<Vec<Tree>>()
                });

                let response = ScrollArea::vertical()
                    .drag_to_scroll(false)
                    .show(ui, |ui| {
                        let mut response = trees[0].ui(ui);
                        trees
                            .iter_mut()
                            .skip(1)
                            .for_each(|tree| response = response.union(tree.ui(ui)));
                        response
                    })
                    .inner;

                if response.changed() {
                    self.state.open = self.open_accounts();
                    self.state.checked = self.checked_accounts();
                }

                response
            }
        }
    }
}

struct Tree {
    account_name: hledger::AccountName,
    is_open: bool,
    checkbox: CheckboxState,
    children: Vec<Tree>,
}

impl Tree {
    pub fn new(
        root: &hledger::Account,
        all_accounts: &[hledger::Account],
        open: &HashSet<hledger::AccountName>,
        checked: &HashSet<hledger::AccountName>,
    ) -> Self {
        let children = all_accounts
            .iter()
            .filter(|a| a.parent.eq(&root.name))
            .map(|account| Self::new(account, all_accounts, open, checked))
            .collect::<Vec<Tree>>();

        let checkbox = if checked.contains(&root.name) {
            CheckboxState::Checked
        } else if children
            .iter()
            .all(|child| matches!(child.checkbox, CheckboxState::Unchecked))
        {
            CheckboxState::Unchecked
        } else {
            CheckboxState::Indeterminate
        };

        Self {
            account_name: root.name.clone(),
            is_open: open.contains(&root.name),
            checkbox,
            children,
        }
    }

    fn set_checkbox_state(&mut self, state: &CheckboxState) {
        self.checkbox = *state;
        if *state != CheckboxState::Indeterminate {
            self.children.iter_mut().for_each(|child| {
                child.set_checkbox_state(state);
            });
        }
    }

    fn set_is_open(&mut self, is_open: bool) {
        self.is_open = is_open;
        if !is_open {
            self.children.iter_mut().for_each(|child| {
                child.set_is_open(is_open);
            });
        }
    }

    pub fn open_accounts(&self) -> HashSet<hledger::AccountName> {
        if self.children.is_empty() {
            HashSet::new()
        } else {
            let mut result = HashSet::new();
            if self.is_open {
                result.insert(self.account_name.clone());
            }
            self.children.iter().for_each(|child| {
                result.extend(child.open_accounts());
            });
            result
        }
    }

    pub fn checked_accounts(&self) -> HashSet<hledger::AccountName> {
        let mut result = HashSet::new();
        if matches!(self.checkbox, CheckboxState::Checked) {
            result.insert(self.account_name.clone());
        }
        self.children.iter().for_each(|child| {
            result.extend(child.checked_accounts());
        });
        result
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Response {
        if self.children.is_empty() {
            // if it's a leaf, just show the checkbox, and return checkbox's response
            ui.horizontal(|ui| {
                ui.add(Label::new(self.account_name.basename()).wrap(false));

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    let checkbox_response = checkbox(ui, &mut self.checkbox);
                    if checkbox_response.changed() {
                        self.set_checkbox_state(&self.checkbox.clone());
                    }

                    checkbox_response
                })
                .inner
            })
            .inner
        } else {
            let mut header = collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                Id::new(self.account_name.to_string()),
                self.is_open,
            );
            header.set_open(self.is_open);

            let (mut collapsing_button_response, header_response, body_response) = header
                .show_header(ui, |ui| {
                    // for the header, draw the checkbox and the account name.
                    // return checkbox response to propagate changed hook.
                    ui.horizontal(|ui| {
                        ui.label(self.account_name.basename());

                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            let checkbox_response = checkbox(ui, &mut self.checkbox);
                            if checkbox_response.changed() {
                                self.set_checkbox_state(&self.checkbox.clone());
                            }

                            checkbox_response
                        })
                        .inner
                    })
                    .inner
                })
                .body(|ui| {
                    // render all children and return the union of all responses
                    // so if any of the children has changed, the resulting response will be marked as changed also
                    let (head, tail) = self.children.split_first_mut().unwrap();
                    let mut response = head.ui(ui);
                    for child in tail {
                        response = response.union(child.ui(ui))
                    }

                    if response.changed() {
                        let all_checked = self
                            .children
                            .iter()
                            .all(|child| child.checkbox == CheckboxState::Checked);
                        let all_unchecked = self
                            .children
                            .iter()
                            .all(|child| child.checkbox == CheckboxState::Unchecked);

                        let state = if all_unchecked {
                            CheckboxState::Unchecked
                        } else if all_checked {
                            CheckboxState::Checked
                        } else {
                            CheckboxState::Indeterminate
                        };

                        if self.checkbox != state {
                            self.checkbox = state;
                            ui.ctx().request_repaint();
                        }
                    }

                    response
                });

            if collapsing_button_response.clicked() {
                self.set_is_open(!self.is_open);
                collapsing_button_response.mark_changed();
            }

            // finally, return union of header change and body change
            if let Some(body_response) = body_response {
                body_response
                    .inner
                    .union(header_response.inner)
                    .union(collapsing_button_response)
            } else {
                header_response.inner.union(collapsing_button_response)
            }
        }
    }
}
