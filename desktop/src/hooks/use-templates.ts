import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listTemplates, previewTemplate, resetTemplate, updateTemplate } from "@/lib/api/templates";

export function useTemplates() {
  return useQuery({ queryKey: ["templates"], queryFn: listTemplates });
}

function useInvalidateTemplates() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["templates"] });
}

export function useUpdateTemplate() {
  const invalidate = useInvalidateTemplates();
  return useMutation({
    mutationFn: ({ id, body }: { id: string; body: string }) => updateTemplate(id, body),
    onSuccess: invalidate,
  });
}

export function useResetTemplate() {
  const invalidate = useInvalidateTemplates();
  return useMutation({ mutationFn: resetTemplate, onSuccess: invalidate });
}

export function usePreviewTemplate() {
  return useMutation({ mutationFn: previewTemplate });
}
