import { formatDistanceToNow } from "date-fns";
import { ko } from "date-fns/locale";
import { VerifiedBadge } from "./VerifiedBadge";
import type { Commit } from "@/lib/types";

export function CommitTimeline({ commits }: { commits: Commit[] }) {
  if (commits.length === 0) return null;

  return (
    <ul className="mt-3 space-y-2 border-l border-zinc-200 pl-4">
      {commits.map((c) => (
        <li key={c.id} className="relative">
          {/* 타임라인 점 */}
          <span className="absolute -left-[1.35rem] top-1.5 h-2 w-2 rounded-full border-2 border-white bg-zinc-400" />
          <div className="flex flex-wrap items-start gap-2">
            <code className="shrink-0 rounded bg-zinc-100 px-1.5 py-0.5 font-mono text-xs text-zinc-600">
              {c.sha.slice(0, 7)}
            </code>
            <VerifiedBadge verified={c.verified} />
            <span className="text-sm text-zinc-700 leading-snug flex-1 min-w-0">
              {c.message.split("\n")[0]}
            </span>
            <time className="shrink-0 text-xs text-zinc-400">
              {formatDistanceToNow(new Date(c.committed_at), {
                addSuffix: true,
                locale: ko,
              })}
            </time>
          </div>
        </li>
      ))}
    </ul>
  );
}
