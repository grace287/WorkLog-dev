#!/bin/bash

echo "🚀 Next.js 14 프론트엔드 초기 세팅을 시작합니다..."

# 1. Next.js 14 프로젝트 생성 (App Router, Tailwind, TypeScript, no-src-dir 적용)
# 기존에 web 폴더가 있다면 충돌할 수 있으므로, 비어있는 상태에서 실행을 권장합니다.
pnpm create next-app web \
  --typescript \
  --tailwind \
  --eslint \
  --app \
  --no-src-dir \
  --import-alias "@/*" \
  --use-pnpm

cd web || exit

# 2. 필수 라이브러리 추가 설치 (zod, 아이콘 등)
echo "📦 필수 패키지 설치 중..."
pnpm add zod lucide-react date-fns
pnpm add @tanstack/react-query next-auth next-themes

# 3. shadcn/ui 초기화 (-y 플래그로 기본 스타일 New York, Zinc 적용)
echo "🎨 shadcn/ui 초기화..."
pnpm dlx shadcn-ui@latest init -y

# 4. 요구사항에 명시된 기본 UI 컴포넌트 다운로드
echo "🧩 컴포넌트 추가: badge, card, separator, skeleton..."
pnpm dlx shadcn-ui@latest add badge card separator skeleton -y

echo "✅ Phase 3: Next.js + shadcn 세팅이 성공적으로 완료되었습니다!"