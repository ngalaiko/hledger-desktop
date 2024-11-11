use std::sync::Arc;

use hledger_parser::{Directive, Format, Include};
use iced::futures::stream::{self, StreamExt};

use crate::glob::walk;

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
    Glob(Arc<wax::BuildError>),
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
            }) => Some(wax::Glob::new(include_path.as_os_str().to_str().unwrap())),
            _ => None,
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| LoadError::Glob(Arc::new(error)))?;

    if includes.is_empty() {
        Ok(Journal {
            path: path.to_path_buf(),
            directives,
        })
    } else {
        let subjournals = load_many_globs(path.parent().unwrap(), includes).await?;
        Ok(Journal {
            path: path.to_path_buf(),
            directives: directives
                .into_iter()
                .chain(subjournals.into_iter().flat_map(|j| j.directives))
                .collect(),
        })
    }
}

async fn load_many_globs<'a, P: wax::Combine<'a>>(
    path: &std::path::Path,
    patterns: Vec<P>,
) -> Result<Vec<Journal>, LoadError> {
    let patterns = wax::any(patterns).map_err(|error| LoadError::Glob(Arc::new(error)))?;
    let paths = walk(path, &patterns).as_stream().collect::<Vec<_>>().await;
    let paths = paths
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            if let Some(io_error) = error.io() {
                LoadError::Io(io_error.kind())
            } else {
                LoadError::Io(std::io::ErrorKind::Other)
            }
        })?;
    let journals = load_many(paths).await;
    journals.into_iter().collect::<Result<Vec<_>, _>>()
}

async fn load_many<P: AsRef<std::path::Path>>(paths: Vec<P>) -> Vec<Result<Journal, LoadError>> {
    stream::iter(paths)
        .map(|path| Journal::load(path))
        .buffer_unordered(1024)
        .collect::<Vec<_>>()
        .await
}
