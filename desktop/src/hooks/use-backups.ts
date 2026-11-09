import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  configureBackups,
  getBackupStatus,
  restoreBackup,
  runBackup,
  type BackupStatus,
} from "@/lib/api/backups";

const KEY = ["backups"];

export function useBackupStatus() {
  return useQuery({ queryKey: KEY, queryFn: getBackupStatus });
}

function useStoreStatus() {
  const queryClient = useQueryClient();
  return (status: BackupStatus) => queryClient.setQueryData(KEY, status);
}

export function useConfigureBackups() {
  const store = useStoreStatus();
  return useMutation({
    mutationFn: ({ folder, keep }: { folder: string | null; keep: number }) =>
      configureBackups(folder, keep),
    onSuccess: store,
  });
}

export function useRunBackup() {
  const store = useStoreStatus();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: runBackup,
    onSuccess: store,
    // a failed run is recorded in the status too
    onError: () => queryClient.invalidateQueries({ queryKey: KEY }),
  });
}

export function useRestoreBackup() {
  return useMutation({ mutationFn: restoreBackup });
}
