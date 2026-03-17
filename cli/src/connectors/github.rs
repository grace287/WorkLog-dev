use anyhow::{Context, Result, bail};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::time::Duration;

// ── GitHub API 응답 타입 ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CommitResponse {
    commit: CommitDetail,
}

#[derive(Deserialize)]
struct CommitDetail {
    verification: Verification,
}

#[derive(Deserialize)]
struct Verification {
    verified: bool,
}

// ── HTTP 클라이언트 ─────────────────────────────────────────────────────────

fn build_client(pat: &str) -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("worklog-cli/0.1.0"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", pat)).context("Invalid PAT characters")?,
    );
    // GitHub API v3 권장 Accept 헤더
    headers.insert(
        reqwest::header::ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        reqwest::header::HeaderName::from_static("x-github-api-version"),
        HeaderValue::from_static("2022-11-28"),
    );

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .build()
        .context("Failed to build HTTP client")
}

// ── 커밋 검증 ───────────────────────────────────────────────────────────────

/// GitHub REST API로 단일 커밋 서명을 검증한다.
/// 429 / 403 응답 시 지수 백오프로 최대 5회 재시도.
///
/// `owner_repo` 형식: "grace287/WorkLog-dev"
pub async fn verify_commit(pat: &str, owner_repo: &str, sha: &str) -> Result<bool> {
    let client = build_client(pat)?;
    let url = format!(
        "https://api.github.com/repos/{}/commits/{}",
        owner_repo, sha
    );

    const MAX_RETRIES: u32 = 5;
    let mut delay_secs = 1u64;

    for attempt in 0..=MAX_RETRIES {
        let resp = client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {} failed", url))?;

        match resp.status().as_u16() {
            200 => {
                let body: CommitResponse = resp
                    .json()
                    .await
                    .context("Failed to parse GitHub commit response")?;
                return Ok(body.commit.verification.verified);
            }
            404 => {
                // 로컬 전용 커밋이거나 비공개 repo에 접근 권한 없음
                return Ok(false);
            }
            429 | 403 => {
                if attempt == MAX_RETRIES {
                    bail!(
                        "GitHub rate limit: exceeded {} retries for {}",
                        MAX_RETRIES,
                        &sha[..sha.len().min(12)]
                    );
                }
                // Retry-After 헤더 우선, 없으면 지수 백오프
                let wait = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(delay_secs);

                tokio::time::sleep(Duration::from_secs(wait)).await;
                delay_secs = (delay_secs * 2).min(60);
            }
            status => {
                let body = resp.text().await.unwrap_or_default();
                bail!("GitHub API error {}: {}", status, body);
            }
        }
    }

    unreachable!()
}

/// SHA 배열을 순차적으로 검증해 결과를 반환한다.
/// 개별 오류는 false로 처리하고 계속 진행한다.
pub async fn verify_commits_batch(
    pat: &str,
    owner_repo: &str,
    shas: &[String],
) -> Vec<(String, bool)> {
    let mut results = Vec::with_capacity(shas.len());
    for sha in shas {
        let verified = verify_commit(pat, owner_repo, sha)
            .await
            .unwrap_or(false);
        results.push((sha.clone(), verified));
    }
    results
}
