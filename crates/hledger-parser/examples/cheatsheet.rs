use ariadne::{Color, Config, IndexType, Label, Report, ReportKind, Source};

const CHEATSHEET_JOURNAL: &str = include_str!("./fixture/cheatsheet.journal");

pub fn main() {
    let result = hledger_parser::parse(CHEATSHEET_JOURNAL);

    match result {
        Ok(directives) => println!("{directives:#?}"),
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
                    .eprint(Source::from(&CHEATSHEET_JOURNAL))
                    .expect("should build report");
            }
        }
    }
}
