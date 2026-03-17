import { formatDistanceToNow } from "date-fns";
import { ko } from "date-fns/locale";
import { TaskCard } from "./TaskCard";
import type { Portfolio } from "@/lib/types";

export function PortfolioView({ portfolio }: { portfolio: Portfolio }) {
  const totalCommits = portfolio.tasks.reduce((s, t) => s + t.commits.length, 0);
  const verifiedCommits = portfolio.tasks.reduce(
    (s, t) => s + t.commits.filter((c) => c.verified).length,
    0
  );
  const doneTasks = portfolio.tasks.filter((t) => t.status === "done").length;

  return (
    <main className="mx-auto max-w-3xl px-4 py-12">
      {/* 헤더 */}
      <header className="mb-8">
        <div className="flex items-baseline gap-3">
          <h1 className="text-3xl font-bold tracking-tight text-zinc-900">
            {portfolio.project_id}
          </h1>
          <span className="text-sm text-zinc-400">
            published{" "}
            {formatDistanceToNow(new Date(portfolio.published_at), {
              addSuffix: true,
              locale: ko,
            })}
          </span>
        </div>

        {/* 통계 */}
        <div className="mt-4 flex gap-6 text-sm text-zinc-600">
          <Stat label="Tasks" value={`${doneTasks}/${portfolio.tasks.length} done`} />
          <Stat label="Evidence" value={`${totalCommits} commits`} />
          <Stat label="Verified" value={`${verifiedCommits}/${totalCommits}`} highlight={verifiedCommits > 0} />
        </div>
      </header>

      {/* 태스크 목록 */}
      {portfolio.tasks.length === 0 ? (
        <p className="text-zinc-400">No tasks found.</p>
      ) : (
        <div className="space-y-4">
          {portfolio.tasks.map((task) => (
            <TaskCard key={task.id} task={task} />
          ))}
        </div>
      )}
    </main>
  );
}

function Stat({
  label,
  value,
  highlight,
}: {
  label: string;
  value: string;
  highlight?: boolean;
}) {
  return (
    <div>
      <span className="text-zinc-400">{label}: </span>
      <span className={highlight ? "font-semibold text-green-700" : "font-medium"}>{value}</span>
    </div>
  );
}
