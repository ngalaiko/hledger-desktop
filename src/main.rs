mod app;
mod hledger;
mod widgets;

use tauri::Manager;

use app::App;

fn main() {
    tracing_subscriber::fmt::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            app.wry_plugin(tauri_egui::EguiPluginBuilder::new(app.handle()));

            let native_options = tauri_egui::eframe::NativeOptions {
                drag_and_drop_support: true,
                initial_window_size: Some([800.0, 600.0].into()),
                ..Default::default()
            };

            let handle = app.handle().clone();
            app.state::<tauri_egui::EguiPluginHandle>()
                .create_window(
                    "main".to_string(),
                    Box::new(|cc| Box::new(App::new(cc, handle))),
                    "hledger-desktop".into(),
                    native_options,
                )
                .expect("failed to create window");

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, _event| {});
}
