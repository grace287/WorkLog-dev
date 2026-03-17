from unittest.mock import AsyncMock, patch

import pytest
from httpx import AsyncClient

from app.config import settings
from jose import jwt


@pytest.mark.anyio
async def test_issue_token_success(client: AsyncClient):
    mock_github_user = {"login": "grace287", "id": 12345678}

    with patch("app.routers.auth._get_github_user", new_callable=AsyncMock) as mock_gh:
        mock_gh.return_value = mock_github_user

        resp = await client.post(
            "/api/v1/auth/token",
            json={"github_pat": "ghp_test_token"},
        )

    assert resp.status_code == 200
    data = resp.json()
    assert "access_token" in data

    payload = jwt.decode(data["access_token"], settings.jwt_secret, algorithms=["HS256"])
    assert payload["sub"] == "grace287"
    assert payload["github_id"] == 12345678


@pytest.mark.anyio
async def test_issue_token_invalid_pat(client: AsyncClient):
    with patch("app.routers.auth._get_github_user", new_callable=AsyncMock) as mock_gh:
        mock_gh.side_effect = Exception("401")

        resp = await client.post(
            "/api/v1/auth/token",
            json={"github_pat": "invalid"},
        )

    assert resp.status_code in (401, 500)


@pytest.mark.anyio
async def test_health(client: AsyncClient):
    resp = await client.get("/health")
    assert resp.status_code == 200
    assert resp.json()["status"] == "ok"
