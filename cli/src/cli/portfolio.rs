use anyhow::{Context, Result};
use serde::Serialize;

use crate::cli::auth::load_pat;
use crate::core::config::{WorklogPaths, load_config, load_links, load_tasks};
use crate::core::http::{ApiClient, get_jwt_token};
use crate::core::model::TaskStatus;
use crate::core::output::{new_spinner, print_info, print_success, print_warn};

// ── publish 요청 DTO ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct CommitPayload {
    sha: String,
    message: String,
    committed_at: String,
    verified: bool,
}

#[derive(Serialize)]
struct TaskPayload {
    task_key: String,
    title: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    done_at: Option<String>,
    commits: Vec<CommitPayload>,
}

#[derive(Serialize)]
struct PublishRequest {
    project_id: String,
    tasks: Vec<TaskPayload>,
    visibility: String,
}

// ── publish ───────────────────────────────────────────────────────────────────

pub async fn run_publish(visibility: Option<&str>) -> Result<()> {
    let vis = visibility.unwrap_or("public");
    let paths = WorklogPaths::new()?;
    let config = load_config(&paths)?;

    let tasks = load_tasks(&paths)?;
    let links = load_links(&paths)?;

    if tasks.is_empty() {
        print_warn("No tasks found — add tasks first with `worklog task add`");
        return Ok(());
    }

    // PAT → JWT
    let pat = load_pat()?;
    let api_url = &config.api_url;

    let pb = new_spinner("Authenticating with worklog API…");
    let jwt = get_jwt_token(api_url, &pat)
        .await
        .context("Failed to obtain JWT token — check network and PAT")?;
    pb.finish_and_clear();

    // 태스크-커밋 링크 조합
    let project_id = config
        .default_project
        .clone()
        .unwrap_or_else(|| "PROJ".to_string());

    let task_payloads: Vec<TaskPayload> = tasks
        .iter()
        .map(|t| {
            let task_links: Vec<CommitPayload> = links
                .iter()
                .filter(|l| l.task_key.eq_ignore_ascii_case(&t.task_key))
                .map(|l| CommitPayload {
                    sha: l.sha.clone(),
                    message: String::new(), // commits.json에서 채울 수 있으나 선택적
                    committed_at: chrono::Utc::now().to_rfc3339(),
                    verified: l.verified,
                })
                .collect();

            TaskPayload {
                task_key: t.task_key.clone(),
                title: t.title.clone(),
                status: t.status.as_str().to_string(),
                done_at: t.done_at.map(|d| d.to_rfc3339()),
                commits: task_links,
            }
        })
        .collect();

    let request = PublishRequest {
        project_id: project_id.clone(),
        tasks: task_payloads,
        visibility: vis.to_string(),
    };

    // FastAPI POST
    let pb = new_spinner("Publishing portfolio…");
    let client = ApiClient::new(api_url)?.with_token(jwt);
    let resp = client
        .post("/api/v1/portfolios", &request)
        .await
        .context("Failed to reach worklog API")?;
    pb.finish_and_clear();

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Publish failed {}: {}", status, body);
    }

    let json: serde_json::Value = resp.json().await.context("Failed to parse publish response")?;
    let url = json["url"].as_str().unwrap_or("(unknown URL)");

    print_success(&format!("{} 공개됨", url));
    Ok(())
}

// ── export ────────────────────────────────────────────────────────────────────

pub fn run_export(format: Option<&str>) -> Result<()> {
    let fmt = format.unwrap_or("md");
    let paths = WorklogPaths::new()?;
    let tasks = load_tasks(&paths)?;
    let links = load_links(&paths)?;

    match fmt {
        "json" => {
            let out = serde_json::json!({ "tasks": tasks, "links": links });
            let filename = "worklog-export.json";
            std::fs::write(filename, serde_json::to_string_pretty(&out)?)?;
            print_success(&format!("Exported to {}", filename));
        }
        "md" => {
            let mut md = String::from("# Worklog Portfolio\n\n");
            for task in &tasks {
                let icon = match task.status {
                    TaskStatus::Done => "✅",
                    TaskStatus::Doing => "🔄",
                    TaskStatus::Todo => "⬜",
                };
                md.push_str(&format!("## {} {} — {}\n\n", icon, task.task_key, task.title));

                let task_links: Vec<_> = links
                    .iter()
                    .filter(|l| l.task_key.eq_ignore_ascii_case(&task.task_key))
                    .collect();

                for l in &task_links {
                    let badge = if l.verified { " ✓" } else { "" };
                    md.push_str(&format!("- `{}`{}\n", &l.sha[..l.sha.len().min(12)], badge));
                }
                md.push('\n');
            }
            let filename = "worklog-export.md";
            std::fs::write(filename, md)?;
            print_success(&format!("Exported to {}", filename));
        }
        other => anyhow::bail!("Unknown format '{}' — use: md | json", other),
    }

    Ok(())
}

// ── status ────────────────────────────────────────────────────────────────────

pub fn run_status() -> Result<()> {
    let paths = WorklogPaths::new()?;
    let config = load_config(&paths)?;
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

    let project = config.default_project.as_deref().unwrap_or("(no project)");

    println!(" {} · {}% complete", project, pct);
    println!(
        " Tasks: {} total  ({} todo · {} doing · {} done)",
        total, todo, doing, done
    );
    println!(" Evidence: {}  Verified: {}", evidence, verified);

    if unverified > 0 {
        print_info(&format!("{} unverified link(s)", unverified));
    }
    if !tasks.is_empty() && evidence == 0 {
        print_info("No linked commits — run `worklog sync` first.");
    }

    Ok(())
}
