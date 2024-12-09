use eframe::egui::Ui;

pub fn ui(ui: &mut Ui) {
    ui.horizontal_centered(|ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 2.0 - 50.0);
            ui.spinner()
        });
    });
}
