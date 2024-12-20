use std::{process::exit, sync::Arc};

use eframe::icon_data::from_png_bytes;
use macro_rules_attribute::apply;
use smol_macros::{main, Executor};
use tracing::Level;
use tracing::{metadata::LevelFilter, subscriber::set_global_default};
use tracing_subscriber::filter::Targets;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, Layer};

use hledger_desktop::app::App;

#[apply(main!)]
async fn main(executor: Arc<Executor<'static>>) {
    init_logs();

    tracing::info!("starting app");

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder {
            title_shown: Some(false),
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
        "rocks.galaiko.hledger.desktop",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc, executor)))),
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

    let filter = Targets::new()
        .with_default(log_level)
        .with_target("winit", Level::INFO);

    set_global_default(
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .event_format(log_format.clone())
                    .with_span_events(FmtSpan::CLOSE)
                    .with_filter(filter.clone()),
            )
            .with(
                // subscriber that writes spans to a file
                tracing_subscriber::fmt::layer()
                    .event_format(log_format)
                    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                    .with_filter(filter),
            ),
    )
    .expect("failed to set global logs subscriber");
}
