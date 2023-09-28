mod app;
mod converter;
mod frame;
mod hledger;
mod widgets;

use std::fs;

use tauri::{AppHandle, Manager};

use app::App;

use tracing::{metadata::LevelFilter, subscriber::set_global_default};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, Layer};

#[tokio::main]
async fn main() {
    tauri::async_runtime::set(tokio::runtime::Handle::current());
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            init_logs(&handle);

            tracing::info!("starting app");

            app.wry_plugin(tauri_egui::EguiPluginBuilder::new(app.handle()));

            let native_options = tauri_egui::eframe::NativeOptions {
                drag_and_drop_support: true,
                initial_window_size: Some([800.0, 600.0].into()),
                ..Default::default()
            };

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

fn init_logs(handle: &AppHandle) {
    let logs_dir = handle
        .path()
        .app_log_dir()
        .expect("failed to get app log dir");
    fs::create_dir_all(&logs_dir).expect("failed to create logs dir");

    let file_appender = tracing_appender::rolling::never(&logs_dir, "log.txt");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
    handle.manage(guard); // keep the guard alive for the lifetime of the app

    let log_format = tracing_subscriber::fmt::format()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .compact();

    let log_level = if cfg!(debug) {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    set_global_default(
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(log_format.clone())
                    .with_span_events(FmtSpan::CLOSE)
                    .with_filter(log_level),
            )
            .with(
                // subscriber that writes spans to a file
                tracing_subscriber::fmt::layer()
                    .event_format(log_format)
                    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                    .with_writer(file_writer)
                    .with_filter(log_level),
            ),
    )
    .expect("failed to set global logs subscriber");
}
