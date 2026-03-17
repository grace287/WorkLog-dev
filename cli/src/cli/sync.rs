use anyhow::Result;

use crate::core::config::WorklogPaths;
use crate::core::config::{load_links, load_tasks};
use crate::core::output::{print_info, print_timeline_entry, print_warn};

// ── sync ──────────────────────────────────────────────────────────────────────

pub async fn run_sync(
    since: Option<&str>,
    dry_run: bool,
    _no_verify: bool,
) -> Result<()> {
    if dry_run {
        print_info("Dry-run mode: no changes will be written");
    }

    if let Some(s) = since {
        print_info(&format!("Scanning commits since: {}", s));
    } else {
        print_info("Scanning commits (default: last 30 days)");
    }

    // Phase 2: git2 revwalk + GitHub verification will be wired in here.
    // connectors::git::parse_commits() and connectors::github::verify_commit()
    print_warn("sync is not yet fully implemented — coming in Phase 2 (SYNC-*)");
    Ok(())
}

// ── push ──────────────────────────────────────────────────────────────────────

pub async fn run_push() -> Result<()> {
    // Phase 2: sync HEAD commit immediately
    print_warn("push is not yet implemented — coming in Phase 2");
    Ok(())
}

// ── log ───────────────────────────────────────────────────────────────────────

pub fn run_log() -> Result<()> {
    let paths = WorklogPaths::new()?;
    let tasks = load_tasks(&paths)?;
    let links = load_links(&paths)?;

    if links.is_empty() {
        print_info("No commit links found — run `worklog sync` to populate.");
        return Ok(());
    }

    for link in &links {
        // Look up the task title for a richer display
        let title = tasks
            .iter()
            .find(|t| t.task_key.eq_ignore_ascii_case(&link.task_key))
            .map(|t| t.title.as_str())
            .unwrap_or("(unknown task)");

        print_timeline_entry(&link.task_key, &link.sha, title, link.verified);
    }

    Ok(())
}
