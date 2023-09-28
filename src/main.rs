mod app;
mod converter;
mod frame;
mod hledger;
mod widgets;

use std::fs;

use tauri::{AppHandle, Manager};

use app::App;

use tauri_egui::egui::vec2;
use tracing::{metadata::LevelFilter, subscriber::set_global_default};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, Layer};

use crate::frame::state::app::State;

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

            let state = State::try_from(&handle).unwrap_or_default();

            let native_options = tauri_egui::eframe::NativeOptions {
                initial_window_size: Some(vec2(state.window.size[0], state.window.size[1])),
                initial_window_pos: state.window.position.map(|p| p.into()),
                fullscreen: state.window.fullscreen,
                maximized: state.window.maximized,
                drag_and_drop_support: true,
                icon_data: None,
                #[cfg(target_os = "macos")]
                fullsize_content: true,
                ..Default::default()
            };

            let manager = hledger::Manager::from(&handle);
            app.manage(manager);

            app.state::<tauri_egui::EguiPluginHandle>()
                .create_window(
                    "main".to_string(),
                    Box::new(|cc| Box::new(App::new(cc, handle, state))),
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
