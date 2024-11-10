use std::sync::Arc;

use glob::glob;
use hledger_parser::{Directive, Format, Include};
use iced::futures::stream::{self, StreamExt};

#[derive(Debug, Clone)]
pub struct Journal {
    pub path: std::path::PathBuf,
    pub directives: Vec<hledger_parser::Directive>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum LoadError {
    #[error("io: {0}")]
    Io(std::io::ErrorKind),
    #[error("failed to parse glob")]
    Glob(Arc<glob::PatternError>),
    #[error("failed to parse file")]
    Parse(Vec<hledger_parser::ParseError>),
}

impl Journal {
    pub async fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, LoadError> {
        load(path).await
    }
}

#[tracing::instrument(skip_all, fields(path = %path.as_ref().display()))]
async fn parse<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<Directive>, LoadError> {
    let contents = smol::fs::read_to_string(&path)
        .await
        .map_err(|error| LoadError::Io(error.kind()))?;
    hledger_parser::parse(contents).map_err(LoadError::Parse)
}

async fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Journal, LoadError> {
    let path = path.as_ref();

    let directives = parse(path).await?;

    let includes = directives
        .iter()
        .filter_map(|directive| match directive {
            Directive::Include(Include {
                path: include_path,
                format: None | Some(Format::Journal),
            }) => path.parent().map(|parent| parent.join(include_path)),
            _ => None,
        })
        .collect::<Vec<_>>();

    if includes.is_empty() {
        Ok(Journal {
            path: path.to_path_buf(),
            directives,
        })
    } else {
        let subjournals = load_many_globs(includes).await;
        let subjournals = subjournals.into_iter().collect::<Result<Vec<_>, _>>()?;
        Ok(Journal {
            path: path.to_path_buf(),
            directives: directives
                .into_iter()
                .chain(subjournals.into_iter().flatten().flat_map(|j| j.directives))
                .collect(),
        })
    }
}

async fn load_many_globs<P: AsRef<std::path::Path>>(
    patterns: Vec<P>,
) -> Vec<Result<Vec<Journal>, LoadError>> {
    stream::iter(patterns)
        .map(|pattern| load_glob(pattern))
        .buffer_unordered(1024)
        .collect::<Vec<_>>()
        .await
}

async fn load_glob<P: AsRef<std::path::Path>>(pattern: P) -> Result<Vec<Journal>, LoadError> {
    let pattern = pattern.as_ref();

    let files = glob(&pattern.display().to_string())
        .map_err(|error| LoadError::Glob(Arc::new(error)))?
        .map(|resolved_file| {
            resolved_file.map_err(|glob_error| LoadError::Io(glob_error.error().kind()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let journals = load_many(files).await;
    journals.into_iter().collect::<Result<Vec<_>, _>>()
}

async fn load_many<P: AsRef<std::path::Path>>(paths: Vec<P>) -> Vec<Result<Journal, LoadError>> {
    stream::iter(paths)
        .map(|path| Journal::load(path))
        .buffer_unordered(1024)
        .collect::<Vec<_>>()
        .await
}
