from datetime import datetime, timedelta, timezone

import httpx
from fastapi import APIRouter, HTTPException, status
from jose import jwt

from app.config import settings
from app.schemas.auth import TokenRequest, TokenResponse

router = APIRouter(prefix="/api/v1/auth", tags=["auth"])

GITHUB_USER_URL = "https://api.github.com/user"


async def _get_github_user(pat: str) -> dict:
    headers = {
        "Authorization": f"Bearer {pat}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
        "User-Agent": "worklog-api/0.2.0",
    }
    async with httpx.AsyncClient(timeout=10) as client:
        resp = await client.get(GITHUB_USER_URL, headers=headers)

    if resp.status_code != 200:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"GitHub PAT verification failed: {resp.status_code}",
        )
    return resp.json()


def _create_jwt(github_login: str, github_id: int) -> str:
    expire = datetime.now(timezone.utc) + timedelta(hours=settings.jwt_expire_hours)
    payload = {
        "sub": github_login,
        "github_id": github_id,
        "exp": expire,
    }
    return jwt.encode(payload, settings.jwt_secret, algorithm="HS256")


@router.post("/token", response_model=TokenResponse)
async def issue_token(body: TokenRequest) -> TokenResponse:
    """GitHub PAT를 검증하고 JWT를 발급한다."""
    try:
        user = await _get_github_user(body.github_pat)
    except HTTPException:
        raise
    except Exception as e:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail=f"GitHub PAT verification failed: {e}",
        )
    token = _create_jwt(github_login=user["login"], github_id=user["id"])
    return TokenResponse(access_token=token)
