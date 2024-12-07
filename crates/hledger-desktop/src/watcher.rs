use std::time::Duration;

use iced::{
    futures::{self, channel::mpsc, SinkExt, Stream, StreamExt},
    stream,
};
use notify_debouncer_mini::{new_debouncer, Debouncer};

#[derive(Debug)]
pub enum Input {
    Watch(Vec<std::path::PathBuf>),
}

#[derive(Debug, Clone)]
pub enum Message {
    Started(mpsc::Sender<Input>),
    FileChange(Vec<std::path::PathBuf>),
}

enum State<W: notify::Watcher> {
    NotRunning,
    Running(
        Debouncer<W>,
        mpsc::Receiver<Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>>,
        mpsc::Receiver<Input>,
    ),
}

pub fn run() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        let mut state = State::NotRunning;

        loop {
            match &mut state {
                State::NotRunning => {
                    let (mut tx, rx) = mpsc::channel(100);
                    let debouncer = new_debouncer(Duration::from_millis(200), move |res| {
                        futures::executor::block_on(async {
                            tx.send(res).await.unwrap();
                        });
                    })
                    .unwrap();
                    let (input_tx, input_rx) = mpsc::channel(100);
                    state = State::Running(debouncer, rx, input_rx);
                    let _ = output.send(Message::Started(input_tx)).await;
                }
                State::Running(debouncer, rx, input_rx) => {
                    let mut fused_rx = rx.by_ref().fuse();
                    let mut fused_input_rx = input_rx.by_ref().fuse();
                    futures::select! {
                        event = fused_rx.select_next_some() => {
                            match event {
                                Ok(events) => {
                                    let paths = events.into_iter().map(|e| e.path).collect();
                                    let _ = output.send(Message::FileChange(paths)).await;
                                },
                                Err(error) => {
                                    tracing::error!(?error, "watcher error");
                                }
                            }
                        }
                        input_event = fused_input_rx.select_next_some() => {
                            match input_event {
                                Input::Watch(paths) => {
                                    let watcher = debouncer.watcher();
                                    for path in &paths {
                                    let _ = watcher
                                        .watch(path, notify::RecursiveMode::NonRecursive)
                                        .map(|()| tracing::info!(path = %path.display(), "started watching"))
                                        .map_err(|error| tracing::error!(?error, "failed to watch"));
                                    }
                                }
                            };
                        }
                    }
                }
            }
        }
    })
}
