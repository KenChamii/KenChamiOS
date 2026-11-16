import { Bot } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Select } from "@/components/ui/select";
import { useStartRun } from "@/hooks/use-agent-runs";
import { useTasks } from "@/hooks/use-tasks";
import type { AgentRun } from "@/lib/api/agent";
import { errorMessage } from "@/lib/errors";
import { runnableTasks } from "./runs";

const NONE = "";

export function StartRun({ projectId, runs }: { projectId: string; runs: AgentRun[] }) {
  const tasks = useTasks(projectId);
  const start = useStartRun(projectId);
  const [taskId, setTaskId] = useState(NONE);
  const options = runnableTasks(tasks.data ?? [], runs);
  const chosen = options.some((task) => task.id === taskId) ? taskId : NONE;

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="flex items-center gap-2 font-semibold tracking-tight">
        <Bot size={16} />
        Run a task with Claude
      </h2>
      <p className="text-sm text-muted-foreground">
        Claude works in its own branch and folder. Your files are not touched until you approve and
        merge.
      </p>
      <div className="flex flex-wrap items-center gap-2">
        <Select
          className="w-full max-w-md"
          aria-label="Task"
          value={chosen}
          options={[
            { value: NONE, label: options.length ? "Pick a task" : "No open tasks to run" },
            ...options.map((task) => ({ value: task.id, label: task.title })),
          ]}
          onChange={setTaskId}
        />
        <Button
          disabled={chosen === NONE || start.isPending}
          onClick={() => start.mutate(chosen, { onSuccess: () => setTaskId(NONE) })}
        >
          Run
        </Button>
      </div>
      {start.error && <p className="text-sm text-destructive">{errorMessage(start.error)}</p>}
    </Card>
  );
}
