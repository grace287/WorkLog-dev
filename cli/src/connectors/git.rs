use anyhow::{Context, Result};
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use git2::{Repository, Sort};

use crate::core::model::Commit;

// ── since 파싱 ──────────────────────────────────────────────────────────────

/// "--since" 인자를 DateTime<Utc>로 변환한다.
/// - "30" 또는 "30d" → 지금부터 30일 전
/// - "2024-01-01"    → 해당 날짜 자정 (UTC)
/// - 파싱 실패 시 None 반환 (호출자가 기본값 30일 사용)
pub fn parse_since(since: &str) -> Option<DateTime<Utc>> {
    let days_str = since.trim_end_matches('d');
    if let Ok(days) = days_str.parse::<i64>() {
        return Some(Utc::now() - Duration::days(days));
    }
    if let Ok(date) = NaiveDate::parse_from_str(since, "%Y-%m-%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|ndt| Utc.from_utc_datetime(&ndt));
    }
    None
}

// ── revwalk ─────────────────────────────────────────────────────────────────

/// `start_path` 이상의 git 저장소를 찾아 `cutoff` 이후 커밋을 파싱한다.
pub fn parse_commits(start_path: &std::path::Path, cutoff: DateTime<Utc>) -> Result<Vec<Commit>> {
    let repo = Repository::discover(start_path)
        .context("Not inside a git repository — run from your project root")?;

    let repo_name = extract_owner_repo(&repo).unwrap_or_else(|| {
        repo.path()
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    let mut revwalk = repo.revwalk().context("Failed to create revwalk")?;
    revwalk
        .push_head()
        .context("Failed to push HEAD — is the repo empty?")?;
    revwalk
        .set_sorting(Sort::TIME)
        .context("Failed to set revwalk sort")?;

    let cutoff_secs = cutoff.timestamp();
    let mut commits = Vec::new();

    for oid_result in revwalk {
        let oid = oid_result.context("Failed to iterate revwalk")?;
        let commit = repo
            .find_commit(oid)
            .with_context(|| format!("Failed to find commit {}", oid))?;

        let committed_secs = commit.time().seconds();
        if committed_secs < cutoff_secs {
            // revwalk은 역시간순 — 이후는 모두 cutoff 이전
            break;
        }

        let committed_at = Utc
            .timestamp_opt(committed_secs, 0)
            .single()
            .unwrap_or_else(Utc::now);

        commits.push(Commit {
            sha: oid.to_string(),
            message: commit.message().unwrap_or("").trim().to_string(),
            repo: repo_name.clone(),
            committed_at,
            verified: false,
        });
    }

    Ok(commits)
}

/// `since` 문자열을 파싱해 `parse_commits`를 호출한다. (편의 래퍼)
pub fn parse_commits_since(repo_path: &std::path::Path, since: &str) -> Result<Vec<Commit>> {
    let cutoff = parse_since(since).unwrap_or_else(|| Utc::now() - Duration::days(30));
    parse_commits(repo_path, cutoff)
}

// ── remote URL 파싱 ─────────────────────────────────────────────────────────

/// 저장소 origin 리모트에서 "owner/repo" 문자열을 추출한다.
pub fn get_owner_repo(repo_path: &std::path::Path) -> Option<String> {
    let repo = Repository::discover(repo_path).ok()?;
    extract_owner_repo(&repo)
}

fn extract_owner_repo(repo: &Repository) -> Option<String> {
    let remote = repo.find_remote("origin").ok()?;
    let url = remote.url()?;
    parse_repo_from_url(url)
}

/// GitHub URL에서 "owner/repo" 를 추출한다.
/// - `https://github.com/owner/repo.git`
/// - `git@github.com:owner/repo.git`
pub fn parse_repo_from_url(url: &str) -> Option<String> {
    let path = if let Some(p) = url.strip_prefix("https://github.com/") {
        p
    } else if let Some(p) = url.strip_prefix("git@github.com:") {
        p
    } else {
        return None;
    };
    Some(path.trim_end_matches(".git").to_string())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_since_days() {
        let t = parse_since("7d").unwrap();
        let diff = Utc::now() - t;
        // 허용 오차 1초
        assert!((diff.num_seconds() - 7 * 86400).abs() < 2);
    }

    #[test]
    fn parse_since_days_no_suffix() {
        let t = parse_since("14").unwrap();
        let diff = Utc::now() - t;
        assert!((diff.num_seconds() - 14 * 86400).abs() < 2);
    }

    #[test]
    fn parse_since_date() {
        let t = parse_since("2024-01-15").unwrap();
        assert_eq!(t.format("%Y-%m-%d").to_string(), "2024-01-15");
    }

    #[test]
    fn parse_since_invalid() {
        assert!(parse_since("not-a-date").is_none());
    }

    #[test]
    fn parse_https_url() {
        assert_eq!(
            parse_repo_from_url("https://github.com/grace287/WorkLog-dev.git"),
            Some("grace287/WorkLog-dev".to_string())
        );
    }

    #[test]
    fn parse_ssh_url() {
        assert_eq!(
            parse_repo_from_url("git@github.com:grace287/WorkLog-dev.git"),
            Some("grace287/WorkLog-dev".to_string())
        );
    }

    #[test]
    fn parse_non_github_url() {
        assert!(parse_repo_from_url("https://gitlab.com/foo/bar.git").is_none());
    }
}
