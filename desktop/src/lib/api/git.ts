import { invoke } from "@tauri-apps/api/core";

export type CommitInfo = {
  hash: string;
  summary: string;
  author: string;
  time: number;
};

export type RepoInfo = {
  branch: string | null;
  head: string | null;
  dirty_files: number;
  commits: CommitInfo[];
};

export type GitInfo =
  { state: "no_repo_path" } | { state: "not_a_repo" } | ({ state: "repo" } & RepoInfo);

export function getGitInfo(projectId: string) {
  return invoke<GitInfo>("git_info", { projectId });
}
