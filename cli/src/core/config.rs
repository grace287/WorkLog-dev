use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::core::model::{Commit, Config, Task, TaskCommitLink};

pub struct WorklogPaths {
    pub base_dir: PathBuf,
    pub config_file: PathBuf,
    pub tasks_file: PathBuf,
    pub commits_file: PathBuf,
    pub links_file: PathBuf,
}

impl WorklogPaths {
    pub fn new() -> Result<Self> {
        let base_dir = dirs::data_local_dir()
            .context("Failed to resolve local data directory")?
            .join("worklog");

        Ok(WorklogPaths {
            config_file: base_dir.join("config.toml"),
            tasks_file: base_dir.join("tasks.json"),
            commits_file: base_dir.join("commits.json"),
            links_file: base_dir.join("links.json"),
            base_dir,
        })
    }
}

pub fn ensure_dirs(paths: &WorklogPaths) -> Result<()> {
    std::fs::create_dir_all(&paths.base_dir)
        .with_context(|| format!("Failed to create worklog dir: {}", paths.base_dir.display()))?;
    Ok(())
}

// ── Config ────────────────────────────────────────────────────────────────────

pub fn load_config(paths: &WorklogPaths) -> Result<Config> {
    if !paths.config_file.exists() {
        return Ok(Config::default());
    }
    let raw = std::fs::read_to_string(&paths.config_file)
        .with_context(|| format!("Failed to read config: {}", paths.config_file.display()))?;
    let config: Config = toml::from_str(&raw).context("Failed to parse config.toml")?;
    Ok(config)
}

pub fn save_config(paths: &WorklogPaths, config: &Config) -> Result<()> {
    ensure_dirs(paths)?;
    let raw = toml::to_string_pretty(config).context("Failed to serialize config")?;
    std::fs::write(&paths.config_file, raw)
        .with_context(|| format!("Failed to write config: {}", paths.config_file.display()))?;
    Ok(())
}

// ── Tasks ─────────────────────────────────────────────────────────────────────

pub fn load_tasks(paths: &WorklogPaths) -> Result<Vec<Task>> {
    if !paths.tasks_file.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&paths.tasks_file)
        .with_context(|| format!("Failed to read tasks: {}", paths.tasks_file.display()))?;
    let tasks: Vec<Task> = serde_json::from_str(&raw).context("Failed to parse tasks.json")?;
    Ok(tasks)
}

pub fn save_tasks(paths: &WorklogPaths, tasks: &[Task]) -> Result<()> {
    ensure_dirs(paths)?;
    let raw = serde_json::to_string_pretty(tasks).context("Failed to serialize tasks")?;
    std::fs::write(&paths.tasks_file, raw)
        .with_context(|| format!("Failed to write tasks: {}", paths.tasks_file.display()))?;
    Ok(())
}

// ── Commits ───────────────────────────────────────────────────────────────────

pub fn load_commits(paths: &WorklogPaths) -> Result<Vec<Commit>> {
    if !paths.commits_file.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&paths.commits_file)
        .with_context(|| format!("Failed to read commits: {}", paths.commits_file.display()))?;
    let commits: Vec<Commit> =
        serde_json::from_str(&raw).context("Failed to parse commits.json")?;
    Ok(commits)
}

pub fn save_commits(paths: &WorklogPaths, commits: &[Commit]) -> Result<()> {
    ensure_dirs(paths)?;
    let raw = serde_json::to_string_pretty(commits).context("Failed to serialize commits")?;
    std::fs::write(&paths.commits_file, raw)
        .with_context(|| format!("Failed to write commits: {}", paths.commits_file.display()))?;
    Ok(())
}

// ── Links ─────────────────────────────────────────────────────────────────────

pub fn load_links(paths: &WorklogPaths) -> Result<Vec<TaskCommitLink>> {
    if !paths.links_file.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&paths.links_file)
        .with_context(|| format!("Failed to read links: {}", paths.links_file.display()))?;
    let links: Vec<TaskCommitLink> =
        serde_json::from_str(&raw).context("Failed to parse links.json")?;
    Ok(links)
}

pub fn save_links(paths: &WorklogPaths, links: &[TaskCommitLink]) -> Result<()> {
    ensure_dirs(paths)?;
    let raw = serde_json::to_string_pretty(links).context("Failed to serialize links")?;
    std::fs::write(&paths.links_file, raw)
        .with_context(|| format!("Failed to write links: {}", paths.links_file.display()))?;
    Ok(())
}

// ── Init data files ───────────────────────────────────────────────────────────

/// Create empty JSON data files if they don't exist yet.
pub fn init_data_files(paths: &WorklogPaths) -> Result<()> {
    ensure_dirs(paths)?;

    if !paths.tasks_file.exists() {
        std::fs::write(&paths.tasks_file, "[]")?;
    }
    if !paths.commits_file.exists() {
        std::fs::write(&paths.commits_file, "[]")?;
    }
    if !paths.links_file.exists() {
        std::fs::write(&paths.links_file, "[]")?;
    }
    Ok(())
}
