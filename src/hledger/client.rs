use std::{path, sync::Arc};

use reqwest::{Client, Method};
use tauri::AppHandle;

use super::{process, types};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to spawn hledger-web: {0}")]
    Process(#[from] process::Error),
    #[error("failed to send http request: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to parse json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
pub struct HLedgerWeb {
    inner: Arc<HLedgerWebInner>,
}

impl HLedgerWeb {
    pub async fn new<P: AsRef<path::Path>>(
        handle: &AppHandle,
        file_path: P,
    ) -> Result<Self, Error> {
        let inner = HLedgerWebInner::new(handle, file_path).await?;
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub async fn prices(&self) -> Result<Vec<types::Price>, Error> {
        self.inner.prices().await
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
    process: process::HLedgerWeb,
    client: Client,
}

impl HLedgerWebInner {
    pub async fn new<P: AsRef<path::Path>>(
        handle: &AppHandle,
        file_path: P,
    ) -> Result<Self, Error> {
        Ok(Self {
            process: process::HLedgerWeb::new(handle, file_path.as_ref())
                .await
                .map_err(Error::Process)?,
            client: Client::new(),
        })
    }

    pub async fn prices(&self) -> Result<Vec<types::Price>, Error> {
        let url = self
            .process
            .endpoint()
            .join("/prices")
            .expect("failed to join url");
        let request = self.client.request(Method::GET, url);
        let response = request.send().await.map_err(Error::Http)?;
        let bytes = response.bytes().await.map_err(Error::Http)?;
        serde_json::from_slice(&bytes).map_err(Error::Json)
    }

    pub async fn accounts(&self) -> Result<Vec<types::Account>, Error> {
        let url = self
            .process
            .endpoint()
            .join("/accounts")
            .expect("failed to join url");
        let request = self.client.request(Method::GET, url);
        let response = request.send().await.map_err(Error::Http)?;
        let bytes = response.bytes().await.map_err(Error::Http)?;
        serde_json::from_slice(&bytes).map_err(Error::Json)
    }

    pub async fn transactions(&self) -> Result<Vec<types::Transaction>, Error> {
        let url = self
            .process
            .endpoint()
            .join("/transactions")
            .expect("failed to join url");
        let request = self.client.request(Method::GET, url);
        let response = request.send().await.map_err(Error::Http)?;
        let bytes = response.bytes().await.map_err(Error::Http)?;
        serde_json::from_slice(&bytes).map_err(Error::Json)
    }

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
        let response = request.send().await.map_err(Error::Http)?;
        response.error_for_status_ref().map_err(Error::Http)?;
        Ok(())
    }
}
