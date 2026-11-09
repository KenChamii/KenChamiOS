import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  getBriefTarget,
  listBriefTargets,
  previewBrief,
  renderBrief,
  setBriefTarget,
  writeBrief,
} from "@/lib/api/brief";

export function useBriefTargets() {
  return useQuery({ queryKey: ["brief", "targets"], queryFn: listBriefTargets });
}

export function useBriefTarget(projectId: string) {
  return useQuery({
    queryKey: ["brief", "target", projectId],
    queryFn: () => getBriefTarget(projectId),
  });
}

export function useSetBriefTarget(projectId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (target: string) => setBriefTarget(projectId, target),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["brief", "target", projectId] }),
  });
}

export function useCopyBrief(projectId: string) {
  return useMutation({
    mutationFn: async () => navigator.clipboard.writeText(await renderBrief(projectId)),
  });
}

export function usePreviewBrief(projectId: string) {
  return useMutation({ mutationFn: () => previewBrief(projectId) });
}

export function useWriteBrief(projectId: string) {
  return useMutation({ mutationFn: () => writeBrief(projectId) });
}
