import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { getDoc, upsertDoc, type DocKind } from "@/lib/api/docs";

export function useDoc(projectId: string, kind: DocKind) {
  return useQuery({
    queryKey: ["docs", projectId, kind],
    queryFn: () => getDoc(projectId, kind),
  });
}

export function useSaveDoc(projectId: string, kind: DocKind) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (bodyMd: string) => upsertDoc(projectId, kind, "", bodyMd),
    onSuccess: (doc) => {
      queryClient.setQueryData(["docs", projectId, kind], doc);
      return queryClient.invalidateQueries({ queryKey: ["projects"] });
    },
  });
}
