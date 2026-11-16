import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  getBriefs,
  getMcpStatus,
  newMcpToken,
  runBrief,
  setBriefEnabled,
  setMcpEnabled,
  undoMcpCall,
  type BriefKind,
  type McpStatus,
} from "@/lib/api/mcp";

const STATUS_KEY = ["mcp", "status"];
const BRIEFS_KEY = ["mcp", "briefs"];

export function useMcpStatus() {
  // the call list changes while Claude works, so it is read again every few seconds
  return useQuery({ queryKey: STATUS_KEY, queryFn: getMcpStatus, refetchInterval: 5000 });
}

function useStatusMutation<T>(action: (arg: T) => Promise<McpStatus>) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: action,
    onSuccess: (status) => queryClient.setQueryData(STATUS_KEY, status),
  });
}

export function useSetMcpEnabled() {
  return useStatusMutation((enabled: boolean) => setMcpEnabled(enabled));
}

export function useNewMcpToken() {
  return useStatusMutation(() => newMcpToken());
}

export function useUndoMcpCall() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: undoMcpCall,
    // the undone change shows up everywhere, so everything is read again
    onSuccess: () => queryClient.invalidateQueries(),
  });
}

export function useBriefs() {
  return useQuery({ queryKey: BRIEFS_KEY, queryFn: getBriefs });
}

export function useSetBrief() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: { kind: BriefKind; enabled: boolean }) =>
      setBriefEnabled(input.kind, input.enabled),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: BRIEFS_KEY }),
  });
}

export function useRunBrief() {
  return useMutation({ mutationFn: (kind: BriefKind) => runBrief(kind) });
}
