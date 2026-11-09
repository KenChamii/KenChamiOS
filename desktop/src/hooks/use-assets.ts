import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { addLink, addScreenshot, deleteAsset, listAssets, setAssetCaption } from "@/lib/api/assets";

export function useAssets(projectId: string) {
  return useQuery({ queryKey: ["assets", projectId], queryFn: () => listAssets(projectId) });
}

function useInvalidateAssets(projectId: string) {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["assets", projectId] });
}

export function useAddScreenshot(projectId: string) {
  const invalidate = useInvalidateAssets(projectId);
  return useMutation({
    mutationFn: (sourcePath: string) => addScreenshot(projectId, sourcePath, ""),
    onSuccess: invalidate,
  });
}

export function useAddLink(projectId: string) {
  const invalidate = useInvalidateAssets(projectId);
  return useMutation({
    mutationFn: ({ url, caption }: { url: string; caption: string }) =>
      addLink(projectId, url, caption),
    onSuccess: invalidate,
  });
}

export function useSetAssetCaption(projectId: string) {
  const invalidate = useInvalidateAssets(projectId);
  return useMutation({
    mutationFn: ({ id, caption }: { id: string; caption: string }) => setAssetCaption(id, caption),
    onSuccess: invalidate,
  });
}

export function useDeleteAsset(projectId: string) {
  const invalidate = useInvalidateAssets(projectId);
  return useMutation({ mutationFn: deleteAsset, onSuccess: invalidate });
}
