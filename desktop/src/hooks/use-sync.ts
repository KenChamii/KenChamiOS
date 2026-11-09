import { listen } from "@tauri-apps/api/event";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import {
  cancelPairing,
  getSyncStatus,
  revokeDevice,
  setSyncEnabled,
  startPairing,
} from "@/lib/api/sync";

const STATUS_REFRESH_MS = 5_000;

export function useSyncStatus() {
  return useQuery({
    queryKey: ["sync-status"],
    queryFn: getSyncStatus,
    refetchInterval: STATUS_REFRESH_MS,
  });
}

function useInvalidateStatus() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["sync-status"] });
}

export function useSetSyncEnabled() {
  const invalidate = useInvalidateStatus();
  return useMutation({ mutationFn: setSyncEnabled, onSuccess: invalidate });
}

export function useStartPairing() {
  const invalidate = useInvalidateStatus();
  return useMutation({ mutationFn: startPairing, onSuccess: invalidate });
}

export function useCancelPairing() {
  const invalidate = useInvalidateStatus();
  return useMutation({ mutationFn: cancelPairing, onSuccess: invalidate });
}

export function useRevokeDevice() {
  const invalidate = useInvalidateStatus();
  return useMutation({ mutationFn: revokeDevice, onSuccess: invalidate });
}

/** Refreshes everything on screen when a phone sent changes. */
export function useRefreshOnSync() {
  const queryClient = useQueryClient();
  useEffect(() => {
    const unlisten = listen("sync-changed", () => queryClient.invalidateQueries());
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [queryClient]);
}
