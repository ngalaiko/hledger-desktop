use std::{io, path, str, sync::Arc};

use rand::Rng;
use tauri::{async_runtime, AppHandle};
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent, TerminatedPayload},
    ShellExt,
};
use tokio::{select, sync::watch};
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use url::Url;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("hledger-web not found")]
    NotFound,
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
    state: (Arc<watch::Sender<State>>, watch::Receiver<State>),
    cancel_token: CancellationToken,
}

impl HLedgerWeb {
    pub async fn new<P: AsRef<path::Path>>(
        handle: &AppHandle,
        file_path: P,
    ) -> Result<Self, Error> {
        let file_path = file_path.as_ref().to_path_buf();
        let port = rand::thread_rng().gen_range(32768..65536);
        let endpoint =
            Url::parse(format!("http://localhost:{}", port).as_str()).expect("failed to parse url");
        let (state_tx, mut state_rx) = watch::channel(State::Starting);
        let state_tx = Arc::new(state_tx);
        let cancel_token = CancellationToken::new();
        let c_cancel_token = cancel_token.clone();
        let _handle: async_runtime::JoinHandle<Result<(), Error>> = tauri::async_runtime::spawn({
            let handle = handle.clone();
            let state_tx = state_tx.clone();
            let span = tracing::span!(
                tracing::Level::INFO,
                "hledger-web",
                file_path = file_path.display().to_string()
            );
            async move {
                let _span_guard = span.enter();

                let send_state = |state: State| {
                    tracing::info!(?state);
                    state_tx.send(state).unwrap();
                };

                send_state(State::Starting);
                match spawn(&handle, &file_path, &port).await {
                    Err(error) => {
                        send_state(State::Stopped(Some(error.clone())));
                        Err(error)
                    }
                    Ok((mut rx, child)) => loop {
                        select! {
                            _ = c_cancel_token.cancelled() => {
                                if child.kill().is_err() {
                                    send_state(State::Stopped(Some(Error::FailedToStop)));
                                    return Err(Error::FailedToStop);
                                } else {
                                    send_state(State::Stopped(None));
                                    return Ok(());
                                }
                            }
                            Some(event) = rx.recv() =>  match event {
                                CommandEvent::Stdout(line) => {
                                    let line = str::from_utf8(&line).unwrap();
                                    tracing::debug!(line);
                                    if line.eq("Press ctrl-c to quit") {
                                        send_state(State::Running);
                                    }
                                }
                                CommandEvent::Stderr(line) => {
                                    let line = str::from_utf8(&line).unwrap();
                                    tracing::error!(line);
                                }
                                CommandEvent::Error(error) => {
                                    send_state(State::Stopped(Some(Error::CommandEvent(error.clone()))));
                                    return Err(Error::CommandEvent(error));
                                }
                                CommandEvent::Terminated(payload) => {
                                    send_state(State::Stopped(Some(Error::Terminated(payload.clone()))));
                                    return Err(Error::Terminated(payload));
                                }
                                _ => {}
                            }
                        }
                    },
                }
            }
        });

        while state_rx.changed().await.is_ok() {
            match state_rx.borrow().clone() {
                State::Starting => continue,
                State::Running => break,
                State::Stopped(error) => return Err(error.unwrap_or(Error::FailedToSpawn)),
            }
        }

        Ok(Self {
            endpoint,
            state: (state_tx, state_rx),
            cancel_token,
        })
    }

    pub async fn stop(&mut self) -> Result<(), Error> {
        self.cancel_token.cancel();
        while self.state.1.changed().await.is_ok() {
            match self.state.1.borrow().clone() {
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

#[instrument(skip(handle, path))]
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
        .command("hledger-web")
        .args(args)
        .spawn()
        .map_err(|error| match error {
            tauri_plugin_shell::Error::Io(error) => {
                if error.kind() == io::ErrorKind::NotFound {
                    Error::NotFound
                } else {
                    tracing::error!(?error);
                    Error::FailedToSpawn
                }
            }
            error => {
                tracing::error!(?error);
                Error::FailedToSpawn
            }
        })
}
