import { Bot } from "lucide-react";
import { EmptyState } from "@/components/ui/empty-state";
import { useAgentRuns } from "@/hooks/use-agent-runs";
import { errorMessage } from "@/lib/errors";
import { AgentSettingsCard } from "./agent-settings";
import { RunBoard } from "./run-board";
import { StartRun } from "./start-run";

export function AgentTab({ projectId }: { projectId: string }) {
  const runs = useAgentRuns(projectId);
  const all = runs.data ?? [];

  return (
    <div className="flex flex-col gap-6">
      <StartRun projectId={projectId} runs={all} />
      {runs.error && <p className="text-sm text-destructive">{errorMessage(runs.error)}</p>}
      {runs.data && all.length === 0 ? (
        <EmptyState
          icon={Bot}
          area="projects"
          line="No runs yet. Pick a task above and let Claude try it."
        />
      ) : (
        <RunBoard runs={all} projectId={projectId} />
      )}
      <AgentSettingsCard projectId={projectId} />
    </div>
  );
}
