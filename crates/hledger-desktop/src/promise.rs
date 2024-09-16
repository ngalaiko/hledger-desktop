pub struct Promise<T: Send + 'static>(Option<poll_promise::Promise<T>>);

impl<T: Send + 'static> Default for Promise<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<'de, T: Send + 'static> serde::Deserialize<'de> for Promise<T> {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::default())
    }
}

impl<T: Send + 'static> Promise<T> {
    pub fn from_ready(value: T) -> Self {
        Self(Some(poll_promise::Promise::from_ready(value)))
    }

    pub fn spawn_async(future: impl std::future::Future<Output = T> + 'static + Send) -> Self {
        Self(Some(poll_promise::Promise::spawn_async(future)))
    }

    pub fn spawn_blocking<F>(f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        Self(Some(poll_promise::Promise::spawn_blocking(f)))
    }
}

impl<T: Send + 'static> Promise<T> {
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn ready(&self) -> Option<&T> {
        self.0.as_ref().and_then(|promise| promise.ready())
    }
}
