# ── 빌드 스테이지 ──────────────────────────────────────────────────────────────
FROM python:3.12-slim AS builder

WORKDIR /build
COPY requirements.txt .
RUN pip install --no-cache-dir --prefix=/install -r requirements.txt

# ── 런타임 스테이지 ─────────────────────────────────────────────────────────────
FROM python:3.12-slim

WORKDIR /app

# 빌드 스테이지에서 설치된 패키지 복사
COPY --from=builder /install /usr/local

# 소스 복사
COPY app/ ./app/

EXPOSE 8000

CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8000"]
