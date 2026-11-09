import { useQuery } from "@tanstack/react-query";
import { getGitInfo } from "@/lib/api/git";

export function useGitInfo(projectId: string) {
  return useQuery({ queryKey: ["git", projectId], queryFn: () => getGitInfo(projectId) });
}
