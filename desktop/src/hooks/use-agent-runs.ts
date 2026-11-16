import { listen } from "@tauri-apps/api/event";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import {
  approveRun,
  discardRun,
  getAgentSettings,
  getRunLog,
  getRunReview,
  listRuns,
  mergeRun,
  requeueRun,
  setAgentSettings,
  startRun,
  stopRun,
  type AgentSettings,
} from "@/lib/api/agent";

const runsKey = (projectId: string) => ["agent-runs", projectId];

export function useAgentSettings(projectId: string) {
  return useQuery({
    queryKey: ["agent-settings", projectId],
    queryFn: () => getAgentSettings(projectId),
  });
}

export function useSaveAgentSettings(projectId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (settings: AgentSettings) => setAgentSettings(projectId, settings),
    onSuccess: (saved) => queryClient.setQueryData(["agent-settings", projectId], saved),
  });
}

/** The project's runs, refreshed whenever the core says a run changed. */
export function useAgentRuns(projectId: string) {
  const queryClient = useQueryClient();
  useEffect(() => {
    const unlisten = listen("agent-run-changed", () => {
      void queryClient.invalidateQueries({ queryKey: runsKey(projectId) });
      void queryClient.invalidateQueries({ queryKey: ["tasks", projectId] });
      void queryClient.invalidateQueries({ queryKey: ["git", projectId] });
      void queryClient.invalidateQueries({ queryKey: ["save-points", projectId] });
    });
    return () => void unlisten.then((stop) => stop());
  }, [queryClient, projectId]);
  return useQuery({ queryKey: runsKey(projectId), queryFn: () => listRuns(projectId) });
}

/** The run's log, read again every second while it is going. */
export function useRunEvents(runId: string, running: boolean) {
  return useQuery({
    queryKey: ["agent-log", runId],
    queryFn: () => getRunLog(runId),
    refetchInterval: running ? 1000 : false,
  });
}

export function useRunReview(runId: string, enabled: boolean) {
  return useQuery({
    queryKey: ["agent-review", runId],
    queryFn: () => getRunReview(runId),
    enabled,
  });
}

function useRunAction<T>(projectId: string, action: (arg: T) => Promise<unknown>) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: action,
    onSettled: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: runsKey(projectId) }),
        queryClient.invalidateQueries({ queryKey: ["agent-review"] }),
        queryClient.invalidateQueries({ queryKey: ["agent-log"] }),
        queryClient.invalidateQueries({ queryKey: ["tasks", projectId] }),
        queryClient.invalidateQueries({ queryKey: ["git", projectId] }),
        queryClient.invalidateQueries({ queryKey: ["save-points", projectId] }),
        queryClient.invalidateQueries({ queryKey: ["projects"] }),
        queryClient.invalidateQueries({ queryKey: ["dashboard"] }),
      ]),
  });
}

export function useStartRun(projectId: string) {
  return useRunAction(projectId, (taskId: string) => startRun(taskId));
}

export function useApproveRun(projectId: string) {
  return useRunAction(projectId, (input: { runId: string; message: string }) =>
    approveRun(input.runId, input.message),
  );
}

export function useMergeRun(projectId: string) {
  return useRunAction(projectId, (runId: string) => mergeRun(runId));
}

export function useRequeueRun(projectId: string) {
  return useRunAction(projectId, (input: { runId: string; feedback: string }) =>
    requeueRun(input.runId, input.feedback),
  );
}

export function useStopRun(projectId: string) {
  return useRunAction(projectId, (runId: string) => stopRun(runId));
}

export function useDiscardRun(projectId: string) {
  return useRunAction(projectId, (runId: string) => discardRun(runId));
}
