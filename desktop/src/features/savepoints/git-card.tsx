import { GitBranch } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { useGitInfo } from "@/hooks/use-git";
import type { GitInfo } from "@/lib/api/git";
import { timeAgo } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import { branchLabel, shortHash } from "./meta";

export function GitCard({ projectId }: { projectId: string }) {
  const git = useGitInfo(projectId);
  return (
    <Card className="flex flex-col gap-3">
      <h2 className="flex items-center gap-2 font-medium">
        <GitBranch size={16} />
        Git
      </h2>
      {git.error && <p className="text-sm text-destructive">{errorMessage(git.error)}</p>}
      {git.data && <GitDetails info={git.data} />}
    </Card>
  );
}

function GitDetails({ info }: { info: GitInfo }) {
  if (info.state === "no_repo_path") {
    return (
      <p className="text-sm text-muted-foreground">Set a repository folder to see Git info.</p>
    );
  }
  if (info.state === "not_a_repo") {
    return (
      <p className="text-sm text-muted-foreground">
        The repository folder is not a Git repository.
      </p>
    );
  }
  const last = info.commits[0];
  return (
    <div className="flex flex-col gap-2 text-sm">
      <div className="flex flex-wrap items-center gap-2">
        <Badge>{branchLabel(info)}</Badge>
        <span className="text-muted-foreground">
          {info.dirty_files === 0 ? "Clean" : `${info.dirty_files} uncommitted files`}
        </span>
      </div>
      {last && (
        <p>
          <span className="font-mono text-xs text-muted-foreground">{shortHash(last.hash)}</span>{" "}
          {last.summary}{" "}
          <span className="text-muted-foreground">
            by {last.author}, {timeAgo(last.time)}
          </span>
        </p>
      )}
    </div>
  );
}
