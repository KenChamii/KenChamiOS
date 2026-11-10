import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listTrash, restoreFromTrash, type TrashKind } from "@/lib/api/trash";

export function useTrash() {
  return useQuery({ queryKey: ["trash"], queryFn: listTrash });
}

export function useRestore() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ kind, id }: { kind: TrashKind; id: string }) => restoreFromTrash(kind, id),
    // a restore can bring back rows of any list
    onSuccess: () => queryClient.invalidateQueries(),
  });
}
