mod app;
mod hledger;
mod state;
mod ui;
mod widgets;

use tauri::Manager;

use app::App;

use tracing::{metadata::LevelFilter, subscriber::set_global_default};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, Layer};

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    init_logs();
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

            let manager = hledger::Manager::from(&handle);
            app.manage(manager);

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

fn init_logs() {
    set_global_default(
        tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .event_format(
                    tracing_subscriber::fmt::format()
                        .with_file(true)
                        .with_line_number(true)
                        .with_target(false)
                        .compact(),
                )
                .with_span_events(FmtSpan::CLOSE)
                .with_filter(if cfg!(debug) {
                    LevelFilter::DEBUG
                } else {
                    LevelFilter::INFO
                }),
        ),
    )
    .expect("failed to set global logs subscriber");
}
