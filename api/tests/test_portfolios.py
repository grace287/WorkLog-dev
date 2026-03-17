from datetime import datetime, timezone
from unittest.mock import AsyncMock, patch

import pytest
from httpx import AsyncClient
from jose import jwt

from app.config import settings


def _make_token(login: str = "grace287", github_id: int = 12345678) -> str:
    from datetime import timedelta
    expire = datetime.now(timezone.utc) + timedelta(hours=1)
    return jwt.encode(
        {"sub": login, "github_id": github_id, "exp": expire},
        settings.jwt_secret,
        algorithm="HS256",
    )


PUBLISH_PAYLOAD = {
    "project_id": "ONDO",
    "visibility": "public",
    "tasks": [
        {
            "task_key": "ONDO-1",
            "title": "Docker 500 에러 수정",
            "status": "done",
            "done_at": "2025-03-01T12:00:00Z",
            "commits": [
                {
                    "sha": "abc123def456abc123def456abc123def456abc1",
                    "message": "fix(api): connection pool 설정 수정 [ONDO-1]",
                    "committed_at": "2025-03-01T10:00:00Z",
                    "verified": True,
                }
            ],
        }
    ],
}


@pytest.mark.anyio
async def test_publish_portfolio(client: AsyncClient):
    token = _make_token()
    with patch("app.routers.portfolios._get_or_create_user", new_callable=AsyncMock) as mock_user:
        from app.models.user import User
        import uuid
        fake_user = User(github_login="grace287", github_id=12345678)
        fake_user.id = uuid.uuid4()
        mock_user.return_value = fake_user

        resp = await client.post(
            "/api/v1/portfolios",
            json=PUBLISH_PAYLOAD,
            headers={"Authorization": f"Bearer {token}"},
        )

    assert resp.status_code == 201
    data = resp.json()
    assert "url" in data
    assert "grace287" in data["url"]
    assert "ondo" in data["slug"]


@pytest.mark.anyio
async def test_publish_requires_auth(client: AsyncClient):
    resp = await client.post("/api/v1/portfolios", json=PUBLISH_PAYLOAD)
    assert resp.status_code == 403


@pytest.mark.anyio
async def test_get_portfolio_not_found(client: AsyncClient):
    resp = await client.get("/api/v1/portfolios/nonexistent-slug-xyz")
    assert resp.status_code == 404
