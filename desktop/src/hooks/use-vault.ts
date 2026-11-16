import { listen } from "@tauri-apps/api/event";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import {
  getGraph,
  getVaultStatus,
  previewVault,
  setVaultEnabled,
  setVaultExcluded,
  setVaultPath,
  syncVaultNow,
  type VaultReport,
} from "@/lib/api/vault";
import { useToasts } from "@/stores/toasts";

const STATUS_KEY = ["vault", "status"];

export function useVaultStatus() {
  return useQuery({ queryKey: STATUS_KEY, queryFn: getVaultStatus });
}

/** Tells the user when a background sync changed the app or hit a conflict. */
export function useVaultEvents() {
  const queryClient = useQueryClient();
  useEffect(() => {
    const unlisten = listen<VaultReport>("vault-synced", ({ payload }) => {
      void queryClient.invalidateQueries();
      if (payload.changes.some((change) => change.kind === "conflict")) {
        useToasts
          .getState()
          .show("Obsidian and the app both changed a note. The vault version was kept.");
      }
    });
    return () => void unlisten.then((stop) => stop());
  }, [queryClient]);
}

export function useConfigureVault() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (input: { path: string; enabled: boolean; excluded: string[] }) => {
      await setVaultPath(input.path);
      await setVaultExcluded(input.excluded);
      await setVaultEnabled(input.enabled);
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: STATUS_KEY }),
  });
}

export function usePreviewVault() {
  return useMutation({ mutationFn: previewVault });
}

export function useSyncVault() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: syncVaultNow,
    onSuccess: () => queryClient.invalidateQueries(),
  });
}

export function useGraph() {
  return useQuery({ queryKey: ["vault", "graph"], queryFn: getGraph, staleTime: 0 });
}
