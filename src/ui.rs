use std::path;

use egui_autocomplete::AutoCompleteTextEdit;
use egui_extras::{Column, DatePickerButton, TableBuilder};
use egui_modal::Modal;
use tauri_egui::egui::{
    collapsing_header, Align, Button, CentralPanel, ComboBox, Context, Hyperlink, Id, Label,
    Layout, RichText, ScrollArea, SidePanel, TextEdit, TextStyle, TopBottomPanel, Ui, Visuals,
};

use crate::{
    hledger,
    state::{
        tab::{
            new_transaction::{
                Error as NewTransactionStateError, Update as NewTransactionStateUpdate,
            },
            AccountTreeNode, State as TabState, Update as TabStateUpdate,
        },
        RenderMode, State, Theme, Update as StateUpdate,
    },
    widgets::{checkbox, CheckboxState},
};

pub fn show(ctx: &Context, state: &State) -> Vec<StateUpdate> {
    let mut updates = vec![];

    TopBottomPanel::top("top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            updates.extend(tabs_list(ui, state));

            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if let Some(update) = dark_light_mode_switch_ui(ui, state) {
                    updates.push(update);
                }
                ui.separator();
            });
        })
    });

    TopBottomPanel::bottom("botttom_bar").show(ctx, |ui| {
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            if let Some(update) = render_mode_ui(ui, state) {
                updates.push(update)
            }
            frames_per_second_ui(ui, state);
        });
    });

    CentralPanel::default().show(ctx, |ui| {
        if let Some(active_tab_index) = state.active_tab_index() {
            let active_tab = state
                .tabs()
                .get(active_tab_index)
                .expect("active tab index is valid");
            for update in tab_ui(ui, active_tab) {
                updates.push(StateUpdate::update_tab(active_tab_index, update));
            }
        } else {
            updates.extend(welcome_screen_ui(ui));
        }
    });

    updates
}

fn account_tree_node_ui(ui: &mut Ui, node: &AccountTreeNode) -> Vec<TabStateUpdate> {
    let mut updates = vec![];
    if node.children().is_empty() {
        ui.horizontal(|ui| {
            ui.add(Label::new(node.name().basename()).wrap(false));

            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                let mut checkbox_state = *node.checkbox_state();

                if checkbox(ui, &mut checkbox_state).changed() {
                    match checkbox_state {
                        CheckboxState::Checked => updates.push(TabStateUpdate::check_account(node)),
                        CheckboxState::Unchecked => {
                            updates.push(TabStateUpdate::uncheck_account(node))
                        }
                        CheckboxState::Indeterminate => {}
                    }
                }
            });
        });
    } else {
        let is_open = *node.is_expanded();

        let mut header = collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            Id::new(node.name().to_string()),
            is_open,
        );
        header.set_open(is_open);

        let (collapsing_button_response, _header_response, _body_response) = header
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(node.name().basename());

                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                        let mut checkbox_state = *node.checkbox_state();

                        if checkbox(ui, &mut checkbox_state).changed() {
                            match checkbox_state {
                                CheckboxState::Checked => {
                                    updates.push(TabStateUpdate::check_account(node))
                                }
                                CheckboxState::Unchecked => {
                                    updates.push(TabStateUpdate::uncheck_account(node))
                                }
                                CheckboxState::Indeterminate => {
                                    updates.push(TabStateUpdate::uncheck_account(node))
                                }
                            }
                        }
                    })
                })
            })
            .body(|ui| {
                for child in node.children() {
                    updates.extend(account_tree_node_ui(ui, child));
                }
            });

        if collapsing_button_response.clicked() {
            if is_open {
                updates.push(TabStateUpdate::collapse_account(node.name()))
            } else {
                updates.push(TabStateUpdate::expand_account(node.name()))
            }
        }
    }

    updates
}

fn tab_ui(ui: &mut Ui, tab_state: &TabState) -> Vec<TabStateUpdate> {
    match (
        tab_state.accounts_tree(),
        tab_state.transactions(),
        tab_state.commodities(),
    ) {
        (Some(account_trees), Some(transactions), Some(commodities)) => {
            match (
                account_trees.ready(),
                transactions.ready(),
                commodities.ready(),
            ) {
                (Some(Err(hledger::Error::Process(hledger::ProcessError::NotFound))), _, _)
                | (_, Some(Err(hledger::Error::Process(hledger::ProcessError::NotFound))), _)
                | (_, _, Some(Err(hledger::Error::Process(hledger::ProcessError::NotFound)))) => {
                    ui.vertical_centered(|ui| {
                        ui.heading("Hledger is not installed");
                        ui.label("Follow official installation instructions to continue:");
                        ui.add(Hyperlink::from_label_and_url(
                            "Hledger documentation",
                            "https://hledger.org/install.html",
                        ))
                    });
                    vec![]
                }
                (account_trees, transactions, commodities) => {
                    let mut updates = vec![];

                    updates.extend(new_transaction_modal_ui(ui, tab_state));

                    SidePanel::left("accounts_tree").show(ui.ctx(), |ui| {
                        ScrollArea::vertical().drag_to_scroll(false).show(ui, |ui| {
                            match account_trees {
                                Some(Ok(root_node)) => {
                                    for top_level_node in root_node.children() {
                                        updates.extend(account_tree_node_ui(ui, top_level_node));
                                    }
                                }
                                None => loading_ui(ui),
                                Some(Err(error)) => {
                                    ui.vertical_centered(|ui| {
                                        ui.heading("Failed to load accounts");
                                        ui.label(error.to_string())
                                    });
                                }
                            }
                        });
                    });

                    CentralPanel::default().show(ui.ctx(), |ui| {
                        match (transactions, commodities) {
                            (Some(Ok(transactions)), Some(Ok(commodities))) => {
                                TopBottomPanel::top("transactions top bar").show(ui.ctx(), |ui| {
                                    ui.horizontal(|ui| {
                                        if let Some(update) =
                                            display_commodity_ui(ui, tab_state, commodities)
                                        {
                                            updates.push(update);
                                        }
                                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                            if ui
                                                .button(egui_phosphor::regular::PLUS)
                                                .on_hover_text("Create new transaction")
                                                .clicked()
                                            {
                                                updates.push(
                                                    TabStateUpdate::open_new_transaction_modal(),
                                                );
                                            }
                                        });
                                    });
                                });

                                CentralPanel::default().show(ui.ctx(), |ui| {
                                    transactions_ui(ui, transactions);
                                });
                            }
                            (None, _) | (_, None) => loading_ui(ui),
                            (Some(Err(error)), _) | (_, Some(Err(error))) => {
                                ui.vertical_centered(|ui| {
                                    ui.heading("Failed to load transactions");
                                    ui.label(error.to_string())
                                });
                            }
                        }
                    });
                    updates
                }
            }
        }
        _ => {
            vec![
                TabStateUpdate::load_account_trees(),
                TabStateUpdate::load_transactions(),
                TabStateUpdate::load_commodities(),
            ]
        }
    }
}

fn display_commodity_ui(
    ui: &mut Ui,
    tab_state: &TabState,
    commodities: &[hledger::Commodity],
) -> Option<TabStateUpdate> {
    let mut update = None;

    ui.horizontal(|ui| {
        let mut display_commodity = tab_state
            .display_commodity()
            .map(|c| c.to_string())
            .unwrap_or("-".to_string());

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
                        update.replace(TabStateUpdate::set_display_commodity(Some(
                            display_commodity.clone(),
                        )));
                    }
                }
            });

        if tab_state.display_commodity().is_some() && ui.button(egui_phosphor::regular::X).clicked()
        {
            update.replace(TabStateUpdate::set_display_commodity(None));
        }
    });

    update
}

fn new_transaction_modal_ui(ui: &mut Ui, tab_state: &TabState) -> Vec<TabStateUpdate> {
    tab_state
        .new_transaction_modal_state()
        .map(|state| {
            let mut updates = vec![];
            let new_transaction_modal =
                Modal::new(ui.ctx(), "new tranaction modal").with_close_on_outside_click(true);

            new_transaction_modal.show(|ui| {
                ui.vertical(|ui| {
                    let suggestions = state.suggestions();
                    ui.horizontal(|ui| {
                        let mut date = *state.date();
                        if ui
                            .add(DatePickerButton::new(&mut date).calendar_week(true))
                            .changed()
                        {
                            updates.push(NewTransactionStateUpdate::set_date(&date).into())
                        }

                        let mut description = state.description().to_string();
                        ui.add(
                            AutoCompleteTextEdit::new(&mut description, &suggestions.descriptions)
                                .highlight_matches(true)
                                .set_text_edit_properties(|text_edit| {
                                    text_edit.hint_text("description")
                                }),
                        );
                        updates
                            .push(NewTransactionStateUpdate::set_description(&description).into());
                    });

                    let is_loading = state.is_loading();
                    for (i, posting) in state.postings().iter().enumerate() {
                        ui.horizontal(|ui| {
                            let mut account = posting.account().to_string();
                            ui.add(
                                AutoCompleteTextEdit::new(&mut account, &suggestions.account_names)
                                    .highlight_matches(true)
                                    .max_suggestions(5)
                                    .set_text_edit_properties(move |text_edit| {
                                        text_edit
                                            .hint_text(format!("account {}", i + 1))
                                            .interactive(!is_loading)
                                    }),
                            );
                            updates.push(
                                NewTransactionStateUpdate::set_posting_account(i, &account).into(),
                            );

                            let mut amount = posting.amount().to_string();
                            if ui
                                .add(
                                    TextEdit::singleline(&mut amount)
                                        .interactive(!is_loading)
                                        .hint_text(format!("amount {}", i + 1))
                                        .text_color(if posting.parsed_amount().is_ok() {
                                            ui.visuals().widgets.inactive.text_color()
                                        } else {
                                            ui.style().visuals.error_fg_color
                                        }),
                                )
                                .changed()
                            {
                                updates.push(
                                    NewTransactionStateUpdate::set_posting_amount(i, &amount)
                                        .into(),
                                );
                            }
                        });
                    }

                    ui.horizontal(|ui| {
                        let mut selected_destination = state.destination().clone();
                        ComboBox::from_id_source("destination file")
                            .selected_text(
                                selected_destination
                                    .file_name()
                                    .unwrap()
                                    .to_str()
                                    .unwrap()
                                    .to_string(),
                            )
                            .show_ui(ui, |ui| {
                                for destination in &suggestions.destinations {
                                    if ui
                                        .selectable_value(
                                            &mut selected_destination,
                                            destination,
                                            destination
                                                .file_name()
                                                .unwrap()
                                                .to_str()
                                                .unwrap()
                                                .to_string(),
                                        )
                                        .clicked()
                                    {
                                        updates.push(
                                            NewTransactionStateUpdate::set_destination(
                                                selected_destination,
                                            )
                                            .into(),
                                        );
                                    }
                                }
                            });

                        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                            if is_loading {
                                ui.spinner();
                            } else if let Some(Ok(())) = state.result() {
                                updates.extend(vec![
                                    TabStateUpdate::close_new_transaction_modal(),
                                    TabStateUpdate::reload_transactions(),
                                ]);
                            } else {
                                match state.parsed_postings() {
                                    Err(NewTransactionStateError::InvalidPostings) => {
                                        ui.add_enabled(false, Button::new("add"));
                                    }
                                    Ok(postings) => {
                                        if ui.button("add").clicked() {
                                            let tx = hledger::Transaction {
                                                date: *state.date(),
                                                description: state.description().to_string(),
                                                postings: postings.to_vec(),
                                                ..Default::default()
                                            };
                                            updates.push(
                                                NewTransactionStateUpdate::create_transaction(
                                                    state.destination(),
                                                    &tx,
                                                )
                                                .into(),
                                            );
                                        }
                                    }
                                }
                            }
                        });
                    });
                });
            });
            new_transaction_modal.open();
            if new_transaction_modal.was_outside_clicked() {
                updates.push(TabStateUpdate::close_new_transaction_modal());
            }

            updates
        })
        .unwrap_or_default()
}

fn welcome_screen_ui(ui: &mut Ui) -> Vec<StateUpdate> {
    let mut updates = vec![];
    ui.vertical_centered(|ui| {
        ui.heading("Welcome to hledger-desktop");
        if ui.button("Open a new file...").clicked() {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                updates.push(StateUpdate::create_tab(file_path.to_path_buf()));
            }
        }

        let default_file = std::env::var("LEDGER_FILE").map(path::PathBuf::from).ok();
        if let Some(default_file) = default_file {
            let default_file_name = default_file.file_name().unwrap().to_str().unwrap();
            if ui.button(format!("Open {}", default_file_name)).clicked() {
                updates.push(StateUpdate::create_tab(default_file.to_path_buf()));
            }
        }
    });
    updates
}

fn tabs_list(ui: &mut Ui, state: &State) -> Vec<StateUpdate> {
    let mut updates = vec![];

    ui.horizontal(|ui| {
        let mut new_selected = None;
        let mut deleted = vec![];

        for (tab_index, tab) in state.tabs().iter().enumerate() {
            let is_selected = state.active_tab_index() == Some(tab_index);
            if ui
                .selectable_label(is_selected, tab.name())
                .context_menu(|ui| {
                    if ui.button("Close").clicked() {
                        deleted.push(tab_index);
                        ui.close_menu();
                    }
                })
                .clicked()
            {
                new_selected.replace(tab_index);
            }
        }

        if !state.tabs().is_empty()
            && ui
                .button(egui_phosphor::regular::PLUS)
                .on_hover_text("Open new file")
                .clicked()
        {
            if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                updates.push(StateUpdate::create_tab(file_path.to_path_buf()));
            }
        }

        if let Some(index) = new_selected {
            updates.push(StateUpdate::set_active_tab_index(index));
        }

        deleted.drain(..).for_each(|index| {
            updates.push(StateUpdate::delete_tab(index));
        });
    });

    updates
}

fn dark_light_mode_switch_ui(ui: &mut Ui, state: &State) -> Option<StateUpdate> {
    let new_theme = match state.theme() {
        Theme::Light => {
            if ui
                .add(Button::new(egui_phosphor::regular::MOON).frame(false))
                .on_hover_text("Switch to dark mode")
                .clicked()
            {
                Some(Theme::Dark)
            } else {
                None
            }
        }
        Theme::Dark => {
            if ui
                .add(Button::new(egui_phosphor::regular::SUN).frame(false))
                .on_hover_text("Switch to light mode")
                .clicked()
            {
                Some(Theme::Light)
            } else {
                None
            }
        }
    };

    if let Some(theme) = new_theme {
        ui.ctx().set_visuals(theme.into());
        Some(StateUpdate::set_theme(&theme))
    } else {
        None
    }
}

fn render_mode_ui(ui: &mut Ui, state: &State) -> Option<StateUpdate> {
    match state.render_mode() {
        RenderMode::Continious => {
            ui.ctx().request_repaint();
            if ui
                .add(Button::new(egui_phosphor::regular::PLAY).frame(false))
                .on_hover_text("Switch to reactive rendering")
                .clicked()
            {
                Some(StateUpdate::set_render_mode(&RenderMode::Reactive))
            } else {
                None
            }
        }
        RenderMode::Reactive => {
            if ui
                .add(Button::new(egui_phosphor::regular::PAUSE).frame(false))
                .on_hover_text("Switch to continious rendering")
                .clicked()
            {
                Some(StateUpdate::set_render_mode(&RenderMode::Continious))
            } else {
                None
            }
        }
    }
}

fn frames_per_second_ui(ui: &mut Ui, state: &State) {
    ui.label(format!(
        "FPS: {:.1} ({:.2} ms / frame)",
        state.frames_per_second(),
        1e3 * state.frame_mean_time(),
    ))
    .on_hover_text(
        "Includes egui layout and tessellation time.\n\
            Does not include GPU usage, nor overhead for sending data to GPU.",
    );
}

impl From<&Theme> for Visuals {
    fn from(value: &Theme) -> Self {
        match value {
            Theme::Light => Visuals::light(),
            Theme::Dark => Visuals::dark(),
        }
    }
}

impl From<Theme> for Visuals {
    fn from(value: Theme) -> Self {
        Self::from(&value)
    }
}

fn loading_ui(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.ctx().request_repaint();
        ui.spinner()
    });
}

fn transaction_height(ui: &Ui, transaction: &hledger::Transaction) -> f32 {
    let row_height = ui.text_style_height(&TextStyle::Monospace);
    let inter_height = ui.spacing().item_spacing.y;
    row_height * transaction.postings.len() as f32
        + inter_height * (transaction.postings.len() - 1) as f32
}

fn transactions_ui(ui: &mut Ui, transactions: &[hledger::Transaction]) {
    // TODO:
    // - display rolling balance
    // - wrap account names like a:b:account, similar to how hledger does it
    let heights = transactions
        .iter()
        .map(|tx| transaction_height(ui, tx))
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
                    ui.add(
                        Label::new(RichText::new(transaction.date.to_string()).monospace())
                            .wrap(false),
                    );
                });
                row.col(|ui| {
                    ui.add(
                        Label::new(RichText::new(transaction.description.to_string()).monospace())
                            .wrap(false),
                    );
                });
                row.col(|ui| {
                    ui.vertical(|ui| {
                        transaction.postings.iter().for_each(|posting| {
                            posting.amount.iter().for_each(|_| {
                                ui.add(
                                    Label::new(
                                        RichText::new(posting.account.to_string()).monospace(),
                                    )
                                    .wrap(false),
                                );
                            });
                        });
                    });
                });
                row.col(|ui| {
                    ui.vertical(|ui| {
                        transaction.postings.iter().for_each(|posting| {
                            posting.amount.iter().for_each(|amount| {
                                ui.add(
                                    Label::new(RichText::new(amount.to_string()).monospace())
                                        .wrap(false),
                                );
                            });
                        });
                    });
                });
            });
        });
}
