import uuid
from typing import Annotated

from fastapi import APIRouter, Depends, HTTPException, status
from jose import JWTError, jwt
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from slugify import slugify
from sqlalchemy import select
from sqlalchemy.ext.asyncio import AsyncSession
from sqlalchemy.orm import selectinload

from app.config import settings
from app.db.session import get_db
from app.models.portfolio import Portfolio, Task, TaskCommit
from app.models.user import User
from app.schemas.portfolio import (
    PortfolioOut,
    PortfolioPublishRequest,
    PortfolioPublishResponse,
)

router = APIRouter(prefix="/api/v1/portfolios", tags=["portfolios"])
bearer = HTTPBearer()

PUBLIC_BASE_URL = "https://worklog.dev/p"


# ── JWT 검증 ─────────────────────────────────────────────────────────────────

def _decode_token(token: str) -> dict:
    try:
        return jwt.decode(token, settings.jwt_secret, algorithms=["HS256"])
    except JWTError as e:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED, detail=f"Invalid token: {e}"
        )


async def _get_or_create_user(
    payload: dict, db: AsyncSession
) -> User:
    github_id: int = payload["github_id"]
    github_login: str = payload["sub"]

    result = await db.execute(select(User).where(User.github_id == github_id))
    user = result.scalar_one_or_none()

    if user is None:
        user = User(github_login=github_login, github_id=github_id)
        db.add(user)
        await db.flush()

    return user


def _make_slug(github_login: str, project_id: str, attempt: int = 0) -> str:
    base = slugify(f"{github_login}-{project_id}", separator="-")
    return base if attempt == 0 else f"{base}-{attempt}"


# ── POST /api/v1/portfolios ───────────────────────────────────────────────────

@router.post("", response_model=PortfolioPublishResponse, status_code=status.HTTP_201_CREATED)
async def publish_portfolio(
    body: PortfolioPublishRequest,
    creds: Annotated[HTTPAuthorizationCredentials, Depends(bearer)],
    db: Annotated[AsyncSession, Depends(get_db)],
) -> PortfolioPublishResponse:
    """CLI의 `worklog publish` 요청을 수신해 슬러그를 발급한다."""
    payload = _decode_token(creds.credentials)
    user = await _get_or_create_user(payload, db)

    # 중복 없는 slug 생성
    slug = None
    for attempt in range(10):
        candidate = _make_slug(user.github_login, body.project_id, attempt)
        existing = await db.execute(
            select(Portfolio).where(Portfolio.slug == candidate)
        )
        if existing.scalar_one_or_none() is None:
            slug = candidate
            break

    if slug is None:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Could not generate a unique slug — try a different project_id",
        )

    # Portfolio 저장
    portfolio = Portfolio(
        user_id=user.id,
        slug=slug,
        project_id=body.project_id,
        visibility=body.visibility,
    )
    db.add(portfolio)
    await db.flush()

    # Task + TaskCommit 저장
    for task_in in body.tasks:
        task = Task(
            id=uuid.uuid4(),
            portfolio_id=portfolio.id,
            task_key=task_in.task_key,
            title=task_in.title,
            status=task_in.status,
            done_at=task_in.done_at,
        )
        db.add(task)
        await db.flush()

        for commit_in in task_in.commits:
            db.add(
                TaskCommit(
                    task_id=task.id,
                    sha=commit_in.sha,
                    message=commit_in.message,
                    verified=commit_in.verified,
                    committed_at=commit_in.committed_at,
                )
            )

    return PortfolioPublishResponse(
        url=f"{PUBLIC_BASE_URL}/{slug}",
        slug=slug,
    )


# ── GET /api/v1/portfolios/{slug} (공개, 인증 불필요) ─────────────────────────

@router.get("/{slug}", response_model=PortfolioOut)
async def get_portfolio(
    slug: str,
    db: Annotated[AsyncSession, Depends(get_db)],
) -> PortfolioOut:
    result = await db.execute(
        select(Portfolio)
        .where(Portfolio.slug == slug, Portfolio.visibility != "private")
        .options(selectinload(Portfolio.tasks).selectinload(Task.commits))
    )
    portfolio = result.scalar_one_or_none()
    if portfolio is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Portfolio not found")
    return PortfolioOut.model_validate(portfolio)


# ── GET /api/v1/portfolios (내 목록, JWT 필요) ────────────────────────────────

@router.get("", response_model=list[PortfolioOut])
async def list_portfolios(
    creds: Annotated[HTTPAuthorizationCredentials, Depends(bearer)],
    db: Annotated[AsyncSession, Depends(get_db)],
) -> list[PortfolioOut]:
    payload = _decode_token(creds.credentials)
    github_id: int = payload["github_id"]

    result = await db.execute(
        select(Portfolio)
        .join(User)
        .where(User.github_id == github_id)
        .options(selectinload(Portfolio.tasks).selectinload(Task.commits))
        .order_by(Portfolio.published_at.desc())
    )
    portfolios = result.scalars().all()
    return [PortfolioOut.model_validate(p) for p in portfolios]
