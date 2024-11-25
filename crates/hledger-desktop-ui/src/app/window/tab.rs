pub struct State {
    pub file_path: std::path::PathBuf,
}

impl State {
    #[must_use]
    pub fn name(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
    }

    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        Self {
            file_path: path.as_ref().to_path_buf(),
        }
    }
}
