import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  copyPrompt,
  createPrompt,
  deletePrompt,
  getTaskPrompt,
  listPromptVariables,
  listPrompts,
  setPromptFavorite,
  updatePrompt,
  type PromptInput,
} from "@/lib/api/prompts";

export function usePrompts() {
  return useQuery({ queryKey: ["prompts"], queryFn: listPrompts });
}

function useInvalidatePrompts() {
  const queryClient = useQueryClient();
  return () => queryClient.invalidateQueries({ queryKey: ["prompts"] });
}

export function useCreatePrompt() {
  const invalidate = useInvalidatePrompts();
  return useMutation({ mutationFn: createPrompt, onSuccess: invalidate });
}

export function useUpdatePrompt() {
  const invalidate = useInvalidatePrompts();
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: PromptInput }) => updatePrompt(id, input),
    onSuccess: invalidate,
  });
}

export function useSetPromptFavorite() {
  const invalidate = useInvalidatePrompts();
  return useMutation({
    mutationFn: ({ id, favorite }: { id: string; favorite: boolean }) =>
      setPromptFavorite(id, favorite),
    onSuccess: invalidate,
  });
}

export function useDeletePrompt() {
  const invalidate = useInvalidatePrompts();
  return useMutation({ mutationFn: deletePrompt, onSuccess: invalidate });
}

export function usePromptVariables() {
  return useMutation({ mutationFn: listPromptVariables });
}

type CopyArgs = { id: string; projectId: string | null; variables: Record<string, string> };

export function useCopyPrompt() {
  const invalidate = useInvalidatePrompts();
  return useMutation({
    mutationFn: async ({ id, projectId, variables }: CopyArgs) =>
      navigator.clipboard.writeText(await copyPrompt(id, projectId, variables)),
    onSuccess: invalidate,
  });
}

export function useCopyTaskPrompt() {
  return useMutation({
    mutationFn: async (taskId: string) =>
      navigator.clipboard.writeText(await getTaskPrompt(taskId)),
  });
}
