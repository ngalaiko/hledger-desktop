use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use async_lock::{Mutex, MutexGuardArc};
use async_task::Task;
use futures::future::{self, join3};
use notify_debouncer_mini::new_debouncer;
use smol_macros::Executor;

pub struct Watcher {
    _task: future::Join3<Task<()>, Task<()>, Task<()>>,

    journal: Arc<Mutex<Option<hledger_journal::Journal>>>,
    error: Arc<Mutex<HashMap<std::path::PathBuf, hledger_journal::Error>>>,
}

enum WatcherTask {
    Watch(Vec<std::path::PathBuf>),
    Unwatch(Vec<std::path::PathBuf>),
}

impl Watcher {
    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::too_many_lines)]
    pub fn watch<P: AsRef<std::path::Path>>(executor: &Executor<'static>, path: P) -> Self {
        let path = path.as_ref();

        let (watcher_sender, watcher_receiver) = async_channel::unbounded();

        let (watch_sender, watch_receiver) = async_channel::unbounded::<WatcherTask>();
        let path_clone = path.to_path_buf();
        let debouncer_task = executor.spawn(async move {
            let mut debouncer = new_debouncer(Duration::from_millis(200), move |res| {
                futures::executor::block_on(async {
                    watcher_sender.send(res).await.unwrap();
                });
            })
            .expect("failed to start file watcher");

            loop {
                match watch_receiver.recv().await {
                    Ok(WatcherTask::Watch(paths)) => {
                        for path in paths{
                            debouncer.watcher()
                                .watch(&path, notify::RecursiveMode::NonRecursive)
                                .unwrap();
                            tracing::info!(path = %path.display(), "start watching");
                        }
                    }
                    Ok(WatcherTask::Unwatch(paths)) => {
                        for path in paths{
                            debouncer.watcher()
                                .unwatch(&path)
                                .unwrap();
                            tracing::info!(path = %path.display(), "stop watching");
                        }
                    }
                    Err(error) => {
                        tracing::error!(?error, path = %path_clone.display(), "watch event receiver error");
                    }
                }
            }
        });

        let path_clone = path.to_path_buf();
        let (paths_sender, paths_receiver) = async_channel::unbounded();
        let path_sender_clone = paths_sender.clone();
        let changed_paths_task = executor.spawn(async move {
            loop {
                match watcher_receiver.recv().await {
                    Ok(Ok(events)) => {
                        let paths = events.into_iter().map(|e| e.path).collect::<HashSet<_>>();
                        for path in &paths {
                            path_sender_clone.send(path.clone()).await.unwrap();
                        }
                    }
                    Ok(Err(error)) => {
                        tracing::error!(?error, path = %path_clone.display(), "watcher error");
                    }
                    Err(error) => {
                        tracing::error!(?error, path = %path_clone.display(), "watcher event receiver error");
                    }
                }
            }
        });

        let path_clone = path.to_path_buf();
        let journal = Arc::new(Mutex::new(None::<hledger_journal::Journal>));
        let journal_clone = journal.clone();
        let error = Arc::new(Mutex::new(HashMap::new()));
        let error_clone = error.clone();
        let process_path_task = executor.spawn(async move {
            loop {
                match paths_receiver.recv().await {
                    Ok(path) => {
                        let parsed_journal = hledger_journal::Journal::load(&path).await;
                        match parsed_journal {
                            Ok(parsed_journal) => {
                                let mut journal_guard = journal_clone.lock_arc().await;
                                if path == path_clone {
                                    let old_paths = journal_guard.as_ref().map(|j| j.includes().collect::<HashSet<_>>()).unwrap_or_default();
                                    let new_paths = parsed_journal.includes().collect::<HashSet<_>>();
                                    let to_watch = new_paths.difference(&old_paths).cloned().collect::<Vec<_>>();
                                    if !to_watch.is_empty() {
                                        watch_sender.send(WatcherTask::Watch(to_watch)).await.unwrap();
                                    }
                                    let to_unwatch = old_paths.difference(&new_paths).cloned().collect::<Vec<_>>();
                                    if !to_unwatch.is_empty() {
                                        watch_sender.send(WatcherTask::Unwatch(to_unwatch)).await.unwrap();
                                    }
                                }
                                if path == path_clone {
                                    *journal_guard = Some(parsed_journal);
                                } else if let Some(journal) = journal_guard.as_mut() {
                                    journal.merge(&parsed_journal);
                                }
                                let mut error_guard = error_clone.lock_arc().await;
                                error_guard.remove(&path_clone);
                            },
                            Err(error) => {
                                let mut error_guard = error_clone.lock_arc().await;
                                error_guard.insert(path_clone.clone(), error);
                            }
                        }
                    }
                    Err(error) => {
                        tracing::error!(?error, path = %path_clone.display(), "path receiver error");
                    }
                }
            }
        });

        paths_sender
            .send_blocking(path.to_path_buf())
            .expect("failed to send initial path");

        Self {
            _task: join3(debouncer_task, process_path_task, changed_paths_task),
            error,
            journal,
        }
    }

    #[must_use]
    pub fn journal(&self) -> MutexGuardArc<Option<hledger_journal::Journal>> {
        self.journal.lock_arc_blocking()
    }

    #[must_use]
    pub fn error(&self) -> MutexGuardArc<HashMap<std::path::PathBuf, hledger_journal::Error>> {
        self.error.lock_arc_blocking()
    }
}
