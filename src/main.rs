mod app;
mod converter;
mod frame;
mod hledger;
mod widgets;

use std::process::exit;

use app::App;

use eframe::{epaint::vec2, IconData};
use tracing::{metadata::LevelFilter, subscriber::set_global_default};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, Layer};

use crate::frame::state::app::State;

#[tokio::main]
async fn main() {
    init_logs();

    tracing::info!("starting app");

    let state = State::load()
        .map_err(|error| tracing::error!(%error, "failed to load state"))
        .unwrap_or_default();

    let native_options = eframe::NativeOptions {
        initial_window_size: Some(vec2(state.window.size[0], state.window.size[1])),
        initial_window_pos: state.window.position.map(Into::into),
        fullscreen: state.window.fullscreen,
        maximized: state.window.maximized,
        drag_and_drop_support: true,
        #[cfg(target_os = "macos")]
        fullsize_content: true,
        icon_data: Some(
            IconData::try_from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                .expect("failed to parse icon"),
        ),
        ..Default::default()
    };

    let manager = hledger::Manager::default();
    if let Err(error) = eframe::run_native(
        "hledger",
        native_options,
        Box::new(|cc| Box::new(App::new(cc, manager, state))),
    ) {
        tracing::error!(%error, "failed to run the app");
        exit(2)
    }
}

fn init_logs() {
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
                    .with_filter(log_level),
            ),
    )
    .expect("failed to set global logs subscriber");
}
