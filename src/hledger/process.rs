use std::{
    fmt, io, path,
    process::{ExitStatus, Stdio},
    str,
    sync::Arc,
};

use rand::Rng;

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    select,
    sync::watch,
    task::JoinHandle,
};
use tracing::instrument;
use url::Url;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("hledger-web not found")]
    NotFound,
    #[error("failed to execute hledger-web: {exit_status:?} {message}")]
    Terminated {
        exit_status: ExitStatus,
        message: String,
    },
    #[error("failed to spawn hledger-web: {0}")]
    FailedToRun(Arc<io::Error>),
}

#[derive(Debug, Clone)]
pub enum State {
    Starting,
    Running,
    Stopped(Option<Error>),
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::Starting => write!(f, "starting"),
            State::Running => write!(f, "running"),
            State::Stopped(None) => write!(f, "stopped"),
            State::Stopped(Some(error)) => write!(f, "stopped: {error}"),
        }
    }
}

#[derive(Debug)]
pub struct HLedgerWeb {
    endpoint: Url,
}

impl HLedgerWeb {
    pub async fn new<P: AsRef<path::Path>>(file_path: P) -> Result<Self, Error> {
        let file_path = file_path.as_ref().to_path_buf();
        let port = rand::thread_rng().gen_range(32768..65536);
        let endpoint =
            Url::parse(format!("http://localhost:{port}").as_str()).expect("failed to parse url");
        let (state_tx, mut state_rx) = watch::channel(State::Starting);
        let state_tx = Arc::new(state_tx);
        let _handle: JoinHandle<Result<(), Error>> = tokio::spawn({
            let state_tx = state_tx.clone();
            let span = tracing::span!(
                tracing::Level::INFO,
                "hledger-web",
                file_path = file_path.display().to_string()
            );
            async move {
                let _span_guard = span.enter();

                let send_state = |state: State| {
                    tracing::info!(%state);
                    state_tx.send(state).unwrap();
                };

                send_state(State::Starting);

                match spawn(&file_path, &port).await {
                    Err(error) => {
                        send_state(State::Stopped(Some(error.clone())));
                        Err(error)
                    }
                    Ok(mut child) => {
                        let stdout = child
                            .stdout
                            .take()
                            .expect("child did not have a handle to stdout");
                        let stderr = child
                            .stderr
                            .take()
                            .expect("child did not have a handle to stdout");

                        let mut stdout_reader = BufReader::new(stdout).lines();
                        let mut stderr_reader = BufReader::new(stderr).lines();

                        loop {
                            select! {
                                line = stdout_reader.next_line() => {
                                    let Ok(Some(line)) = line else {
                                        continue;
                                    };
                                    tracing::debug!(line);
                                    if line.eq("Press ctrl-c to quit") {
                                        send_state(State::Running);
                                    }
                                },
                                exit_code = child.wait() => {
                                    match exit_code {
                                        Ok(exit_code) if exit_code.success() => {
                                            send_state(State::Stopped(None));
                                            return Ok(());
                                        }
                                        Ok(exit_status) => {
                                            let mut message = vec![];
                                            while let Ok(Some(line)) = stderr_reader.next_line().await {
                                                message.push(line);
                                            }
                                            let error = Error::Terminated{
                                                exit_status,
                                                message: message.join("\n")
                                            };
                                            send_state(State::Stopped(Some(error.clone())));
                                            return Err(error);
                                        }
                                        Err(error) => {
                                            let error = Arc::new(error);
                                            send_state(State::Stopped(Some(Error::FailedToRun(error.clone()))));
                                            return Err(Error::FailedToRun(error))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        while state_rx.changed().await.is_ok() {
            match state_rx.borrow().clone() {
                State::Starting => continue,
                State::Running => break,
                State::Stopped(error) => error.map_or(Ok(()), Err)?,
            }
        }

        Ok(Self { endpoint })
    }

    pub fn endpoint(&self) -> &Url {
        &self.endpoint
    }
}

#[instrument(skip(path))]
async fn spawn(path: &path::Path, port: &usize) -> Result<tokio::process::Child, Error> {
    let args = [
        "--infer-costs".to_string(),
        "--infer-market-prices".to_string(),
        "--file".to_string(),
        path.display().to_string(),
        "--port".to_string(),
        port.to_string(),
        "--serve-api".to_string(),
    ];

    tokio::process::Command::new("hledger-web")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .args(args)
        .spawn()
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                Error::NotFound
            } else {
                Error::FailedToRun(Arc::new(error))
            }
        })
}

#[instrument]
pub async fn exec(args: &[&str]) -> Result<Vec<u8>, Error> {
    let output = tokio::process::Command::new("hledger-web")
        .args(args)
        .output()
        .await
        .map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                Error::NotFound
            } else {
                Error::FailedToRun(Arc::new(error))
            }
        })?;

    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(Error::Terminated {
            exit_status: output.status,
            message: String::from_utf8(output.stderr).expect("failed to parse stderr as utf8"),
        })
    }
}
