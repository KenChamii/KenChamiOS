import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  confirmImport,
  exportCsv,
  exportExists,
  previewImport,
  type ImportFormat,
} from "@/lib/api/media-transfer";

export function usePreviewImport() {
  return useMutation({
    mutationFn: (args: { path: string; format: ImportFormat }) =>
      previewImport(args.path, args.format),
  });
}

export function useConfirmImport() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (args: { path: string; format: ImportFormat }) =>
      confirmImport(args.path, args.format),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["media"] }),
  });
}

export function useExportExists() {
  return useMutation({ mutationFn: exportExists });
}

export function useExportCsv() {
  return useMutation({
    mutationFn: (args: { folder: string; overwrite: boolean }) =>
      exportCsv(args.folder, args.overwrite),
  });
}
