# worklog

> **커밋 메시지에 `[TASK-ID]` 하나만 넣으면 Git 히스토리 전체가 포트폴리오 증거 DB가 된다.**

```
$ worklog sync
✓ 5 commits scanned
✓ 4 evidences linked  (3 verified)
⚠ 1 unlinked: f19a44c

$ worklog publish
⠹ Generating public portfolio…
✓ worklog.dev/p/grace287-worklog  공개됨
```

[![CI](https://github.com/grace287/WorkLog-dev/actions/workflows/ci.yml/badge.svg)](https://github.com/grace287/WorkLog-dev/actions/workflows/ci.yml)
[![Release](https://github.com/grace287/WorkLog-dev/actions/workflows/release.yml/badge.svg)](https://github.com/grace287/WorkLog-dev/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](#)

---

## 개요

worklog는 개발자가 **Git 커밋을 포트폴리오 증거로 자동 변환**해주는 CLI 도구입니다.

- 커밋 메시지의 `[TASK-ID]` 태그를 파싱해 태스크-커밋 링크를 생성합니다.
- GitHub REST API로 커밋 서명을 검증해 **Verified 배지**를 부여합니다.
- `worklog publish` 한 번으로 공개 포트폴리오 URL을 발급합니다.
- 면접관이 브라우저에서 바로 확인 가능: `worklog.dev/p/{slug}`

---

## 아키텍처

```
┌──────────────────────────────────────────────┐
│  🦀 CLI (Rust · 로컬)                         │
│  auth · task · sync ★ · publish              │
└──────────────┬───────────────────────────────┘
               │ HTTPS · JWT
               ▼
┌──────────────────────────────────────────────┐
│  ⚙ FastAPI  (api.worklog.dev)                │
│  /auth/token  /portfolios  /portfolios/{slug} │
└──────────────┬───────────────────────────────┘
               │ REST · fetch (SSG/ISR)
               ▼
┌──────────────────────────────────────────────┐
│  🌐 Next.js  (worklog.dev)                   │
│  /p/[slug]  /dashboard                       │
└──────────────────────────────────────────────┘
```

---

## 설치

### 바이너리 (권장)

```bash
# macOS / Linux
curl -fsSL https://worklog.dev/install.sh | sh

# Windows (PowerShell)
irm https://worklog.dev/install.ps1 | iex
```

### Cargo

```bash
cargo install worklog
```

### 소스 빌드

```bash
git clone https://github.com/grace287/WorkLog-dev.git
cd WorkLog-dev
cargo build --release -p worklog
# 바이너리: ./target/release/worklog
```

> **요구사항:** Rust 1.75+, git 2.30+

---

## 빠른 시작

```bash
# 1. GitHub PAT 등록 (scopes: read:user, repo)
worklog init

# 2. 태스크 생성
worklog task add "Docker 500 에러 수정" --project ONDO
# → ONDO-1 created

# 3. 커밋할 때 태스크 ID 포함
git commit -m "fix(api): connection pool 설정 수정 [ONDO-1]"

# 4. 커밋 스캔 + GitHub 검증
worklog sync

# 5. 포트폴리오 공개
worklog publish
# → worklog.dev/p/grace287-ondo
```

---

## 명령어 레퍼런스

### 인증

| 명령어 | 설명 |
|--------|------|
| `worklog init` | GitHub PAT를 OS keyring에 저장, config.toml 생성 |
| `worklog whoami` | 현재 인증된 GitHub 로그인 확인 |
| `worklog logout` | OS keyring에서 자격증명 삭제 |

### 태스크 관리

| 명령어 | 설명 |
|--------|------|
| `worklog task add <title> [--project PROJ]` | 새 태스크 추가 (자동 ID 부여) |
| `worklog task ls [--project PROJ]` | 태스크 목록 |
| `worklog task done <TASK-KEY>` | 태스크 완료 처리 |
| `worklog task move <TASK-KEY> <status>` | 상태 변경: `todo` \| `doing` \| `done` |
| `worklog task link <TASK-KEY> <SHA>` | 커밋 수동 연결 |

### 동기화

| 명령어 | 설명 |
|--------|------|
| `worklog sync` | git2 revwalk → TASK-ID 추출 → GitHub 검증 (기본: 30일) |
| `worklog sync --since 7d` | 최근 7일 커밋 스캔 |
| `worklog sync --since 2024-01-01` | 특정 날짜 이후 스캔 |
| `worklog sync --dry-run` | 변경 없이 결과 미리보기 |
| `worklog sync --no-verify` | GitHub 검증 건너뜀 |
| `worklog push` | HEAD 커밋 즉시 동기화 |
| `worklog log` | 태스크-커밋 타임라인 출력 |

### 포트폴리오

| 명령어 | 설명 |
|--------|------|
| `worklog publish [--visibility public\|unlisted\|private]` | FastAPI에 publish → URL 발급 |
| `worklog status` | 프로젝트 진행률 + 증거 현황 요약 |
| `worklog export --format md\|json` | 로컬 파일로 내보내기 |

---

## TASK-ID 규칙

커밋 메시지 어디에나 `[PROJ-N]` 형식으로 포함하면 자동 감지됩니다.

```
feat(api): slug 생성 로직 추가 [WLOG-20]
fix: 429 재시도 버그 수정 [WLOG-15] [SYNC-3]
```

- 대문자 + 숫자 조합: `WLOG-1`, `ONDO-42`, `API-7`
- 하나의 커밋에 여러 TASK-ID 허용

---

## 로컬 데이터 구조

```
~/.local/share/worklog/      (Linux)
~/Library/Application Support/worklog/  (macOS)
%LOCALAPPDATA%\worklog\      (Windows)

├── config.toml   — github_login, default_project, api_url
├── tasks.json    — Task 배열
├── commits.json  — Commit 배열
└── links.json    — TaskCommitLink 배열 (verified 포함)
```

---

## 커밋 컨벤션

이 레포는 [gitmoji](https://gitmoji.dev) 스타일을 사용합니다.

```
✨ feat(cli/sync): git2 revwalk 파서 구현 [WLOG-12]
🐛 fix(cli/github): 429 지수 백오프 수정 [WLOG-15]
📝 docs: README.md 초안 작성
```

---

## 개발 환경

```bash
# Rust 테스트
cargo test --all-features --workspace

# 커버리지 (cargo-tarpaulin 필요)
cargo tarpaulin --all-features --workspace

# Python 백엔드
cd api && pip install -r requirements.txt && pytest

# Next.js 프론트엔드
cd web && pnpm install && pnpm dev
```

---

## 마일스톤

| Phase | 버전 | 내용 |
|-------|------|------|
| **P1** ✅ | v0.1.0 | CLI Auth + Task CRUD |
| **P2** 🔄 | v0.2.0 | Sync 파이프라인 + FastAPI 백엔드 |
| **P3** | v0.3.0 | Next.js 뷰어 `/p/{slug}` |
| **P4** | v1.0.0 | SQLite · TUI · OAuth · CI/CD 완성 |

---

## 라이선스

MIT © 2025 grace287
