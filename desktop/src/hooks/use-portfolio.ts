import { useMutation } from "@tanstack/react-query";
import { exportPortfolio, renderPortfolio } from "@/lib/api/portfolio";

export function usePreviewPortfolio(projectId: string) {
  return useMutation({ mutationFn: () => renderPortfolio(projectId) });
}

export function useCopyPortfolio(projectId: string) {
  return useMutation({
    mutationFn: async () => navigator.clipboard.writeText(await renderPortfolio(projectId)),
  });
}

export function useExportPortfolio(projectId: string) {
  return useMutation({ mutationFn: (folder: string) => exportPortfolio(projectId, folder) });
}
