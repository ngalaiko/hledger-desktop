mod app;
mod glob;
mod journal;

use app::HledgerDesktop;
use tracing_subscriber::util::SubscriberInitExt;

#[allow(clippy::missing_errors_doc)]
pub fn main() -> iced::Result {
    tracing_subscriber::fmt::fmt()
        .with_span_events(
            tracing_subscriber::fmt::format::FmtSpan::NEW
                | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
        )
        .finish()
        .init();

    iced::application(
        HledgerDesktop::title,
        HledgerDesktop::update,
        HledgerDesktop::view,
    )
    .run()
}
