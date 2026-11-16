import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import type { AgentRun } from "@/lib/api/agent";
import { BOARD_COLUMNS, groupRuns, runSummary } from "./runs";
import { RunSheet } from "./run-sheet";

export function RunBoard({ runs, projectId }: { runs: AgentRun[]; projectId: string }) {
  const [openId, setOpenId] = useState<string | null>(null);
  const columns = groupRuns(runs);
  const open = runs.find((run) => run.id === openId);

  return (
    <>
      <div className="grid gap-3 md:grid-cols-5">
        {BOARD_COLUMNS.map(({ status, label }) => (
          <div key={status} className="flex flex-col gap-2">
            <h3 className="flex items-center gap-2 text-label text-muted-foreground">
              {label}
              <Badge>{columns[status].length}</Badge>
            </h3>
            {columns[status].map((run) => (
              <button
                key={run.id}
                type="button"
                onClick={() => setOpenId(run.id)}
                className="cursor-pointer text-left focus-visible:outline-2 focus-visible:outline-accent"
              >
                <Card className="flex flex-col gap-1 p-3">
                  <span className="text-sm font-medium">{run.task_title}</span>
                  <span className="line-clamp-2 text-xs text-muted-foreground">
                    {runSummary(run)}
                  </span>
                </Card>
              </button>
            ))}
          </div>
        ))}
      </div>
      {open && <RunSheet run={open} projectId={projectId} onClose={() => setOpenId(null)} />}
    </>
  );
}
