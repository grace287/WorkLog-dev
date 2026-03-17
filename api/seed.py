"""
demo 포트폴리오 시드 데이터 생성 스크립트
실행: python seed.py  (api/ 디렉터리에서)
"""
import asyncio
import uuid
from datetime import datetime, timedelta

from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from dotenv import load_dotenv
import os

load_dotenv()
DATABASE_URL = os.environ.get(
    "DATABASE_URL",
    "postgresql+asyncpg://worklog:worklog@localhost:5433/worklog",
)

from app.models.user import User
from app.models.portfolio import Portfolio, Task, TaskCommit

engine = create_async_engine(DATABASE_URL, echo=False)
SessionLocal = async_sessionmaker(engine, expire_on_commit=False)


def dt(days_ago: int, hour: int = 10) -> datetime:
    """naive UTC datetime (asyncpg TIMESTAMP WITHOUT TIME ZONE 호환)"""
    return datetime.utcnow() - timedelta(days=days_ago, hours=-hour)


async def seed():
    async with SessionLocal() as session:
        from sqlalchemy import select, delete
        # 기존 demo user 삭제 (cascade -> portfolios/tasks/commits 전부 삭제)
        existing_user = await session.execute(
            select(User).where(User.github_id == 12345678)
        )
        if existing_user.scalar_one_or_none():
            await session.execute(delete(User).where(User.github_id == 12345678))
            await session.commit()
            print("existing demo data cleared")

        # User
        user = User(
            id=uuid.uuid4(),
            github_login="grace287",
            github_id=12345678,
            created_at=datetime.utcnow(),
        )
        session.add(user)
        await session.flush()

        # Portfolio
        portfolio = Portfolio(
            id=uuid.uuid4(),
            user_id=user.id,
            slug="demo",
            project_id="WORKLOG",
            visibility="public",
            published_at=dt(0),
        )
        session.add(portfolio)
        await session.flush()

        # Task 1 — done
        t1 = Task(
            id=uuid.uuid4(),
            portfolio_id=portfolio.id,
            task_key="WLOG-1",
            title="CLI Auth 구현 (worklog init / whoami)",
            status="done",
            done_at=dt(12),
        )
        session.add(t1)
        await session.flush()

        for sha, msg, verified, days in [
            ("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2", "✨ feat(cli/auth): worklog init keyring PAT 저장 구현 [WLOG-1]", True, 14),
            ("b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3", "🐛 fix(cli/auth): PAT 검증 실패 시 오류 메시지 개선 [WLOG-1]", True, 13),
            ("c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4", "✅ test(cli/auth): whoami 통합 테스트 추가 [WLOG-1]", False, 12),
        ]:
            session.add(TaskCommit(
                id=uuid.uuid4(), task_id=t1.id,
                sha=sha, message=msg, verified=verified,
                committed_at=dt(days),
            ))

        # Task 2 — done
        t2 = Task(
            id=uuid.uuid4(),
            portfolio_id=portfolio.id,
            task_key="WLOG-2",
            title="git2 revwalk 커밋 파싱 엔진",
            status="done",
            done_at=dt(8),
        )
        session.add(t2)
        await session.flush()

        for sha, msg, verified, days in [
            ("d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5", "✨ feat(cli/git): git2 revwalk 파서 초기 구현 [WLOG-2]", True, 11),
            ("e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6", "♻️ refactor(cli/git): --since 플래그 날짜 파싱 개선 [WLOG-2]", True, 10),
            ("f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1", "✨ feat(cli/git): [TASK-ID] regex 자동 추출 연결 [WLOG-2]", True, 9),
            ("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b3", "🐛 fix(cli/github): 429 지수 백오프 재시도 추가 [WLOG-2]", True, 8),
        ]:
            session.add(TaskCommit(
                id=uuid.uuid4(), task_id=t2.id,
                sha=sha, message=msg, verified=verified,
                committed_at=dt(days),
            ))

        # Task 3 — done
        t3 = Task(
            id=uuid.uuid4(),
            portfolio_id=portfolio.id,
            task_key="WLOG-3",
            title="FastAPI 백엔드 — portfolio publish API",
            status="done",
            done_at=dt(4),
        )
        session.add(t3)
        await session.flush()

        for sha, msg, verified, days in [
            ("b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c4", "✨ feat(api/portfolios): POST /portfolios publish 엔드포인트 구현 [WLOG-3]", True, 7),
            ("c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d5", "🔧 config(api): SQLAlchemy 2.0 asyncpg 세션 설정 [WLOG-3]", True, 6),
            ("d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e6", "🔒 security(api/auth): JWT HS256 발급 라우터 구현 [WLOG-3]", True, 5),
            ("e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f7", "✅ test(api): pytest 통합 테스트 작성 [WLOG-3]", False, 4),
        ]:
            session.add(TaskCommit(
                id=uuid.uuid4(), task_id=t3.id,
                sha=sha, message=msg, verified=verified,
                committed_at=dt(days),
            ))

        # Task 4 — doing
        t4 = Task(
            id=uuid.uuid4(),
            portfolio_id=portfolio.id,
            task_key="WLOG-4",
            title="Next.js 포트폴리오 뷰어 + 다크모드",
            status="doing",
        )
        session.add(t4)
        await session.flush()

        for sha, msg, verified, days in [
            ("f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a2", "✨ feat(web): /p/[slug] SSG 뷰어 구현 [WLOG-4]", True, 3),
            ("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b4", "🎨 ui(web): 다크/라이트 모드 토글 추가 [WLOG-4]", True, 1),
        ]:
            session.add(TaskCommit(
                id=uuid.uuid4(), task_id=t4.id,
                sha=sha, message=msg, verified=verified,
                committed_at=dt(days),
            ))

        await session.commit()
        print("[OK] demo portfolio seeded: http://localhost:3000/p/demo")


if __name__ == "__main__":
    asyncio.run(seed())
