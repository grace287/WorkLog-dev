from __future__ import annotations

import uuid
from datetime import datetime
from typing import Literal

from pydantic import BaseModel


# ── 커밋 ──────────────────────────────────────────────────────────────────────

class CommitIn(BaseModel):
    sha: str
    message: str
    committed_at: datetime
    verified: bool = False


class CommitOut(BaseModel):
    id: uuid.UUID
    sha: str
    message: str
    committed_at: datetime
    verified: bool

    model_config = {"from_attributes": True}


# ── 태스크 ────────────────────────────────────────────────────────────────────

class TaskIn(BaseModel):
    task_key: str
    title: str
    status: Literal["todo", "doing", "done"] = "todo"
    done_at: datetime | None = None
    commits: list[CommitIn] = []


class TaskOut(BaseModel):
    id: uuid.UUID
    task_key: str
    title: str
    status: str
    done_at: datetime | None
    commits: list[CommitOut] = []

    model_config = {"from_attributes": True}


# ── 포트폴리오 ────────────────────────────────────────────────────────────────

class PortfolioPublishRequest(BaseModel):
    project_id: str
    tasks: list[TaskIn]
    visibility: Literal["public", "unlisted", "private"] = "public"


class PortfolioPublishResponse(BaseModel):
    url: str
    slug: str


class PortfolioOut(BaseModel):
    id: uuid.UUID
    slug: str
    project_id: str
    visibility: str
    published_at: datetime
    tasks: list[TaskOut] = []

    model_config = {"from_attributes": True}
