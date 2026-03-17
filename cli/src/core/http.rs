use anyhow::{Context, Result, bail};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;

#[allow(dead_code)]
pub struct ApiClient {
    base_url: String,
    jwt_token: Option<String>,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("worklog-cli/0.1.0"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(ApiClient {
            base_url: base_url.into(),
            jwt_token: None,
            client,
        })
    }

    pub fn with_token(mut self, jwt: impl Into<String>) -> Self {
        self.jwt_token = Some(jwt.into());
        self
    }

    fn auth_header(&self) -> Option<HeaderValue> {
        self.jwt_token.as_ref().and_then(|t| {
            HeaderValue::from_str(&format!("Bearer {}", t)).ok()
        })
    }

    pub async fn get(&self, path: &str) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            req = req.header(AUTHORIZATION, auth);
        }
        let resp = req.send().await.with_context(|| format!("GET {} failed", url))?;
        Ok(resp)
    }

    pub async fn post<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.post(&url).json(body);
        if let Some(auth) = self.auth_header() {
            req = req.header(AUTHORIZATION, auth);
        }
        let resp = req.send().await.with_context(|| format!("POST {} failed", url))?;
        Ok(resp)
    }
}

// ── GitHub helpers ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GithubUser {
    login: String,
}

pub async fn get_github_user(pat: &str) -> Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("worklog-cli/0.1.0"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", pat))
            .context("Invalid PAT characters")?,
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .context("Failed to build HTTP client")?;

    let resp = client
        .get("https://api.github.com/user")
        .send()
        .await
        .context("Failed to reach GitHub API")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("GitHub API returned {}: {}", status, body);
    }

    let user: GithubUser = resp.json().await.context("Failed to parse GitHub user response")?;
    Ok(user.login)
}

// ── Backend JWT helper ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[allow(dead_code)]
pub async fn get_jwt_token(api_url: &str, pat: &str) -> Result<String> {
    let client = ApiClient::new(api_url)?;

    let body = serde_json::json!({ "github_pat": pat });
    let resp = client
        .post("/api/v1/auth/token", &body)
        .await
        .context("Failed to request JWT token")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        bail!("Auth token request failed {}: {}", status, body);
    }

    let token_resp: TokenResponse = resp.json().await.context("Failed to parse token response")?;
    Ok(token_resp.access_token)
}
