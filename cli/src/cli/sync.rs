use anyhow::{Context, Result};
use std::collections::HashSet;

use crate::cli::auth::load_pat;
use crate::connectors::{git, github};
use crate::core::config::{
    WorklogPaths, load_commits, load_config, load_links, load_tasks, save_commits, save_links,
};
use crate::core::model::{Commit, TaskCommitLink, extract_task_ids};
use crate::core::output::{
    new_spinner, print_info, print_success, print_timeline_entry, print_warn,
};

// ── sync ──────────────────────────────────────────────────────────────────────

pub async fn run_sync(since: Option<&str>, dry_run: bool, no_verify: bool) -> Result<()> {
    let paths = WorklogPaths::new()?;

    if dry_run {
        print_info("Dry-run mode: no changes will be written");
    }

    // ── 1. git2 revwalk ─────────────────────────────────────────────────────
    let cutoff_str = since.unwrap_or("30d");
    let pb = new_spinner(&format!("Scanning commits (since {})…", cutoff_str));

    let repo_path = std::env::current_dir().context("Failed to get current directory")?;
    let commits = git::parse_commits_since(&repo_path, cutoff_str)
        .context("Failed to parse git commits")?;

    pb.finish_and_clear();

    if commits.is_empty() {
        print_warn(&format!("No commits found in the last {}", cutoff_str));
        return Ok(());
    }
    print_info(&format!("{} commits scanned", commits.len()));

    // ── 2. 기존 데이터 로드 ──────────────────────────────────────────────────
    let tasks = load_tasks(&paths)?;
    let mut stored_links = load_links(&paths)?;
    let mut stored_commits = load_commits(&paths)?;

    let known_task_keys: HashSet<String> = tasks
        .iter()
        .map(|t| t.task_key.to_uppercase())
        .collect();

    let linked_shas: HashSet<String> = stored_links.iter().map(|l| l.sha.clone()).collect();
    let stored_shas: HashSet<String> = stored_commits.iter().map(|c| c.sha.clone()).collect();

    // ── 3. [TASK-ID] 추출 → TaskCommitLink 생성 ──────────────────────────────
    let mut new_commits: Vec<Commit> = Vec::new();
    let mut new_links: Vec<TaskCommitLink> = Vec::new();
    let mut unlinked_count = 0usize;

    for commit in &commits {
        // 신규 커밋만 저장 목록에 추가
        if !stored_shas.contains(&commit.sha) {
            new_commits.push(commit.clone());
        }

        let ids = extract_task_ids(&commit.message);
        let matched: Vec<String> = ids
            .into_iter()
            .filter(|id| known_task_keys.contains(id))
            .collect();

        if matched.is_empty() {
            unlinked_count += 1;
        } else {
            for task_key in matched {
                // 중복 링크 방지
                if !linked_shas.contains(&commit.sha) {
                    new_links.push(TaskCommitLink::new(task_key, commit.sha.clone()));
                }
            }
        }
    }

    print_info(&format!("{} new evidence(s) found", new_links.len()));

    // ── 4. GitHub 검증 ───────────────────────────────────────────────────────
    if !no_verify && !new_links.is_empty() {
        match load_pat() {
            Err(_) => {
                print_warn("Not authenticated — skipping GitHub verification (run `worklog init`)");
            }
            Ok(pat) => {
                let owner_repo = git::get_owner_repo(&repo_path).unwrap_or_default();
                if owner_repo.is_empty() {
                    print_warn("Cannot detect GitHub remote — skipping verification");
                } else {
                    let pb = new_spinner("Verifying commits with GitHub…");
                    let shas: Vec<String> = new_links.iter().map(|l| l.sha.clone()).collect();
                    let results = github::verify_commits_batch(&pat, &owner_repo, &shas).await;
                    pb.finish_and_clear();

                    let mut verified_count = 0usize;
                    for (link, (_, verified)) in new_links.iter_mut().zip(results.iter()) {
                        link.verified = *verified;
                        if *verified {
                            verified_count += 1;
                        }
                    }
                    print_success(&format!(
                        "{}/{} commits verified",
                        verified_count,
                        new_links.len()
                    ));
                }
            }
        }
    } else if no_verify {
        print_info("Skipping GitHub verification (--no-verify)");
    }

    // ── 5. 저장 ─────────────────────────────────────────────────────────────
    if !dry_run {
        stored_commits.extend(new_commits);
        save_commits(&paths, &stored_commits)?;

        let saved = new_links.len();
        stored_links.extend(new_links);
        save_links(&paths, &stored_links)?;

        if saved > 0 {
            print_success(&format!("{} link(s) saved to links.json", saved));
        }
    }

    // ── 6. 미연결 커밋 리포트 ────────────────────────────────────────────────
    if unlinked_count > 0 {
        print_warn(&format!(
            "{} unlinked commit(s) — add [TASK-KEY] to messages or use `worklog task link`",
            unlinked_count
        ));
    }

    Ok(())
}

// ── push ──────────────────────────────────────────────────────────────────────

/// HEAD 커밋 하나만 즉시 sync한다.
pub async fn run_push() -> Result<()> {
    let paths = WorklogPaths::new()?;
    let repo_path = std::env::current_dir().context("Failed to get current directory")?;

    // HEAD 커밋만 가져오기 (since=1d는 넉넉히 당일 커밋 포함)
    let commits = git::parse_commits_since(&repo_path, "1d")
        .context("Failed to parse git commits")?;

    let head = match commits.first() {
        Some(c) => c.clone(),
        None => {
            print_warn("No commits found in the last 24h");
            return Ok(());
        }
    };

    print_info(&format!(
        "HEAD: {} {}",
        &head.sha[..12],
        head.message.lines().next().unwrap_or("")
    ));

    // TASK-ID 추출
    let tasks = load_tasks(&paths)?;
    let known_keys: HashSet<String> =
        tasks.iter().map(|t| t.task_key.to_uppercase()).collect();

    let ids = extract_task_ids(&head.message);
    let matched: Vec<String> = ids
        .into_iter()
        .filter(|id| known_keys.contains(id))
        .collect();

    if matched.is_empty() {
        print_warn("No TASK-ID found in HEAD commit message");
        return Ok(());
    }

    // 검증
    let mut link = TaskCommitLink::new(matched[0].clone(), head.sha.clone());
    if let Ok(pat) = load_pat() {
        if let Some(owner_repo) = git::get_owner_repo(&repo_path) {
            match github::verify_commit(&pat, &owner_repo, &head.sha).await {
                Ok(v) => link.verified = v,
                Err(e) => print_warn(&format!("Verification failed: {}", e)),
            }
        }
    }

    // 저장
    let mut links = load_links(&paths)?;
    let already = links
        .iter()
        .any(|l| l.task_key == link.task_key && l.sha == link.sha);

    if !already {
        links.push(link.clone());
        save_links(&paths, &links)?;
    }

    let config = load_config(&paths)?;
    if !already {
        print_success(&format!(
            "{} ↔ {} {}",
            link.task_key,
            &head.sha[..12],
            if link.verified { "[verified]" } else { "" }
        ));
    } else {
        print_info("Already linked — no changes");
    }

    let _ = config; // suppress unused warning
    Ok(())
}

// ── log ───────────────────────────────────────────────────────────────────────

pub fn run_log() -> Result<()> {
    let paths = WorklogPaths::new()?;
    let tasks = load_tasks(&paths)?;
    let links = load_links(&paths)?;
    let commits = load_commits(&paths)?;

    if links.is_empty() {
        print_info("No commit links found — run `worklog sync` to populate.");
        return Ok(());
    }

    for link in &links {
        let title = tasks
            .iter()
            .find(|t| t.task_key.eq_ignore_ascii_case(&link.task_key))
            .map(|t| t.title.as_str())
            .unwrap_or("(unknown task)");

        // 커밋 메시지 첫 줄 가져오기
        let msg = commits
            .iter()
            .find(|c| c.sha == link.sha)
            .map(|c| c.message.lines().next().unwrap_or("").to_string())
            .unwrap_or_else(|| title.to_string());

        print_timeline_entry(&link.task_key, &link.sha, &msg, link.verified);
    }

    Ok(())
}
