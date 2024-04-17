use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;

use super::{Client, Error};

#[derive(Clone, Default)]
pub struct Manager {
    clients: Arc<Mutex<HashMap<String, Client>>>,
}

impl Manager {
    pub async fn client<P: AsRef<std::path::Path>>(&self, path: P) -> Result<Client, Error> {
        let path = path.as_ref().display().to_string();
        let mut clients = self.clients.lock().await;
        let existing = clients.entry(path.clone());
        if let std::collections::hash_map::Entry::Occupied(entry) = existing {
            Ok(entry.get().clone())
        } else {
            let new_client = Client::new(path.clone()).await?;
            existing.or_insert(new_client.clone());
            Ok(new_client)
        }
    }

    pub async fn shutdown(&self) {
        self.clients.lock().await.clear();
    }
}
