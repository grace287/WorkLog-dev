import uuid
from datetime import datetime, timezone

from sqlalchemy import BigInteger, String, func
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db.session import Base


class User(Base):
    __tablename__ = "users"

    id: Mapped[uuid.UUID] = mapped_column(primary_key=True, default=uuid.uuid4)
    github_login: Mapped[str] = mapped_column(String(255), nullable=False)
    github_id: Mapped[int] = mapped_column(BigInteger, unique=True, nullable=False)
    created_at: Mapped[datetime] = mapped_column(
        default=lambda: datetime.now(timezone.utc),
        server_default=func.now(),
    )

    portfolios: Mapped[list["Portfolio"]] = relationship(  # noqa: F821
        back_populates="user", cascade="all, delete-orphan"
    )
