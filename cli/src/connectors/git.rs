// Phase 2 — git2 revwalk commit parsing engine
// Full implementation in Phase 2 (SYNC-* tasks).

use anyhow::Result;

use crate::core::model::Commit;

/// Parse commits from the repository at `repo_path` going back `since_days` days.
///
/// Phase 2 implementation will:
///   1. `git2::Repository::open(repo_path)`
///   2. Build a `Revwalk` and push HEAD
///   3. Filter by `committed_at >= Utc::now() - Duration::days(since_days)`
///   4. Extract sha, message, author timestamp → `Commit`
///
/// Returns an empty Vec until Phase 2 wires in the real logic.
#[allow(dead_code)]
pub fn parse_commits(_repo_path: &std::path::Path, _since_days: u32) -> Result<Vec<Commit>> {
    Ok(Vec::new())
}

/// Parse commits from `repo_path` since an explicit ISO-8601 date string
/// (e.g. "2024-01-01" or "30d").
///
/// Phase 2 will parse the `since` string via `chrono` and drive the revwalk.
#[allow(dead_code)]
pub fn parse_commits_since(_repo_path: &std::path::Path, _since: &str) -> Result<Vec<Commit>> {
    Ok(Vec::new())
}
