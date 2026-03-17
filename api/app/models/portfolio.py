import uuid
from datetime import datetime, timezone

from sqlalchemy import Boolean, Enum, ForeignKey, String, Text, func
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.session import Base


class Portfolio(Base):
    __tablename__ = "portfolios"

    id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
    user_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("users.id", ondelete="CASCADE"))
    slug: Mapped[str] = mapped_column(String(255), unique=True, nullable=False)
    project_id: Mapped[str] = mapped_column(String(100), nullable=False)
    visibility: Mapped[str] = mapped_column(
        Enum("public", "unlisted", "private", name="visibility_enum"),
        default="public",
    )
    published_at: Mapped[datetime] = mapped_column(
        default=lambda: datetime.now(timezone.utc),
        server_default=func.now(),
    )

    user: Mapped["User"] = relationship(back_populates="portfolios")  # noqa: F821
    tasks: Mapped[list["Task"]] = relationship(
        back_populates="portfolio", cascade="all, delete-orphan"
    )


class Task(Base):
    __tablename__ = "tasks"

    id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
    portfolio_id: Mapped[uuid.UUID] = mapped_column(
        ForeignKey("portfolios.id", ondelete="CASCADE")
    )
    task_key: Mapped[str] = mapped_column(String(50), nullable=False)
    title: Mapped[str] = mapped_column(Text, nullable=False)
    status: Mapped[str] = mapped_column(
        Enum("todo", "doing", "done", name="task_status_enum"), default="todo"
    )
    done_at: Mapped[datetime | None] = mapped_column(nullable=True)

    portfolio: Mapped["Portfolio"] = relationship(back_populates="tasks")
    commits: Mapped[list["TaskCommit"]] = relationship(
        back_populates="task", cascade="all, delete-orphan"
    )


class TaskCommit(Base):
    __tablename__ = "task_commits"

    id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
    task_id: Mapped[uuid.UUID] = mapped_column(ForeignKey("tasks.id", ondelete="CASCADE"))
    sha: Mapped[str] = mapped_column(String(40), nullable=False)
    message: Mapped[str] = mapped_column(Text, nullable=False)
    verified: Mapped[bool] = mapped_column(Boolean, default=False)
    committed_at: Mapped[datetime] = mapped_column(nullable=False)

    task: Mapped["Task"] = relationship(back_populates="commits")
