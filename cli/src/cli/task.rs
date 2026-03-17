use anyhow::{Context, Result, bail};
use chrono::Utc;

use crate::core::config::{
    WorklogPaths, load_config, load_links, load_tasks, save_links, save_tasks,
};
use crate::core::model::{Task, TaskCommitLink, TaskStatus};
use crate::core::output::{print_success, print_task_table, print_warn};

// ── add ───────────────────────────────────────────────────────────────────────

pub fn run_add(title: &str, project_arg: Option<&str>) -> Result<()> {
    let paths = WorklogPaths::new()?;
    let config = load_config(&paths)?;

    let project_id = project_arg
        .map(str::to_string)
        .or(config.default_project.clone())
        .unwrap_or_else(|| "PROJ".to_string());

    let mut tasks = load_tasks(&paths)?;

    // Determine next ID within this project
    let next_n = tasks
        .iter()
        .filter(|t| t.project_id == project_id)
        .map(|t| t.id)
        .max()
        .map(|m| m + 1)
        .unwrap_or(1);

    let task_key = format!("{}-{}", project_id, next_n);
    let task = Task::new(next_n, task_key.clone(), title.to_string(), project_id);
    tasks.push(task);
    save_tasks(&paths, &tasks)?;

    print_success(&format!("Task added: {}", task_key));
    Ok(())
}

// ── ls ────────────────────────────────────────────────────────────────────────

pub fn run_ls(project_filter: Option<&str>) -> Result<()> {
    let paths = WorklogPaths::new()?;
    let config = load_config(&paths)?;

    let effective_filter = project_filter
        .map(str::to_string)
        .or(config.default_project.clone());

    let tasks = load_tasks(&paths)?;

    let filtered: Vec<Task> = tasks
        .into_iter()
        .filter(|t| {
            effective_filter
                .as_deref()
                .map(|p| t.project_id == p)
                .unwrap_or(true)
        })
        .collect();

    print_task_table(&filtered);
    Ok(())
}

// ── done ──────────────────────────────────────────────────────────────────────

pub fn run_done(task_key: &str) -> Result<()> {
    let paths = WorklogPaths::new()?;
    let mut tasks = load_tasks(&paths)?;

    let task = tasks
        .iter_mut()
        .find(|t| t.task_key.eq_ignore_ascii_case(task_key))
        .with_context(|| format!("Task '{}' not found", task_key))?;

    if task.status == TaskStatus::Done {
        print_warn(&format!("{} is already done", task_key));
        return Ok(());
    }

    task.status = TaskStatus::Done;
    task.done_at = Some(Utc::now());
    save_tasks(&paths, &tasks)?;

    print_success(&format!("{} marked as done", task_key));
    Ok(())
}

// ── move ──────────────────────────────────────────────────────────────────────

pub fn run_move(task_key: &str, status_str: &str) -> Result<()> {
    let new_status = TaskStatus::from_str(status_str)
        .with_context(|| format!("Invalid status '{}' — use: todo | doing | done", status_str))?;

    let paths = WorklogPaths::new()?;
    let mut tasks = load_tasks(&paths)?;

    let task = tasks
        .iter_mut()
        .find(|t| t.task_key.eq_ignore_ascii_case(task_key))
        .with_context(|| format!("Task '{}' not found", task_key))?;

    if new_status == TaskStatus::Done && task.done_at.is_none() {
        task.done_at = Some(Utc::now());
    } else if new_status != TaskStatus::Done {
        task.done_at = None;
    }

    task.status = new_status.clone();
    save_tasks(&paths, &tasks)?;

    print_success(&format!("{} moved to {}", task_key, new_status.as_str()));
    Ok(())
}

// ── link ──────────────────────────────────────────────────────────────────────

pub fn run_link(task_key: &str, sha: &str) -> Result<()> {
    let paths = WorklogPaths::new()?;

    // Validate task exists
    let tasks = load_tasks(&paths)?;
    if !tasks.iter().any(|t| t.task_key.eq_ignore_ascii_case(task_key)) {
        bail!("Task '{}' not found — create it first with `worklog task add`", task_key);
    }

    let mut links = load_links(&paths)?;

    // Deduplicate
    if links
        .iter()
        .any(|l| l.task_key.eq_ignore_ascii_case(task_key) && l.sha == sha)
    {
        print_warn(&format!("{} ↔ {} already linked", task_key, sha));
        return Ok(());
    }

    links.push(TaskCommitLink::new(task_key.to_uppercase(), sha.to_string()));
    save_links(&paths, &links)?;

    print_success(&format!("{} linked to commit {}", task_key, &sha[..sha.len().min(12)]));
    Ok(())
}
