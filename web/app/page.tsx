import Link from "next/link";

export default function Home() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center px-4 bg-zinc-950 text-white">
      {/* 히어로 */}
      <section className="text-center max-w-2xl">
        <div className="mb-4 inline-flex items-center gap-2 rounded-full bg-zinc-800 px-3 py-1 text-xs text-zinc-400">
          <span className="h-1.5 w-1.5 rounded-full bg-green-400" />
          Open Beta
        </div>
        <h1 className="text-5xl font-bold tracking-tight mb-4">
          Git commits → Portfolio
        </h1>
        <p className="text-zinc-400 text-lg mb-8">
          커밋 메시지에{" "}
          <code className="rounded bg-zinc-800 px-1.5 py-0.5 text-zinc-200 font-mono text-sm">
            [TASK-ID]
          </code>{" "}
          하나만 넣으면 Git 히스토리 전체가 <br />
          검증된 포트폴리오 증거 DB가 됩니다.
        </p>

        {/* 설치 명령 */}
        <div className="mx-auto mb-8 max-w-md rounded-xl bg-zinc-900 border border-zinc-800 p-4 text-left font-mono text-sm">
          <p className="text-zinc-500 mb-1"># macOS / Linux</p>
          <p className="text-green-400">curl -fsSL https://worklog.dev/install.sh | sh</p>
        </div>

        {/* 데모 터미널 */}
        <div className="mx-auto mb-10 max-w-lg rounded-xl bg-zinc-900 border border-zinc-800 p-5 text-left font-mono text-sm space-y-1">
          <p><span className="text-zinc-500">$</span> <span className="text-white">worklog sync</span></p>
          <p className="text-green-400">✓ 5 commits scanned</p>
          <p className="text-green-400">✓ 4 evidences linked  (3 verified)</p>
          <p className="text-yellow-400">⚠ 1 unlinked: f19a44c</p>
          <p className="mt-2"><span className="text-zinc-500">$</span> <span className="text-white">worklog publish</span></p>
          <p className="text-green-400">✓ worklog.dev/p/grace287-worklog  공개됨</p>
        </div>

        {/* CTA */}
        <div className="flex justify-center gap-3">
          <Link
            href="https://github.com/grace287/WorkLog-dev"
            className="rounded-lg bg-white px-5 py-2.5 text-sm font-semibold text-zinc-900 hover:bg-zinc-100 transition-colors"
          >
            GitHub
          </Link>
          <Link
            href="/p/demo"
            className="rounded-lg border border-zinc-700 px-5 py-2.5 text-sm font-semibold text-zinc-300 hover:border-zinc-500 transition-colors"
          >
            Demo 포트폴리오 →
          </Link>
        </div>
      </section>
    </main>
  );
}
