// Phase 2 — GitHub REST API commit verification + exponential backoff
// Full implementation in Phase 2 (SYNC-* tasks).

use anyhow::Result;

/// Verify a single commit SHA against the GitHub REST API.
///
/// Phase 2 implementation will:
///   GET /repos/{owner}/{repo}/commits/{sha}
///   Authorization: Bearer <PAT>
///   → parse `.commit.verification.verified`
///   Handle HTTP 429 with exponential backoff (up to 5 retries).
///
/// Returns `false` (unverified) until Phase 2 wires in the real logic.
#[allow(dead_code)]
pub async fn verify_commit(
    _pat: &str,
    _owner: &str,
    _repo: &str,
    _sha: &str,
) -> Result<bool> {
    Ok(false)
}

/// Verify a batch of SHAs, respecting the 5000/hr GitHub rate limit.
///
/// Phase 2 will fan these out with a semaphore-limited async loop and
/// apply jittered exponential backoff on 429 responses.
#[allow(dead_code)]
pub async fn verify_commits_batch(
    _pat: &str,
    _owner: &str,
    _repo: &str,
    _shas: &[String],
) -> Result<Vec<bool>> {
    Ok(Vec::new())
}
