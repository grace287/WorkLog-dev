import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { CommitTimeline } from "./CommitTimeline";
import type { Task } from "@/lib/types";

const STATUS_CONFIG = {
  todo:  { label: "Todo",  variant: "outline"   } as const,
  doing: { label: "Doing", variant: "warning"   } as const,
  done:  { label: "Done",  variant: "success"   } as const,
};

export function TaskCard({ task }: { task: Task }) {
  const cfg = STATUS_CONFIG[task.status];
  const verifiedCount = task.commits.filter((c) => c.verified).length;

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-center gap-2 flex-wrap">
          <code className="text-xs font-mono text-zinc-500">{task.task_key}</code>
          <Badge variant={cfg.variant}>{cfg.label}</Badge>
          {verifiedCount > 0 && (
            <span className="ml-auto text-xs text-zinc-400">
              {verifiedCount} verified commit{verifiedCount > 1 ? "s" : ""}
            </span>
          )}
        </div>
        <CardTitle className="text-base">{task.title}</CardTitle>
      </CardHeader>
      <CardContent>
        <CommitTimeline commits={task.commits} />
      </CardContent>
    </Card>
  );
}
