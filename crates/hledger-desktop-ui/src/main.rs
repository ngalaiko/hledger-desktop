use std::{process::exit, sync::Arc};

use eframe::icon_data::from_png_bytes;
use macro_rules_attribute::apply;
use smol_macros::main;
use tracing::{metadata::LevelFilter, subscriber::set_global_default};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, Layer};

use hledger_desktop_ui::app::App;
use hledger_desktop_ui::persistance;

#[apply(main!)]
async fn main() {
    init_logs();

    tracing::info!("starting app");

    let state = persistance::load_state()
        .map_err(|error| tracing::error!(%error, "failed to load state"))
        .unwrap_or_default();

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder {
            fullscreen: Some(state.window.fullscreen),
            maximized: Some(state.window.maximized),
            inner_size: Some(state.window.size.into()),
            position: state.window.position.map(Into::into),
            fullsize_content_view: Some(true),
            titlebar_shown: Some(false),
            icon: Some(Arc::new(
                from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .expect("failed to parse icon"),
            )),
            ..Default::default()
        },
        ..Default::default()
    };

    if let Err(error) = eframe::run_native(
        "hledger",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, state)))),
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

    let log_level = if cfg!(debug_assertions) {
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
