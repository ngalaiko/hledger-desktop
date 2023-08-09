use std::{path, str};

use rand::Rng;
use tauri::{async_runtime, AppHandle};
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent, TerminatedPayload},
    ShellExt,
};
use tokio::{select, sync::watch};
use tokio_util::sync::CancellationToken;
use url::Url;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("failed to spawn hledger-web")]
    FailedToSpawn,
    #[error("failed to stop hledger-web")]
    FailedToStop,
    #[error("{0}")]
    CommandEvent(String),
    #[error("hledger-web terminated")]
    Terminated(TerminatedPayload),
}

#[derive(Debug, Clone)]
pub enum State {
    Starting,
    Running,
    Stopped(Option<Error>),
}

#[derive(Debug)]
pub struct HLedgerWeb {
    endpoint: Url,
    state_rx: watch::Receiver<State>,
    cancel_token: CancellationToken,
}

impl HLedgerWeb {
    pub fn new<P: AsRef<path::Path>>(handle: &AppHandle, file_path: P) -> Result<Self, Error> {
        let file_path = file_path.as_ref().to_path_buf();
        let port = rand::thread_rng().gen_range(32768..65536);
        let endpoint =
            Url::parse(format!("http://localhost:{}", port).as_str()).expect("failed to parse url");
        let (state_tx, state_rx) = watch::channel(State::Starting);
        let cancel_token = CancellationToken::new();
        let c_cancel_token = cancel_token.clone();
        let _handle: async_runtime::JoinHandle<Result<(), Error>> = tauri::async_runtime::spawn({
            let handle = handle.clone();
            async move {
                match spawn(&handle, &file_path, &port).await {
                    Err(error) => Err(error),
                    Ok((mut rx, child)) => loop {
                        select! {
                            _ = c_cancel_token.cancelled() => {
                                if child.kill().is_err() {
                                    state_tx.send(State::Stopped(Some(Error::FailedToStop))).unwrap();
                                    return Err(Error::FailedToStop);
                                } else {
                                    tracing::info!("hledger-web ({}): stopped", file_path.display());
                                    state_tx.send(State::Stopped(None)).unwrap();
                                    return Ok(());
                                }
                            }
                            Some(event) = rx.recv() =>  match event {
                                CommandEvent::Stdout(line) => {
                                    let line = str::from_utf8(&line).unwrap();
                                    tracing::trace!(
                                        "hledger-web({}): {}",
                                        file_path.display(),
                                        line
                                    );
                                    if line.eq("Press ctrl-c to quit") {
                                        tracing::info!("hledger-web({}): started", file_path.display());
                                        state_tx.send(State::Running).unwrap();
                                    }
                                }
                                CommandEvent::Stderr(line) => {
                                    let line = str::from_utf8(&line).unwrap();
                                    tracing::error!(
                                        "hledger-web({}): {}",
                                        file_path.display(),
                                        line
                                    );
                                }
                                CommandEvent::Error(error) => {
                                    tracing::error!(
                                        "hledger-web({}): {}",
                                        file_path.display(),
                                        error
                                    );
                                    state_tx.send(State::Stopped(Some(Error::CommandEvent(error.clone())))).unwrap();
                                    return Err(Error::CommandEvent(error));
                                }
                                CommandEvent::Terminated(payload) => {
                                    tracing::error!(
                                        "hledger-web({}): terminated",
                                        file_path.display()
                                    );
                                    state_tx.send(State::Stopped(Some(Error::Terminated(payload.clone())))).unwrap();
                                    return Err(Error::Terminated(payload));
                                }
                                _ => {}
                            }
                        }
                    },
                }
            }
        });

        Ok(Self {
            endpoint,
            state_rx,
            cancel_token,
        })
    }

    pub async fn wait_until_running(&mut self) -> Result<(), Error> {
        while self.state_rx.changed().await.is_ok() {
            match self.state_rx.borrow().clone() {
                State::Running => return Ok(()),
                State::Stopped(None) => return Err(Error::FailedToSpawn),
                State::Stopped(Some(error)) => return Err(error),
                _ => {}
            }
        }
        unreachable!()
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        self.cancel_token.cancel();
        while self.state_rx.changed().await.is_ok() {
            match self.state_rx.borrow().clone() {
                State::Stopped(None) => return Ok(()),
                State::Stopped(Some(error)) => return Err(error),
                _ => {}
            }
        }
        unreachable!()
    }

    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }
}

impl Drop for HLedgerWeb {
    fn drop(&mut self) {
        if let Err(e) = futures::executor::block_on(self.stop()) {
            tracing::error!("hledger-web: failed to stop: {}", e);
        }
    }
}

async fn spawn(
    handle: &AppHandle,
    path: &path::Path,
    port: &usize,
) -> Result<(tauri::async_runtime::Receiver<CommandEvent>, CommandChild), Error> {
    let args = [
        "--infer-costs".to_string(),
        "--infer-market-prices".to_string(),
        "--file".to_string(),
        path.display().to_string(),
        "--port".to_string(),
        port.to_string(),
        "--serve-api".to_string(),
    ];
    handle
        .shell()
        .sidecar("hledger-web")
        .map_err(|e| {
            tracing::error!(
                "hledger-web ({}): failed to prepare sidecar: {}",
                path.display(),
                e
            );
            Error::FailedToSpawn
        })?
        .args(args)
        .spawn()
        .map_err(|e| {
            tracing::error!("hledger-web ({}): failed to span: {}", path.display(), e);
            Error::FailedToSpawn
        })
}
