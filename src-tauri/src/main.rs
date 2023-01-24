mod hledger;
use hledger::{HLParserError, Journal};
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
fn read_file(file_path: &str) -> Result<String, InvokeError> {
    let contents = fs::read_to_string(file_path);
    if contents.is_ok() {
        return Ok(contents.unwrap());
    } else {
        return Err(contents.err().unwrap().to_string().into());
    }
}

#[tauri::command]
fn parse_hledger_file(file_path: &str) -> Result<Journal, InvokeError> {
    let journal_path = PathBuf::from(file_path);
    let journal: Result<Journal, HLParserError> = journal_path.try_into();
    if journal.is_ok() {
        return Ok(journal.unwrap());
    } else {
        return Err(journal.err().unwrap().to_string().into());
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_ledger_file,
            read_file,
            parse_hledger_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
