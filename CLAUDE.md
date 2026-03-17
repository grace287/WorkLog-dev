# CLAUDE.md — worklog dev

> 목업 앱 + 설계서에서 추출한 작업 참조 문서. 레포 루트에 위치시켜 바로 사용.

---

## 프로젝트 정의

```
커밋 메시지에 [TASK-ID] 하나만 넣으면
Git 히스토리 전체가 포트폴리오 증거 DB가 된다.

worklog sync → worklog publish
→ https://worklog.dev/p/{slug}  (면접관이 브라우저에서 Verified 커밋 확인)
```

| 레이어 | 스택 | 역할 |
|--------|------|------|
| 🦀 CLI | Rust 2021 · 단일 바이너리 | 로컬 태스크 관리, git2 파싱, GitHub 검증, publish |
| ⚙ 백엔드 | FastAPI + PostgreSQL | 포트폴리오 수신·저장·slug 발행, JWT 인증 |
| 🌐 프론트엔드 | Next.js 14 · App Router | 공개 포트폴리오 뷰어 `/p/{slug}` + 대시보드 |
| 🔧 인프라 | Docker + GitHub Actions | CI/CD, 컨테이너, 배포 |

---


## 시스템 아키텍처


```
┌─────────────────────────────────────────────────────┐
│  🦀 CLI (Rust · 로컬)                                │
│  auth: init / whoami / logout                        │
│  task: add / ls / done / move / link                 │
│  sync ★: git2 revwalk → [TASK-ID] regex → GH verify │
│  portfolio: publish / export / status                │
└───────────────────┬─────────────────────────────────┘
                    │ HTTPS · JWT
                    │ POST /api/v1/portfolios
                    ▼
┌─────────────────────────────────────────────────────┐
│  ⚙ FastAPI (Python · api.worklog.dev)               │
│  /auth/token   — JWT 발급 (PAT 검증)                │
│  /portfolios   — publish 수신, slug 생성, PG 저장   │
│  /portfolios/{slug} — 공개 조회 (FE용)              │
└───────────────────┬─────────────────────────────────┘
                    │ REST · JSON · fetch (SSG/ISR)
                    ▼
┌─────────────────────────────────────────────────────┐
│  🌐 Next.js (web · worklog.dev)                     │
│  /              — 랜딩 (설치 가이드·데모)           │
│  /p/[slug]  ★  — 공개 포트폴리오 뷰어 (SSG)        │
│  /dashboard    — 내 포트폴리오 관리 (v1.0)          │
└─────────────────────────────────────────────────────┘
```

### sync 파이프라인 (CLI 핵심)
```
git2::open() → revwalk (--since) → regex [PROJ-N] 추출
→ GitHub REST API 커밋 검증 → Verified 배지
→ links.json 저장 → FastAPI POST (publish)
```


---

## 프로젝트 구조 (모노레포)


```
worklog-dev/
├── cli/                          # 🦀 Rust 2021
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs               # clap 진입점
│       ├── cli/
│       │   ├── auth.rs           # init · whoami · logout
│       │   ├── task.rs           # add · ls · done · move · link
│       │   ├── sync.rs           # ★ 파이프라인 오케스트레이터
│       │   └── portfolio.rs      # publish · export · status
│       ├── core/
│       │   ├── model.rs          # Task · Commit · TaskCommitLink
│       │   ├── config.rs         # config.toml (toml 0.8)
│       │   ├── http.rs           # ApiClient (reqwest 0.12)
│       │   └── output.rs         # colored 터미널 출력
│       ├── connectors/
│       │   ├── git.rs            # ★ git2 revwalk 파싱
│       │   └── github.rs         # GitHub REST API + 백오프
│       └── tests/
│           └── fixtures/         # tempfile 더미 저장소
│
├── api/                          # ⚙ Python · FastAPI
│   ├── app/
│   │   ├── main.py               # FastAPI 앱
│   │   ├── config.py             # Pydantic Settings
│   │   ├── routers/
│   │   │   ├── auth.py           # JWT 발급
│   │   │   ├── portfolios.py     # ★ publish · 조회
│   │   │   └── tasks.py          # CRUD
│   │   ├── models/               # SQLAlchemy 2.0
│   │   ├── schemas/              # Pydantic v2 DTO
│   │   └── db/
│   │       ├── session.py        # asyncpg 세션
│   │       └── migrations/       # Alembic
│   └── requirements.txt
│
└── web/                          # 🌐 Next.js 14 · App Router
    ├── app/
    │   ├── page.tsx              # 랜딩
    │   ├── p/[slug]/
    │   │   ├── page.tsx          # ★ 공개 뷰어 (SSG/ISR)
    │   │   └── loading.tsx       # Suspense 스켈레톤
    │   ├── not-found.tsx         # 404
    │   ├── dashboard/            # v1.0
    │   └── api/og/[slug]/        # next/og OG 이미지
    ├── components/
    │   ├── PortfolioView.tsx
    │   ├── TaskCard.tsx
    │   ├── CommitTimeline.tsx
    │   └── VerifiedBadge.tsx
    ├── lib/
    │   ├── types.ts              # 공통 타입
    │   ├── schemas.ts            # zod 스키마
    │   └── api.ts                # FastAPI 클라이언트
    ├── tailwind.config.ts
    └── next.config.ts
```


---

## 기술 스택


### 🦀 CLI (Rust 2021)

| 크레이트 | 버전 | 용도 | 우선순위 |
|----------|------|------|----------|
| clap | 4 | 커맨드 파서 (derive) | Must |
| colored | 2 | 터미널 컬러 출력 | Must |
| indicatif | 0.17 | 스피너·프로그레스바 | Must |
| **git2** | 0.18 | libgit2 revwalk 파싱 ★ | Must |
| regex | 1 | [TASK-ID] 추출 | Must |
| once_cell | 1 | 정규식 정적 캐싱 | Must |
| chrono | 0.4 | 타임스탬프·--since | Must |
| tokio | 1 | 비동기 런타임 (full) | Must |
| reqwest | 0.12 | HTTP (rustls-tls) | Must |
| serde + serde_json | 1 | 직렬화 | Must |
| **keyring** | 2 | PAT OS 보안 저장 | Must |
| toml | 0.8 | config.toml | Must |
| anyhow | 1 | .context() 에러 | Must |
| dirs | 5 | OS별 저장 경로 | Must |
| ratatui | 0.26 | TUI 대시보드 | P4 (feature: tui) |
| rusqlite | 0.31 | 오프라인 캐시 | P4 (feature: offline) |
| clap_complete | — | shell 자동완성 | P4 |

### ⚙ 백엔드 (FastAPI · Python 3.12)

| 패키지 | 용도 | 우선순위 |
|--------|------|----------|
| fastapi + uvicorn | ASGI 서버 | Must |
| pydantic v2 | DTO 스키마·Settings | Must |
| sqlalchemy 2.0 | ORM (async) | Must |
| asyncpg | PG 비동기 드라이버 | Must |
| alembic | DB 마이그레이션 | Must |
| python-jose | JWT HS256 | Must |
| python-slugify | URL slug 생성 | Must |
| httpx | 외부 HTTP (OAuth) | Must |
| loguru | 구조화 로깅 | Must |
| python-dotenv / pydantic-settings | 환경변수 | Must |
| pytest + pytest-asyncio | 테스트 | Must |
| passlib + bcrypt | 패스워드 해싱 | P4 |
| aioredis | 캐시·rate limit | P4 |

### 🌐 프론트엔드 (Next.js 14)

| 패키지 | 용도 | 우선순위 |
|--------|------|----------|
| Next.js 14 + TypeScript 5 | App Router · SSG/ISR | Must |
| Tailwind CSS 3 | 유틸리티 CSS | Must |
| zod | API 응답 런타임 검증 | Must |
| shadcn/ui | Badge·Card·Skeleton | Should |
| next/og | 동적 OG 이미지 | Should |
| NextAuth.js v5 | GitHub OAuth (대시보드) | P4 |
| TanStack Query v5 | 클라이언트 캐시 | P4 |
| next-themes | 다크모드 | P4 |

### 🔧 인프라

| 도구 | 용도 |
|------|------|
| Docker + docker-compose | api + postgres + redis |
| GitHub Actions ci.yml | cargo test + pytest + next build 병렬 |
| GitHub Actions release.yml | 태그 → 크로스 컴파일 바이너리 |
| Vercel | Next.js 배포 |
| cargo-tarpaulin | 커버리지 ≥ 70% |
| Playwright | E2E (뷰어 페이지) |


---

## API 명세


### GitHub REST API (CLI → 외부)
`Authorization: Bearer <PAT>` · Base: `https://api.github.com`

```
GET /user
  → { login: string }          # worklog whoami

GET /repos/{owner}/{repo}/commits/{sha}
  → { verified: bool }         # Verified 배지 · Rate: 5000/hr · 429 지수 백오프
```

### FastAPI 백엔드 (CLI/FE → `https://api.worklog.dev`)
`Authorization: Bearer <JWT>`

```
POST /api/v1/auth/token
  Body: { github_pat: string }
  → { access_token: string }   # JWT exp: 24h

POST /api/v1/portfolios          # worklog publish
  Body: {
    project_id: string,
    tasks: Task[],              # 커밋 포함
    visibility: "public" | "unlisted" | "private"
  }
  → { url: "https://worklog.dev/p/{slug}" }

GET /api/v1/portfolios/{slug}    # FE 뷰어 (인증 불필요)
  → Portfolio { tasks[], commits[], verified }

GET /api/v1/portfolios           # 내 목록 (JWT 필요)
  → Portfolio[]

PUT /api/v1/portfolios/{id}      # 수정 (v1.0)
DELETE /api/v1/portfolios/{id}   # 삭제 (v1.0)
GET /api/v1/stats/{slug}         # 조회수 (v1.0)
```


---

## ERD · 데이터 모델


### CLI 로컬 저장소 (`~/.local/share/worklog/`)
```
tasks.json    — [ { id, task_key, title, status, done_at } ]
commits.json  — [ { sha, message, repo, committed_at } ]
links.json    — [ { task_key, sha, verified } ]
config.toml   — { github_login, default_project }
```
> Phase 4에서 SQLite로 마이그레이션 (`feature: offline`)

### PostgreSQL 스키마
```sql
users         (id UUID PK, github_login VARCHAR, github_id BIGINT UNIQUE, created_at TIMESTAMPTZ)
portfolios    (id UUID PK, user_id UUID FK, slug VARCHAR UNIQUE, project_id VARCHAR,
               visibility ENUM, published_at TIMESTAMPTZ)
tasks         (id UUID PK, portfolio_id UUID FK, task_key VARCHAR, title TEXT,
               status ENUM, done_at TIMESTAMPTZ)
task_commits  (id UUID PK, task_id UUID FK, sha VARCHAR(40), message TEXT,
               verified BOOLEAN, committed_at TIMESTAMPTZ)
```


---

## PRD · 요구사항


### 기능 요구사항

| ID | 요구사항 | 우선순위 |
|----|----------|----------|
| FR-AUTH-01~03 | GitHub PAT keychain 저장, whoami, config.toml 생성 | Must |
| FR-TASK-01~03 | task add/ls/done CRUD, PROJ-N ID 자동 부여 | Must |
| FR-SYNC-01 | git2 커밋 파싱 (기본 30일, --since) | Must ★ |
| FR-SYNC-02 | [PROJ-N] regex 추출 → 태스크 자동 연결 | Must ★ |
| FR-SYNC-03 | GitHub REST API 검증 → Verified 배지 | Must ★ |
| FR-PORT-01 | worklog publish → FastAPI POST → URL 반환 | Must |
| FR-PORT-02 | worklog export md/json/pdf 로컬 생성 | Should |
| BE-PORT-01 | POST /portfolios — publish 수신, PG 저장, slug 발행 | Must |
| BE-PORT-02 | GET /portfolios/{slug} — FE 공개 조회 | Must |
| BE-AUTH-01 | POST /auth/token — JWT 발급 | Must |
| BE-AUTH-02 | GitHub OAuth — 웹 대시보드 로그인 | Should P4 |
| FE-VIEW-01 | /p/{slug} — 공개 뷰어 SSG/ISR | Must |
| FE-VIEW-02 | OG 이미지 동적 생성 | Should |
| FE-LAND-01 | 랜딩 페이지 (설치 명령·데모) | Should |
| FE-DASH-01 | 로그인 대시보드 — 내 포트폴리오 관리 | Should P4 |

### 비기능 요구사항

| 구분 | 기준 |
|------|------|
| 성능 | CLI task ls ≤ 200ms · sync 100커밋 ≤ 3초 · FE LCP ≤ 1.5s · API p95 ≤ 200ms |
| 보안 | PAT OS keychain만 저장 · 전구간 TLS · JWT 만료 24h |
| 호환성 | CLI macOS/Linux/Windows · git 2.30+ · 크로스 브라우저 |
| 신뢰성 | CLI 오프라인 CRUD · 커버리지 ≥ 70% · API 가동률 ≥ 99% |


---

## 목업 앱 (`worklog-dev-app.html`)


**화면:** dashboard, kanban, checklist, docs


```
$ worklog status
 ONDO Sprint #3 · 44% · 7일 남음
 Evidence: 23 · Verified: 20
 ⚠ 1 unlinked commit

$ worklog task ls
 ONDO-12  ● doing  Docker 500 에러
 ONDO-13  ● doing  PG pool
 ONDO-14  ✓ done   DB 인덱스 최적화

$ worklog sync
 ✓ 5 commits scanned
 ✓ 4 evidences linked
 ⚠ 1 unlinked: f19a44c

$ worklog publish
 ⠹ Generating...
 ✓ worklog.dev/p/han-gyeoul-ondo 공개됨
```


---

## 마일스톤


| Phase | 버전 | 기간 | 완료 기준 |
|-------|------|------|-----------|
| **P1** CLI Auth + Task Core | v0.1.0 | Week 1–2 | init → whoami → task add/ls/done 전체 플로우 |
| **P2 ★** CLI Sync + FastAPI | v0.2.0 | Week 3–4 | worklog publish → FastAPI → URL 발행 동작 |
| **P3** Next.js 뷰어 오픈 | v0.3.0 | Week 5 | /p/{slug} 면접관 브라우저 접근 가능 |
| **P4** v1.0 완성 | v1.0.0 | Week 6+ | 오프라인·TUI·OAuth·대시보드·CI/CD |

**P2가 핵심 마일스톤** — CLI↔FastAPI 연동 완성 시점이 프로젝트 실증 포인트.


---

## 산출물 체크리스트 (84개)



#### Phase 1 — CLI Auth + Task Core ✅
- [x] `[Infra]` Cargo workspace 구성 (cli/ api/ web/) + Cargo.toml features
- [x] `[CLI]` worklog init — keyring PAT 저장, config.toml 생성
- [x] `[CLI]` worklog whoami — GitHub /user API 호출
- [x] `[CLI]` worklog logout — keyring 삭제
- [x] `[CLI]` task add / ls / done / move / link CRUD
- [x] `[CLI]` core/model.rs — Task, Commit, TaskCommitLink struct
- [x] `[CLI]` core/http.rs — ApiClient (reqwest 0.12 + rustls-tls)
- [x] `[CLI]` core/output.rs — 컨러 출력 + 테이블 (colored 2)
- [x] `[CLI]` core/config.rs — config.toml 읽기/쓰기 (toml 0.8 + serde)
- [x] `[CLI]` dirs 5 — OS별 저장 경로
- [x] `[CLI]` chrono 0.4 — 태스크 타임스탬프, done_at 기록
- [x] `[CLI]` 로컈 JSON 초기화 (tasks.json, commits.json, links.json)
- [x] `[CLI]` once_cell + regex 정적 캐싱
- [x] `[CLI]` anyhow .context() 에러 핸들링
- [x] `[Docs]` README.md 작성

#### Phase 2 — CLI Sync + FastAPI 백엔드 ✅
- [x] `[CLI]` connectors/git.rs — git2 revwalk 컨밋 파싱 엔진
- [x] `[CLI]` [TASK-ID] regex 추출 → TaskCommitLink 자동 연결
- [x] `[CLI]` connectors/github.rs — GitHub REST API 컨밋 검증 → Verified 배지
- [x] `[CLI]` 429 rate limit 지수 백오프 재시도
- [x] `[CLI]` worklog sync — 전체 파이프라인 (--since / --dry-run / --no-verify)
- [x] `[CLI]` worklog push — HEAD 컨밋 즉시 동기화
- [x] `[CLI]` worklog log — 태스크-컨밋 타임라인 출력
- [x] `[CLI]` worklog publish — JWT 획득 후 FastAPI POST
- [x] `[CLI]` indicatif 스피너 + 멀티 프로그레스바 UX
- [ ] `[CLI]` tests/fixtures/ — tempfile 더미 git 저장소 픽스처
- [ ] `[CLI]` sync 통합 테스트 (cargo test, 커버리지 70% 이상)
- [x] `[BE]` FastAPI app 초기 구성 (main.py, router.py)
- [x] `[BE]` Pydantic Settings — 환경변수 클래스 (config.py)
- [x] `[BE]` Pydantic v2 DTO 스키마 (schemas/portfolio.py, user.py, task.py)
- [x] `[BE]` SQLAlchemy 2.0 ORM 모델 (models/)
- [x] `[BE]` asyncpg 비동기 DB 세션 (db/session.py)
- [x] `[BE]` Alembic 마이그레이션 초기 설정 + 첫 revision
- [x] `[BE]` POST /api/v1/auth/token — JWT 발급 (python-jose HS256)
- [x] `[BE]` POST /api/v1/portfolios — publish 수신 + slug 생성
- [x] `[BE]` CORS + 요청 로깅 미들웨어
- [x] `[BE]` loguru 구조화 로깅 설정
- [ ] `[BE]` uvicorn 프로덕션 실행 설정
- [x] `[BE]` .env + python-dotenv 환경변수 관리
- [x] `[BE]` requirements.txt 또는 pyproject.toml + uv
- [x] `[BE]` pytest + pytest-asyncio + httpx TestClient 테스트

#### Phase 3 — Next.js 뷰어 오픈 ✅
- [x] `[FE]` Next.js 14 App Router + TypeScript 5 초기화 (pnpm)
- [x] `[FE]` Tailwind CSS 3 + tailwind.config.ts
- [x] `[FE]` shadcn/ui 설치 (Badge, Card, Separator, Skeleton)
- [x] `[FE]` Geist Font 설정 (layout.tsx)
- [x] `[FE]` lib/types.ts — Portfolio, Task, Commit 공통 타입
- [x] `[FE]` lib/api.ts — FastAPI 클라이언트 (fetch + zod 런타임 파싱)
- [x] `[FE]` lib/schemas.ts — zod 스키마 정의
- [x] `[FE]` app/p/[slug]/page.tsx — 공개 뷰어 (generateStaticParams + ISR revalidate)
- [x] `[FE]` app/p/[slug]/loading.tsx — Suspense 스켈레턴
- [x] `[FE]` app/not-found.tsx — 잘못된 slug 404 처리
- [x] `[FE]` components/PortfolioView.tsx — 메인 레이아웃
- [x] `[FE]` components/TaskCard.tsx — 태스크 카드
- [x] `[FE]` components/CommitTimeline.tsx — 컨밋 타임라인
- [x] `[FE]` components/VerifiedBadge.tsx — GitHub Verified 배지
- [x] `[FE]` app/api/og/[slug]/route.tsx — next/og 동적 OG 이미지
- [x] `[FE]` Next.js Metadata API — generateMetadata per slug
- [x] `[FE]` app/page.tsx — 랜딩 페이지 (설치 명령 + 데모)
- [x] `[FE]` next.config.ts — images.remotePatterns, revalidate
- [ ] `[FE]` Vercel 배포 + 환경변수 NEXT_PUBLIC_API_URL 설정
- [x] `[BE]` GET /api/v1/portfolios/{slug} — 공개 조회 (인증 불필요)
- [x] `[BE]` GET /api/v1/portfolios — 내 포트폴리오 목록 (JWT 필요)
- [x] `[Infra]` Dockerfile (BE) 멀티스테이지 빌드
- [x] `[Infra]` docker-compose.yml (api + postgres + redis)
- [x] `[Infra]` .github/workflows/ci.yml — cargo test + pytest + next build 병렬

#### Phase 4 — v1.0 완성
- [ ] `[CLI]` rusqlite 0.31 오프라인 캐시 — JSON → SQLite (feature: offline)
- [ ] `[CLI]` ratatui 0.26 TUI 대시보드 (feature flag: tui, crossterm 백엔드)
- [ ] `[CLI]` clap_complete — shell 자동완성 (bash / zsh / fish)
- [ ] `[CLI]` cargo install 배포 (crates.io publish)
- [ ] `[CLI]` brew tap 배포 (homebrew-worklog Formula)
- [ ] `[Infra]` .github/workflows/release.yml — 태그 → macOS/Linux/Windows 크로스 컴파일
- [ ] `[BE]` GitHub OAuth 콜백 라우터 (httpx)
- [ ] `[BE]` aioredis — slug 캐시 + rate limit 미들웨어
- [ ] `[BE]` passlib + bcrypt 패스워드 해싱
- [ ] `[BE]` PUT /api/v1/portfolios/{id} — 포트폴리오 수정
- [ ] `[BE]` DELETE /api/v1/portfolios/{id} — 삭제
- [ ] `[BE]` GET /api/v1/stats/{slug} — 조회수 통계
- [ ] `[FE]` NextAuth.js v5 — GitHub OAuth 로그인 세션
- [ ] `[FE]` TanStack Query v5 — 대시보드 클라이언트 캐시
- [ ] `[FE]` app/dashboard/page.tsx — 내 포트폴리오 목록
- [ ] `[FE]` app/dashboard/[id]/edit/page.tsx — 포트폴리오 수정
- [x] `[FE]` 다크모드 (Tailwind dark: + next-themes)
- [ ] `[FE]` 반응형 레이아웃 (모바일 breakpoint)
- [ ] `[Infra]` cargo-tarpaulin 커버리지 리포트 CI 연동
- [ ] `[Infra]` Playwright E2E 테스트 — /p/{slug} 뷰어



---

## 작업 시작 가이드

### 1. 모노레포 초기화
```bash
mkdir worklog-dev && cd worklog-dev
git init

# Rust workspace
cargo new cli
cat > Cargo.toml << 'EOF'
[workspace]
members = ["cli"]
resolver = "2"
EOF

# Python 백엔드
mkdir api && cd api
python -m venv .venv && source .venv/bin/activate
pip install fastapi "uvicorn[standard]" sqlalchemy asyncpg alembic \
    python-jose "passlib[bcrypt]" python-slugify httpx loguru \
    "python-dotenv" "pydantic-settings" pytest pytest-asyncio httpx
cd ..

# Next.js 프론트엔드
pnpm create next-app web --typescript --tailwind --app --no-src-dir --no-import-alias
cd web
pnpm add zod @tanstack/react-query next-auth next-themes
pnpm dlx shadcn-ui@latest init
cd ..
```

### 2. cli/Cargo.toml
```toml
[package]
name = "worklog"
version = "0.1.0"
edition = "2021"

[dependencies]
clap        = { version = "4",    features = ["derive"] }
colored     = "2"
indicatif   = "0.17"
git2        = "0.18"
regex       = "1"
once_cell   = "1"
chrono      = { version = "0.4",  features = ["serde"] }
tokio       = { version = "1",    features = ["full"] }
reqwest     = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
serde       = { version = "1",    features = ["derive"] }
serde_json  = "1"
keyring     = "2"
toml        = "0.8"
dirs        = "5"
anyhow      = "1"

[dev-dependencies]
tempfile = "3"

[features]
default = []
tui     = ["dep:ratatui", "dep:crossterm"]
offline = ["dep:rusqlite"]
```

### 3. docker-compose.yml (개발 환경)
```yaml
services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: worklog
      POSTGRES_USER: worklog
      POSTGRES_PASSWORD: worklog
    ports: ["5432:5432"]
    volumes: ["pgdata:/var/lib/postgresql/data"]

  redis:
    image: redis:7-alpine
    ports: ["6379:6379"]

volumes:
  pgdata:
```

### 4. api/.env
```
DATABASE_URL=postgresql+asyncpg://worklog:worklog@localhost:5432/worklog
REDIS_URL=redis://localhost:6379
JWT_SECRET=change-me-in-production
JWT_EXPIRE_HOURS=24
GITHUB_CLIENT_ID=
GITHUB_CLIENT_SECRET=
```

### 5. Phase 1 구현 순서
```
cli/src/main.rs           ← [1] clap App 진입점, 서브커맨드 등록
cli/src/core/model.rs     ← [2] Task · Commit · Config struct
cli/src/core/config.rs    ← [3] config.toml 읽기/쓰기 + dirs 경로
cli/src/core/output.rs    ← [4] colored 출력 헬퍼
cli/src/core/http.rs      ← [5] ApiClient (reqwest + JWT 헤더)
cli/src/cli/auth.rs       ← [6] worklog init (keyring PAT 저장)
cli/src/cli/task.rs       ← [7] task add / ls / done / move
```

---

## 커밋 컨벤션 (Gitmoji Style)

> **Claude는 커밋 시 반드시 아래 규칙을 따른다. 작업 완료 후 커밋할 때 자동 적용.**

```
<이모지> <type>(<scope>): <한국어 설명> [<TASK-ID>]

✨ feat(cli/sync): git2 revwalk 파서 구현 [WLOG-12]
🐛 fix(cli/github): 429 지수 백오프 재시도 추가 [WLOG-15]
♻️ refactor(api/portfolios): publish 엔드포인트 구조 개선 [API-05]
📝 docs: README.md 설치 가이드 추가
```

### 이모지 → 타입 매핑

| 이모지 | 타입 | 설명 |
|--------|------|------|
| ✨ | feat | 새로운 기능 추가 |
| 🐛 | fix | 버그 수정 |
| 💡 | chore | 자잘한 코드 수정 (주석, 포맷 등) |
| 📝 | docs | 문서 수정 (README 등) |
| 🚚 | build | 빌드 시스템, 패키지 관련 수정 |
| ✅ | test | 테스트 코드 추가/수정 |
| ♻️ | refactor | 코드 리팩터링 (기능 변화 없음) |
| 🚑 | hotfix | 긴급 수정 |
| ⚙️ | ci | CI/CD 관련 변경 |
| 🔧 | config | 설정 파일 수정 |
| 🗑️ | remove | 불필요 파일/코드 삭제 |
| 🔒 | security | 보안 관련 수정 |
| 🚀 | deploy | 배포 관련 커밋 |
| 🧩 | style | 코드 스타일 변경 |
| 🎨 | ui | UI/스타일 관련 변경 |
| 🔄 | sync | 코드/데이터 동기화 |
| 🔥 | clean | 코드/로그 정리 |
| 🧠 | perf | 성능 개선 |

**scope:** `cli/{module}` · `api/{router}` · `web/{component}` · `infra`
**TASK-ID 형식:** `WLOG-N` (P1), `SYNC-N` (P2 sync), `API-N` (P2 BE), `WEB-N` (P3)
**커밋 메시지는 한국어로 작성**
