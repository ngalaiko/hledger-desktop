use std::sync::Arc;

use eframe::egui::{
    Align, CentralPanel, Layout, RichText, SidePanel, TextEdit, TextStyle, Ui, Vec2,
};
use hledger_query::Transaction;
use smol_macros::Executor;

use crate::{journal, widgets};

pub struct File {
    pub file_path: std::path::PathBuf,
    watcher: journal::Watcher,
    executor: Arc<Executor<'static>>,

    filter_input: String,
    filtered_transactions_tx: Arc<async_watch::Sender<Option<Vec<Transaction>>>>,
    filtered_transactions_rx: async_watch::Receiver<Option<Vec<Transaction>>>,
    filter_task: Option<async_task::Task<()>>,
}

impl File {
    #[must_use]
    pub fn name(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    }

    pub fn new<P: AsRef<std::path::Path>>(executor: Arc<Executor<'static>>, path: P) -> Self {
        let path = path.as_ref();
        let path_clone = path.to_path_buf();
        let (tx, rx) = async_watch::channel(None);
        Self {
            file_path: path.to_path_buf(),
            watcher: journal::Watcher::watch(&executor, path_clone),
            executor,

            filter_input: String::new(),
            filtered_transactions_tx: Arc::new(tx),
            filtered_transactions_rx: rx,
            filter_task: None,
        }
    }
}

pub fn ui(ui: &mut Ui, state: &mut File) {
    SidePanel::left("tabs")
        .resizable(false)
        .default_width(80.0)
        .show(ui.ctx(), |ui| {
            let _ = ui.selectable_label(true, RichText::new("Register").monospace());
        });

    CentralPanel::default().show(ui.ctx(), |ui| {
        let journal_guard = state.watcher.journal();
        let error_guard = state.watcher.error();
        if !error_guard.is_empty() {
            ui.label("error");
        } else if let Some(journal) = journal_guard.as_ref() {
            transactions_filter_ui(ui, state);
            match state.filtered_transactions_rx.borrow().as_ref() {
                Some(transactions) if !state.filter_input.is_empty() => {
                    let refs = transactions.iter().collect::<Vec<_>>();
                    transactions_list_ui(ui, refs.as_slice());
                }
                _ => {
                    transactions_list_ui(ui, journal.transactions().collect::<Vec<_>>().as_slice());
                }
            }
        } else {
            widgets::spinner_ui(ui);
        }
    });
}

fn transactions_filter_ui(ui: &mut Ui, state: &mut File) {
    let input = TextEdit::singleline(&mut state.filter_input)
        .min_size(Vec2 {
            x: ui.available_width(),
            ..Vec2::ZERO
        })
        .hint_text("filter");
    let response = ui.add(input);
    if response.changed() {
        let filter = state.filter_input.clone();
        let watcher = state.watcher.clone();
        let filtered_transactions_tx = state.filtered_transactions_tx.clone();
        state.filter_task.replace(state.executor.spawn(async move {
            let Ok(response) = hledger_query::Query::parse(filter.trim()) else {
                return;
            };
            let journal_guard = watcher.journal();
            let Some(journal) = journal_guard.as_ref() else {
                return;
            };
            let transactons = journal
                .transactions()
                .filter_map(|tx| response.filter(tx))
                .collect::<Vec<_>>();
            filtered_transactions_tx.send(Some(transactons)).unwrap();
        }));
    }
}

fn transactions_list_ui(ui: &mut Ui, transactions: &[&hledger_journal::Transaction]) {
    use egui_extras::{Column, TableBuilder};

    let heights = transactions
        .iter()
        .map(|transaction| transaction_height(ui, transaction) + ui.spacing().item_spacing.x)
        .collect::<Vec<_>>();

    let available_width = ui.available_width();
    TableBuilder::new(ui)
        .striped(false)
        .stick_to_bottom(true)
        .column(Column::exact(available_width))
        .body(|body| {
            body.heterogeneous_rows(heights.into_iter(), |mut row| {
                let transaction = &transactions[row.index()];
                row.col(|ui| transaction_ui(ui, transaction));
            });
        });
}

#[allow(clippy::cast_precision_loss)]
fn transaction_height(ui: &Ui, transaction: &hledger_journal::Transaction) -> f32 {
    let line_height = ui.text_style_height(&TextStyle::Monospace);
    let header = line_height;
    let postings = transaction.postings.len() as f32 * line_height;
    let gaps = if transaction.postings.len() > 1 {
        (transaction.postings.len() - 1) as f32 * ui.spacing().item_spacing.y
    } else {
        0.0
    };
    header + postings + gaps
}

fn transaction_ui(ui: &mut Ui, transaction: &hledger_journal::Transaction) {
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        ui.horizontal(|ui| {
            ui.add_space(ui.spacing().item_spacing.x * 2.0); // this is space for the scroller
            ui.vertical(|ui| {
                let date_widget = ui
                    .horizontal(|ui| {
                        let date_widget = ui.label(
                            RichText::new(transaction.date.format("%Y-%m-%d").to_string())
                                .monospace(),
                        );
                        if let Some(note) = &transaction.note {
                            ui.label(
                                RichText::new(format!("{} | {note}", transaction.payee))
                                    .monospace(),
                            )
                        } else {
                            ui.label(RichText::new(&transaction.payee).monospace())
                        };
                        date_widget
                    })
                    .inner;

                for posting in &transaction.postings {
                    ui.horizontal(|ui| {
                        ui.add_space(date_widget.rect.width() + ui.spacing().item_spacing.x);
                        posting_ui(ui, posting);
                    });
                }
            });
        });
    });
}

fn posting_ui(ui: &mut Ui, posting: &hledger_journal::Posting) {
    ui.columns(2, |columns| {
        columns[0].label(RichText::new(posting.account_name.to_string()).monospace());
        columns[1].with_layout(Layout::right_to_left(Align::Min), |ui| {
            for amount in &posting.amount {
                ui.label(&amount.commodity);
                ui.label(amount.quantity.to_string());
            }
        });
    });
}
