use super::process;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to exec hledger-web: {0}")]
    Process(#[from] process::Error),
    #[error("failed to parse utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub async fn version() -> Result<String, Error> {
    let output = process::exec(&["--version"]).await?;
    let output = String::from_utf8(output)?;
    let output = output.trim_end().to_string();
    Ok(output)
}
