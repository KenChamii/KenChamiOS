import type { RepoInfo } from "@/lib/api/git";

export function shortHash(hash: string): string {
  return hash.slice(0, 7);
}

export function branchLabel(repo: Pick<RepoInfo, "branch" | "head">): string {
  if (repo.branch) return repo.branch;
  return repo.head ? "Detached HEAD" : "No commits yet";
}

/** "main @ a1b2c3d", or whichever part exists, or null for neither. */
export function commitLabel(branch: string | null, commit: string | null): string | null {
  const parts = [branch, commit && `@ ${shortHash(commit)}`].filter(Boolean);
  return parts.length > 0 ? parts.join(" ") : null;
}
