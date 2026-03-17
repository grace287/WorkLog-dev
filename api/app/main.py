from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from loguru import logger

from app.db.session import engine
from app.models import Portfolio, Task, TaskCommit, User  # noqa: F401 — register models
from app.db.session import Base
from app.routers import auth, portfolios


@asynccontextmanager
async def lifespan(app: FastAPI):
    logger.info("Starting worklog API…")
    # 개발 환경: 테이블 자동 생성 (프로덕션은 Alembic 마이그레이션 사용)
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
    logger.info("Database tables ready")
    yield
    logger.info("Shutting down worklog API…")
    await engine.dispose()


app = FastAPI(
    title="worklog API",
    version="0.2.0",
    description="Git commit → portfolio evidence API",
    lifespan=lifespan,
)

# ── CORS ─────────────────────────────────────────────────────────────────────

app.add_middleware(
    CORSMiddleware,
    allow_origins=["https://worklog.dev", "http://localhost:3000"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# ── 요청 로깅 미들웨어 ─────────────────────────────────────────────────────────

from fastapi import Request  # noqa: E402
import time  # noqa: E402


@app.middleware("http")
async def log_requests(request: Request, call_next):
    start = time.perf_counter()
    response = await call_next(request)
    elapsed = (time.perf_counter() - start) * 1000
    logger.info(f"{request.method} {request.url.path} → {response.status_code} ({elapsed:.1f}ms)")
    return response


# ── 라우터 등록 ───────────────────────────────────────────────────────────────

app.include_router(auth.router)
app.include_router(portfolios.router)


@app.get("/health")
async def health() -> dict:
    return {"status": "ok", "version": "0.2.0"}
