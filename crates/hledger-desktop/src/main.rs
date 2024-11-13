mod app;
mod glob;
mod journal;
mod watcher;
mod promise;

use app::App;
use tracing_subscriber::util::SubscriberInitExt;

#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]
pub fn main() -> iced::Result {
    tracing_subscriber::fmt::fmt()
        .with_span_events(
            tracing_subscriber::fmt::format::FmtSpan::NEW
                | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
        )
        .finish()
        .init();

    iced::application(App::title, App::update, App::view)
        .subscription(App::file_watcher)
        .run()
}
