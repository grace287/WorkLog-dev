use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

// ── TASK-ID regex (statically cached) ─────────────────────────────────────────
// Matches patterns like WLOG-1, PROJ-42, SYNC-100 (upper-case letters, dash, digits)

pub static TASK_ID_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b([A-Z][A-Z0-9]*-\d+)\b").expect("Invalid TASK_ID_RE"));

/// Extract all TASK-ID tokens from a commit message.
pub fn extract_task_ids(message: &str) -> Vec<String> {
    TASK_ID_RE
        .find_iter(message)
        .map(|m| m.as_str().to_string())
        .collect()
}

// ── TaskStatus ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Todo,
    Doing,
    Done,
}

impl TaskStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "todo" => Some(TaskStatus::Todo),
            "doing" => Some(TaskStatus::Doing),
            "done" => Some(TaskStatus::Done),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "todo",
            TaskStatus::Doing => "doing",
            TaskStatus::Done => "done",
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ── Task ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub task_key: String,
    pub title: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_at: Option<DateTime<Utc>>,
    pub project_id: String,
}

impl Task {
    pub fn new(id: u64, task_key: String, title: String, project_id: String) -> Self {
        Task {
            id,
            task_key,
            title,
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            done_at: None,
            project_id,
        }
    }
}

// ── Commit ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub message: String,
    pub repo: String,
    pub committed_at: DateTime<Utc>,
    /// Set to true once GitHub /commits/{sha} verification has confirmed the signature.
    #[serde(default)]
    pub verified: bool,
}

// ── TaskCommitLink ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCommitLink {
    pub task_key: String,
    pub sha: String,
    /// Mirror of Commit::verified at the time the link was written.
    #[serde(default)]
    pub verified: bool,
}

impl TaskCommitLink {
    pub fn new(task_key: String, sha: String) -> Self {
        TaskCommitLink {
            task_key,
            sha,
            verified: false,
        }
    }
}

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_login: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_project: Option<String>,
    pub api_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            github_login: None,
            default_project: None,
            api_url: "https://api.worklog.dev".to_string(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_single_task_id() {
        let ids = extract_task_ids("fix crash [WLOG-12]");
        assert_eq!(ids, vec!["WLOG-12"]);
    }

    #[test]
    fn extract_multiple_task_ids() {
        let ids = extract_task_ids("feat: sync SYNC-3 and API-7 together");
        assert_eq!(ids, vec!["SYNC-3", "API-7"]);
    }

    #[test]
    fn extract_no_task_id() {
        let ids = extract_task_ids("chore: update README");
        assert!(ids.is_empty());
    }

    #[test]
    fn task_status_round_trip() {
        assert_eq!(TaskStatus::from_str("doing"), Some(TaskStatus::Doing));
        assert_eq!(TaskStatus::from_str("DONE"), Some(TaskStatus::Done));
        assert_eq!(TaskStatus::from_str("bogus"), None);
    }
}
