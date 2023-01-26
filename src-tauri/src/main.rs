use std::fs;
use std::path::PathBuf;
use tauri::InvokeError;

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
        return Ok(paths.unwrap()
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
        .invoke_handler(tauri::generate_handler![
            get_ledger_file,
            read_file,
            resolve_glob_pattern,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
