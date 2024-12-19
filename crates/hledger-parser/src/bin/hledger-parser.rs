#![recursion_limit = "256"]

use ariadne::{Color, Config, IndexType, Label, Report, ReportKind, Source};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(long, env)]
    ledger_file: std::path::PathBuf,
}

#[allow(clippy::missing_panics_doc)]
pub fn main() {
    let cli = Cli::parse();
    let contents = match std::fs::read_to_string(cli.ledger_file) {
        Ok(contents) => contents,
        Err(error) => {
            println!("{error}");
            std::process::exit(1);
        }
    };

    let result = hledger_parser::parse(&contents);
    match result {
        Ok(directives) => {
            println!("{directives:#?}");
            std::process::exit(1);
        }
        Err(errs) => {
            for err in errs {
                Report::build(ReportKind::Error, (), err.span.start)
                    .with_config(Config::default().with_index_type(IndexType::Byte))
                    .with_label(
                        Label::new(err.span)
                            .with_message(err.message)
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(Source::from(&contents))
                    .expect("should build report");
            }
        }
    }
}
