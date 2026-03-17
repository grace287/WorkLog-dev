use anyhow::Result;

use crate::core::config::{WorklogPaths, load_links, load_tasks};
use crate::core::model::TaskStatus;
use crate::core::output::{new_spinner, print_info, print_warn};

// ── publish ───────────────────────────────────────────────────────────────────

pub async fn run_publish(visibility: Option<&str>) -> Result<()> {
    let vis = visibility.unwrap_or("public");
    let pb = new_spinner(&format!("Generating {} portfolio…", vis));

    // Phase 2: obtain JWT → POST /api/v1/portfolios
    // For now, simulate with a short delay and show a stub URL.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    pb.finish_and_clear();

    print_warn("publish is not yet fully implemented — coming in Phase 2 (API-*)");
    print_info("Future URL: https://worklog.dev/p/<your-slug>");
    Ok(())
}

// ── export ────────────────────────────────────────────────────────────────────

pub fn run_export(format: Option<&str>) -> Result<()> {
    let fmt = format.unwrap_or("md");
    match fmt {
        "md" | "json" => {
            print_warn(&format!(
                "export --format {} is not yet implemented — coming in Phase 2",
                fmt
            ));
        }
        other => {
            anyhow::bail!(
                "Unknown export format '{}' — supported: md, json",
                other
            );
        }
    }
    Ok(())
}

// ── status ────────────────────────────────────────────────────────────────────

pub fn run_status() -> Result<()> {
    let paths = WorklogPaths::new()?;
    let tasks = load_tasks(&paths)?;
    let links = load_links(&paths)?;

    let total = tasks.len();
    let done = tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
    let doing = tasks.iter().filter(|t| t.status == TaskStatus::Doing).count();
    let todo = tasks.iter().filter(|t| t.status == TaskStatus::Todo).count();

    let evidence = links.len();
    let verified = links.iter().filter(|l| l.verified).count();
    let unverified = evidence - verified;

    let pct = if total > 0 { done * 100 / total } else { 0 };

    // Determine the active project for the sprint line
    use crate::core::config::load_config;
    let config = load_config(&paths)?;
    let project = config
        .default_project
        .as_deref()
        .unwrap_or("(no project set)");

    println!(
        " {} · {}% complete",
        project,
        pct
    );
    println!(
        " Tasks: {} total  ({} todo · {} doing · {} done)",
        total, todo, doing, done
    );
    println!(
        " Evidence: {}  Verified: {}",
        evidence, verified
    );

    if unverified > 0 {
        print_info(&format!("{} unverified link(s)", unverified));
    }

    if !tasks.is_empty() && evidence == 0 {
        print_info("No linked commits yet — run `worklog sync` to scan git history.");
    }

    Ok(())
}
