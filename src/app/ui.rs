use std::{collections::HashMap, path};

use tauri::AppHandle;
use tauri_egui::{
    eframe::CreationContext,
    egui::{Align, CentralPanel, Context, Layout, TopBottomPanel, Ui},
};

use crate::{hledger, widgets};

use super::{state::State, tab};

pub struct App {
    handle: AppHandle,

    state: State,
    manager: hledger::Manager,
    frame_history: widgets::FramesPerSecond,
    tabs: HashMap<path::PathBuf, tab::Tab>,
}

impl App {
    pub fn new(cc: &CreationContext<'_>, handle: AppHandle) -> Self {
        let state = State::load(&handle);
        cc.egui_ctx.set_visuals(state.theme.into());
        Self {
            state,
            manager: hledger::Manager::new(&handle),
            handle,
            frame_history: widgets::FramesPerSecond::default(),
            tabs: HashMap::new(),
        }
    }

    fn open_new_tab(&mut self, file_path: path::PathBuf) {
        self.state.tabs.push((file_path, tab::State::default()));
        self.state.selected_tab = Some(self.state.tabs.len() - 1);
        self.state.save(&self.handle);
    }

    fn tabs_list(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let mut to_delete = None;
            for (i, (file_path, _state)) in self.state.tabs.iter().enumerate() {
                ui.horizontal(|ui| {
                    let is_selected = self.state.selected_tab.as_ref() == Some(&i);
                    if ui
                        .selectable_label(
                            is_selected,
                            file_path.file_name().unwrap().to_str().unwrap(),
                        )
                        .context_menu(|ui| {
                            if ui.button("Close").clicked() {
                                to_delete.replace(i);
                                ui.close_menu();
                            }
                        })
                        .clicked()
                    {
                        self.state.selected_tab.replace(i);
                    }
                });
            }

            if let Some(i) = to_delete {
                let (removed_file_name, _removed_state) = self.state.tabs.remove(i);
                self.tabs.remove(&removed_file_name);
                if self.state.tabs.is_empty() {
                    self.state.selected_tab.take();
                } else if self.state.selected_tab.as_ref() == Some(&i) {
                    self.state.selected_tab.replace(0);
                } else if self.state.selected_tab.as_ref().unwrap() > &i {
                    self.state
                        .selected_tab
                        .replace(self.state.selected_tab.unwrap() - 1);
                    self.state.save(&self.handle);
                }
            }

            if ui.button("+").clicked() {
                if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                    self.open_new_tab(file_path);
                }
            }
        });
    }

    fn welcome_screen(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Welcome to hledger-desktop");
            if ui.button("Open a new file...").clicked() {
                if let Some(file_path) = rfd::FileDialog::new().pick_file() {
                    self.open_new_tab(file_path);
                }
            }

            let default_file = std::env::var("LEDGER_FILE").map(path::PathBuf::from).ok();
            if let Some(default_file) = default_file {
                let default_file_name = default_file.file_name().unwrap().to_str().unwrap();
                if ui.button(format!("Open {}", default_file_name)).clicked() {
                    self.open_new_tab(default_file);
                }
            }
        });
    }
}

impl tauri_egui::eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut tauri_egui::eframe::Frame) {
        // update the frame history
        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);

        // enter the async runtime Contex to make promises work
        let _handle = tauri::async_runtime::handle().inner().enter();

        TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.tabs_list(ui);

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if widgets::dark_light_mode_switch(ui, &mut self.state.theme).clicked() {
                        self.state.save(&self.handle);
                    }
                    ui.separator();
                });
            })
        });

        TopBottomPanel::bottom("botttom_bar").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                self.frame_history.ui(ui);
            });
        });

        if let Some(selected_tab_index) = self.state.selected_tab {
            let (selected_tab_file_path, selected_tab_state) =
                &mut self.state.tabs[selected_tab_index];
            let selected_tab = self
                .tabs
                .entry(selected_tab_file_path.clone())
                .or_insert_with(|| {
                    tab::Tab::new(
                        &self.manager,
                        selected_tab_file_path.clone(),
                        selected_tab_state.clone(),
                    )
                });

            CentralPanel::default().show(ctx, |ui| {
                if selected_tab.ui(ui).changed() {
                    self.state.tabs[selected_tab_index].1 = selected_tab.state().clone();
                    self.state.save(&self.handle);
                }
            });
        } else {
            CentralPanel::default().show(ctx, |ui| {
                self.welcome_screen(ui);
            });
        }
    }
}
