fn setup_binaries() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs, path};
    use which::which;

    let triple = env::var("TARGET")?;
    let binaries_path = path::Path::new("binaries");
    let hledger_web_path = binaries_path.join(format!("hledger-web-{}", triple));
    if hledger_web_path.exists() {
        return Ok(());
    }

    if let Ok(installed_hledger_web) = which("hledger-web") {
        fs::copy(installed_hledger_web, hledger_web_path)?;
        Ok(())
    } else {
        Err("hledger-web not found. follow https://hledger.org/install.html to install it".into())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_binaries()?;
    tauri_build::build();
    Ok(())
}
