use std::path;

use tauri::AppHandle;
use tokio::sync::Mutex;

use super::{Client, Error};

use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct Manager {
    handle: AppHandle,
    clients: Arc<Mutex<HashMap<String, Client>>>,
}

impl Manager {
    pub fn new(handle: &AppHandle) -> Self {
        Self {
            handle: handle.clone(),
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn client<P: AsRef<path::Path>>(&self, path: P) -> Result<Client, Error> {
        let path = path.as_ref().display().to_string();
        let mut clients = self.clients.lock().await;
        let existing = clients.entry(path.clone());
        if let std::collections::hash_map::Entry::Occupied(entry) = existing {
            Ok(entry.get().clone())
        } else {
            let new_client = Client::new(&self.handle, path.clone()).await?;
            let new_client = new_client.clone();
            existing.or_insert(new_client.clone());
            Ok(new_client)
        }
    }
}
