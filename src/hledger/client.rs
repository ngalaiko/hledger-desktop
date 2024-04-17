use std::{path, string, sync::Arc};

use reqwest::{Client, Method};
use tracing::instrument;

use super::{process, types};

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("failed to spawn hledger-web: {0}")]
    Process(process::Error),
    #[error("failed to send http request: {0}")]
    Http(Arc<reqwest::Error>),
    #[error("api error: {0}")]
    Api(String),
    #[error("utf8: {0}")]
    Utf8(string::FromUtf8Error),
    #[error("failed to parse json: {0}")]
    Json(Arc<serde_json::Error>),
}

impl From<process::Error> for Error {
    fn from(err: process::Error) -> Self {
        Self::Process(err)
    }
}

#[derive(Debug, Clone)]
pub struct HLedgerWeb {
    inner: Arc<HLedgerWebInner>,
}

impl HLedgerWeb {
    pub async fn new<P: AsRef<path::Path>>(file_path: P) -> Result<Self, Error> {
        let inner = HLedgerWebInner::new(file_path).await?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub async fn prices(&self) -> Result<Vec<types::Price>, Error> {
        self.inner.prices().await
    }

    pub async fn commodities(&self) -> Result<Vec<types::Commodity>, Error> {
        self.inner.commodities().await
    }

    pub async fn accounts(&self) -> Result<Vec<types::Account>, Error> {
        self.inner.accounts().await
    }

    pub async fn transactions(&self) -> Result<Vec<types::Transaction>, Error> {
        self.inner.transactions().await
    }

    pub async fn add(&self, transaction: &types::Transaction) -> Result<(), Error> {
        self.inner.add(transaction).await
    }
}

#[derive(Debug)]
struct HLedgerWebInner {
    file_path: path::PathBuf,
    process: process::HLedgerWeb,
    client: Client,
}

impl HLedgerWebInner {
    pub async fn new<P: AsRef<path::Path>>(file_path: P) -> Result<Self, Error> {
        let file_path = file_path.as_ref();
        Ok(Self {
            process: process::HLedgerWeb::new(file_path)
                .await
                .map_err(Error::Process)?,
            file_path: file_path.to_path_buf(),
            client: Client::new(),
        })
    }

    #[instrument(skip_all, fields(?self.file_path))]
    pub async fn commodities(&self) -> Result<Vec<types::Commodity>, Error> {
        let url = self
            .process
            .endpoint()
            .join("/commodities")
            .expect("failed to join url");
        let request = self.client.request(Method::GET, url);
        let response = request
            .send()
            .await
            .map_err(|error| Error::Http(Arc::new(error)))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|error| Error::Http(Arc::new(error)))?;
        serde_json::from_slice(&bytes).map_err(|error| Error::Json(Arc::new(error)))
    }

    #[instrument(skip_all, fields(?self.file_path))]
    pub async fn prices(&self) -> Result<Vec<types::Price>, Error> {
        let url = self
            .process
            .endpoint()
            .join("/prices")
            .expect("failed to join url");
        let request = self.client.request(Method::GET, url);
        let response = request
            .send()
            .await
            .map_err(|error| Error::Http(Arc::new(error)))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|error| Error::Http(Arc::new(error)))?;
        serde_json::from_slice(&bytes).map_err(|error| Error::Json(Arc::new(error)))
    }

    #[instrument(skip_all, fields(?self.file_path))]
    pub async fn accounts(&self) -> Result<Vec<types::Account>, Error> {
        let url = self
            .process
            .endpoint()
            .join("/accounts")
            .expect("failed to join url");
        let request = self.client.request(Method::GET, url);
        let response = request
            .send()
            .await
            .map_err(|error| Error::Http(Arc::new(error)))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|error| Error::Http(Arc::new(error)))?;
        serde_json::from_slice(&bytes).map_err(|error| Error::Json(Arc::new(error)))
    }

    #[instrument(skip_all, fields(?self.file_path))]
    pub async fn transactions(&self) -> Result<Vec<types::Transaction>, Error> {
        let url = self
            .process
            .endpoint()
            .join("/transactions")
            .expect("failed to join url");
        let request = self.client.request(Method::GET, url);
        let response = request
            .send()
            .await
            .map_err(|error| Error::Http(Arc::new(error)))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|error| Error::Http(Arc::new(error)))?;
        serde_json::from_slice(&bytes).map_err(|error| Error::Json(Arc::new(error)))
    }

    #[instrument(skip_all, fields(?self.file_path))]
    pub async fn add(&self, transaction: &types::Transaction) -> Result<(), Error> {
        let url = self
            .process
            .endpoint()
            .join("/add")
            .expect("failed to join url");
        let request = self
            .client
            .request(Method::PUT, url)
            .header("Content-Type", "application/json")
            .json(&transaction);
        let response = request
            .send()
            .await
            .map_err(|error| Error::Http(Arc::new(error)))?;
        if response.status().is_client_error() {
            let bytes = response
                .bytes()
                .await
                .map_err(|error| Error::Http(Arc::new(error)))?;
            let message = String::from_utf8(bytes.to_vec()).map_err(Error::Utf8)?;
            Err(Error::Api(message))
        } else {
            Ok(())
        }
    }
}
