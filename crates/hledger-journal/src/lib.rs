mod glob;

use std::sync::Arc;

use futures::{
    channel::oneshot,
    stream::{self, StreamExt},
};
use hledger_parser::{Directive, Format, Include};

use crate::glob::walk;

#[derive(Debug, Clone)]
pub struct Journal {
    pub path: std::path::PathBuf,
    directives: Vec<hledger_parser::Directive>,
    includes: Vec<Journal>,
}

pub use hledger_parser::ParseError;
pub use hledger_parser::Transaction;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("io: {0}")]
    Io(std::io::ErrorKind),
    #[error("failed to parse glob")]
    Glob(Arc<wax::BuildError>),
    #[error("failed to parse file")]
    Parse(Vec<ParseError>),
}

impl Journal {
    #[allow(clippy::missing_errors_doc)]
    pub async fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        load(path).await
    }

    #[must_use]
    pub fn includes(&self) -> Vec<std::path::PathBuf> {
        std::iter::once(self.path.clone())
            .chain(self.includes.iter().map(|journal| journal.path.clone()))
            .collect()
    }

    pub fn transactions(&self) -> impl Iterator<Item = &Transaction> {
        self.directives().filter_map(|directive| match directive {
            Directive::Transaction(tx) => Some(tx),
            _ => None,
        })
    }

    fn directives(&self) -> impl Iterator<Item = &hledger_parser::Directive> {
        self.directives.iter().chain(
            self.includes
                .iter()
                .flat_map(|included| included.directives.iter()),
        )
    }

    pub fn merge(&mut self, other: &Journal) -> bool {
        if self.path == other.path {
            self.directives.clone_from(&other.directives);
            self.includes.clone_from(&other.includes);
            true
        } else {
            for included in &mut self.includes {
                if included.merge(other) {
                    return true;
                }
            }
            false
        }
    }
}

async fn parse<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<Directive>, Error> {
    let contents = async_fs::read_to_string(&path)
        .await
        .map_err(|error| Error::Io(error.kind()))?;
    let (send, recv) = oneshot::channel();
    rayon::spawn(move || {
        let result = hledger_parser::parse(contents).map_err(Error::Parse);
        let _ = send.send(result);
    });
    recv.await.expect("panic in rayon::spawn")
}

#[tracing::instrument(skip_all, fields(path = %path.as_ref().display()))]
async fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Journal, Error> {
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
        .map_err(|error| Error::Glob(Arc::new(error)))?;

    let includes = load_many_globs(path.parent().unwrap(), includes).await?;
    Ok(Journal {
        path: path.to_path_buf(),
        directives,
        includes,
    })
}

async fn load_many_globs<'a, P: wax::Combine<'a>>(
    path: &std::path::Path,
    patterns: Vec<P>,
) -> Result<Vec<Journal>, Error> {
    let patterns = wax::any(patterns).map_err(|error| Error::Glob(Arc::new(error)))?;
    let paths = walk(path, &patterns).as_stream().collect::<Vec<_>>().await;
    let paths = paths
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            if let Some(io_error) = error.io() {
                Error::Io(io_error.kind())
            } else {
                Error::Io(std::io::ErrorKind::Other)
            }
        })?;
    let journals = load_many(paths).await;
    journals.into_iter().collect::<Result<Vec<_>, _>>()
}

async fn load_many<P: AsRef<std::path::Path>>(paths: Vec<P>) -> Vec<Result<Journal, Error>> {
    stream::iter(paths)
        .map(|path| Journal::load(path))
        .buffer_unordered(1024)
        .collect::<Vec<_>>()
        .await
}
