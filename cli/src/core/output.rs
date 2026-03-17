use colored::Colorize;

use crate::core::model::{Task, TaskCommitLink, TaskStatus};

// ── Basic message helpers ──────────────────────────────────────────────────────

pub fn print_success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
}

pub fn print_warn(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", "·".cyan(), msg);
}

// ── Task table ────────────────────────────────────────────────────────────────

pub fn print_task_table(tasks: &[Task]) {
    if tasks.is_empty() {
        print_info("No tasks found.");
        return;
    }

    // Compute dynamic column widths
    let key_w = tasks
        .iter()
        .map(|t| t.task_key.len())
        .max()
        .unwrap_or(8)
        .max(8);

    let status_w = 7usize; // "doing" is the widest label we display

    println!(
        " {:<key_w$}  {:<status_w$}  {}",
        "TASK".bold().underline(),
        "STATUS".bold().underline(),
        "TITLE".bold().underline(),
        key_w = key_w,
        status_w = status_w,
    );

    for task in tasks {
        let (icon, status_str) = match task.status {
            TaskStatus::Todo => (
                "○".white().to_string(),
                "todo".white().to_string(),
            ),
            TaskStatus::Doing => (
                "●".yellow().to_string(),
                "doing".yellow().to_string(),
            ),
            TaskStatus::Done => (
                "✓".green().to_string(),
                "done".green().to_string(),
            ),
        };

        println!(
            " {:<key_w$}  {} {:<status_w$}  {}",
            task.task_key.cyan().bold(),
            icon,
            status_str,
            task.title,
            key_w = key_w,
            // icon takes 1 visual char + 1 space = 2 chars already printed
            status_w = status_w.saturating_sub(2),
        );
    }
}

// ── Link table ────────────────────────────────────────────────────────────────

pub fn print_link_table(links: &[TaskCommitLink]) {
    if links.is_empty() {
        print_info("No links found.");
        return;
    }

    println!(
        " {:<12}  {:<42}  {}",
        "TASK".bold().underline(),
        "SHA".bold().underline(),
        "VERIFIED".bold().underline(),
    );

    for link in links {
        let verified_str = if link.verified {
            "✓ verified".green().to_string()
        } else {
            "· unverified".white().dimmed().to_string()
        };
        println!(
            " {:<12}  {:<42}  {}",
            link.task_key.cyan(),
            link.sha,
            verified_str
        );
    }
}

// ── Commit timeline ───────────────────────────────────────────────────────────

/// Prints a condensed task-commit timeline entry.
/// Used by `worklog log` (Phase 2 will fill this in with real data).
pub fn print_timeline_entry(task_key: &str, sha: &str, message: &str, verified: bool) {
    let verified_badge = if verified {
        " [verified]".green().to_string()
    } else {
        String::new()
    };

    println!(
        " {} {} {}{}",
        task_key.cyan().bold(),
        sha[..sha.len().min(12)].white().dimmed(),
        message,
        verified_badge,
    );
}

// ── Spinner helper (wraps indicatif) ─────────────────────────────────────────

/// Create a spinner with a standard style for long-running operations.
pub fn new_spinner(msg: &str) -> indicatif::ProgressBar {
    use indicatif::{ProgressBar, ProgressStyle};

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}
