use rand::Rng;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::api::process::{Command, CommandChild, CommandEvent};
use tauri::InvokeError;

/// Children contains a map of filenames to running hledger-web processes.
#[derive(Default)]
struct Children(Mutex<HashMap<String, CommandChild>>);

/// Endpoints contains a map of filenames to running hledger-web endpoints.
#[derive(Default)]
struct Endpoints(Mutex<HashMap<String, String>>);

#[tauri::command]
async fn hledger_web(
    file_path: &str,
    cors: &str,
    state: tauri::State<'_, Children>,
    endpoints: tauri::State<'_, Endpoints>,
) -> Result<String, InvokeError> {
    if endpoints
        .0
        .lock()
        .unwrap()
        .get(&file_path.to_string())
        .is_some()
    {
        return Ok(endpoints
            .0
            .lock()
            .unwrap()
            .get(&file_path.to_string())
            .unwrap()
            .to_string());
    }

    let port = rand::thread_rng().gen_range(32768..65536);
    let args = [
        "--file".to_string(),
        file_path.to_string(),
        "--port".to_string(),
        port.to_string(),
        "--cors".to_string(),
        cors.to_string(),
        "--serve-api".to_string(),
    ];

    let (mut rx, child) = Command::new_sidecar("hledger-web")
        .expect("failed to create `binaries/hledger-web` binary command")
        .args(args)
        .spawn()
        .expect("failed to spawn `hledger-web`");

    while let Some(event) = rx.recv().await {
        if let CommandEvent::Stdout(line) = event {
            if line == "Press ctrl-c to quit".to_string() {
                state.0.lock().unwrap().insert(file_path.to_string(), child);
                endpoints
                    .0
                    .lock()
                    .unwrap()
                    .insert(file_path.to_string(), format!("http://127.0.0.1:{}/", port));
                return Ok(format!("http://127.0.0.1:{}/", port));
            }
        }
    }

    return Err("something went wrong".into());
}

#[tauri::command]
fn get_ledger_file() -> Option<String> {
    let ledger_file = std::env::var("LEDGER_FILE");
    if ledger_file.is_ok() {
        return Some(ledger_file.unwrap().to_string());
    } else {
        return None;
    }
}

#[tauri::command]
fn resolve_glob_pattern(pattern: &str) -> Result<Vec<PathBuf>, InvokeError> {
    let paths = glob::glob(pattern);
    if paths.is_ok() {
        return Ok(paths
            .unwrap()
            .filter_map(|path| path.ok())
            .map(|path| path.to_path_buf())
            .collect::<Vec<PathBuf>>());
    } else {
        return Err("invalid glob pattern".into());
    }
}

#[tauri::command]
fn read_file(file_path: &str) -> Result<String, InvokeError> {
    let contents = fs::read_to_string(file_path);
    if contents.is_ok() {
        return Ok(contents.unwrap());
    } else {
        return Err(contents.err().unwrap().to_string().into());
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Children(Default::default()))
        .manage(Endpoints(Default::default()))
        .invoke_handler(tauri::generate_handler![
            get_ledger_file,
            read_file,
            resolve_glob_pattern,
            hledger_web,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
