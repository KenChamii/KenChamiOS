import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetDescription, SheetTitle } from "@/components/ui/sheet";
import { Textarea } from "@/components/ui/textarea";
import {
  useApproveRun,
  useDiscardRun,
  useMergeRun,
  useRequeueRun,
  useRunEvents,
  useRunReview,
  useStopRun,
} from "@/hooks/use-agent-runs";
import type { AgentRun } from "@/lib/api/agent";
import { errorMessage } from "@/lib/errors";
import { RunDiffView } from "./run-diff";
import { RunLog } from "./run-log";
import { runSummary } from "./runs";

type Props = { run: AgentRun; projectId: string; onClose: () => void };

export function RunSheet({ run, projectId, onClose }: Props) {
  const [message, setMessage] = useState(`feat: ${run.task_title.toLowerCase()}`);
  const [feedback, setFeedback] = useState("");
  const events = useRunEvents(run.id, run.status === "running" || run.status === "queued");
  const showReview = run.status === "review" || run.status === "failed";
  const review = useRunReview(run.id, showReview && run.worktree_path !== null);
  const approve = useApproveRun(projectId);
  const merge = useMergeRun(projectId);
  const requeue = useRequeueRun(projectId);
  const stop = useStopRun(projectId);
  const discard = useDiscardRun(projectId);

  const error = [approve, merge, requeue, stop, discard].find((action) => action.error)?.error;
  const approved = run.commit_sha !== null;
  const blocked = review.data?.diff.secret != null || review.data?.diff.files.length === 0;

  return (
    <Sheet open onOpenChange={(open) => !open && onClose()}>
      <SheetContent className="flex max-w-2xl flex-col gap-5">
        <div className="flex flex-col gap-1">
          <SheetTitle className="text-xl font-semibold tracking-tight">{run.task_title}</SheetTitle>
          <SheetDescription className="text-sm text-muted-foreground">
            {run.branch ? `${run.branch} · ` : ""}
            {runSummary(run)}
          </SheetDescription>
        </div>

        {run.status === "failed" && run.error && (
          <p className="rounded-xl bg-muted p-3 text-sm text-destructive">{run.error}</p>
        )}

        <section className="flex flex-col gap-2">
          <h3 className="text-label text-muted-foreground">Log</h3>
          <RunLog events={events.data ?? []} />
        </section>

        {showReview && review.data && (
          <section className="flex flex-col gap-2">
            <h3 className="text-label text-muted-foreground">Changes</h3>
            <RunDiffView review={review.data} />
          </section>
        )}
        {review.error && <p className="text-sm text-destructive">{errorMessage(review.error)}</p>}
        {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}

        <div className="mt-auto flex flex-col gap-3">
          {run.status === "review" && !approved && (
            <>
              <Textarea
                aria-label="Commit message"
                className="min-h-10"
                rows={1}
                value={message}
                onChange={(event) => setMessage(event.target.value)}
              />
              <Textarea
                aria-label="What to change"
                placeholder="Ask for changes: say what should be different"
                value={feedback}
                onChange={(event) => setFeedback(event.target.value)}
              />
            </>
          )}
          <div className="flex flex-wrap gap-2">
            {run.status === "review" && !approved && (
              <>
                <Button
                  disabled={blocked || approve.isPending}
                  onClick={() => approve.mutate({ runId: run.id, message })}
                >
                  Approve
                </Button>
                <Button
                  variant="outline"
                  disabled={feedback.trim() === "" || requeue.isPending}
                  onClick={() =>
                    requeue.mutate({ runId: run.id, feedback }, { onSuccess: onClose })
                  }
                >
                  Ask for changes
                </Button>
              </>
            )}
            {run.status === "review" && approved && (
              <Button
                disabled={merge.isPending}
                onClick={() => merge.mutate(run.id, { onSuccess: onClose })}
              >
                Merge into {run.base_branch ?? "the current branch"}
              </Button>
            )}
            {run.status === "failed" && (
              <Button
                variant="outline"
                disabled={requeue.isPending}
                onClick={() =>
                  requeue.mutate({ runId: run.id, feedback: "" }, { onSuccess: onClose })
                }
              >
                Try again
              </Button>
            )}
            {(run.status === "running" || run.status === "queued") && (
              <Button
                variant="destructive"
                disabled={stop.isPending}
                onClick={() => stop.mutate(run.id)}
              >
                Stop
              </Button>
            )}
            {(run.status === "review" || run.status === "failed") && (
              <Button
                variant="ghost"
                disabled={discard.isPending}
                onClick={() => discard.mutate(run.id, { onSuccess: onClose })}
              >
                Discard
              </Button>
            )}
          </div>
        </div>
      </SheetContent>
    </Sheet>
  );
}
