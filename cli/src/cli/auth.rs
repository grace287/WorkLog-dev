use anyhow::{Context, Result, bail};

use crate::core::config::{WorklogPaths, init_data_files, load_config, save_config};
use crate::core::http::get_github_user;
use crate::core::output::{print_error, print_info, print_success};

const KEYRING_SERVICE: &str = "worklog";
const KEYRING_USER: &str = "github_pat";

fn get_keyring_entry() -> Result<keyring::Entry> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .context("Failed to access OS keyring")
}

/// OS keyring에서 저장된 PAT를 로드한다.
/// sync, publish 등 인증이 필요한 커맨드에서 공유 사용.
pub fn load_pat() -> Result<String> {
    get_keyring_entry()?
        .get_password()
        .context("No credentials found — run `worklog init` first")
}

pub async fn run_init() -> Result<()> {
    print_info("Enter your GitHub Personal Access Token (PAT):");
    print_info("  Required scopes: read:user, repo (for private repos)");

    let pat = read_secret_line("GitHub PAT: ")?;
    if pat.trim().is_empty() {
        bail!("PAT cannot be empty");
    }
    let pat = pat.trim().to_string();

    print_info("Verifying PAT with GitHub...");
    let login = get_github_user(&pat)
        .await
        .context("PAT verification failed — check your token and network connection")?;

    // Save to keyring
    let entry = get_keyring_entry()?;
    entry
        .set_password(&pat)
        .context("Failed to save PAT to OS keyring")?;

    // Save config
    let paths = WorklogPaths::new()?;
    let mut config = load_config(&paths)?;
    config.github_login = Some(login.clone());

    save_config(&paths, &config)?;
    init_data_files(&paths)?;

    print_success(&format!(
        "Authenticated as {} — config saved to {}",
        login,
        paths.config_file.display()
    ));
    print_info(&format!("Data directory: {}", paths.base_dir.display()));
    Ok(())
}

pub async fn run_whoami() -> Result<()> {
    let entry = get_keyring_entry()?;
    let pat = entry.get_password().context(
        "No credentials found — run `worklog init` first",
    )?;

    let login = get_github_user(&pat)
        .await
        .context("Failed to reach GitHub — check your network")?;

    println!("{}", login);
    Ok(())
}

pub fn run_logout() -> Result<()> {
    let entry = get_keyring_entry()?;
    match entry.delete_password() {
        Ok(()) => {
            print_success("Credentials removed from OS keyring");
        }
        Err(keyring::Error::NoEntry) => {
            print_error("No credentials found — already logged out");
        }
        Err(e) => {
            return Err(anyhow::anyhow!(e).context("Failed to delete password from keyring"));
        }
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Read a line from stdin. On TTY we could use rpassword, but for simplicity we
/// use a plain stdin read (the PAT will be visible in terminal — acceptable for
/// a dev-tool CLI running locally).
fn read_secret_line(prompt: &str) -> Result<String> {
    use std::io::{self, Write};
    print!("{}", prompt);
    io::stdout().flush().ok();
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .context("Failed to read input")?;
    Ok(buf.trim_end_matches('\n').trim_end_matches('\r').to_string())
}
