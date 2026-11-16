import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Sheet, SheetContent, SheetDescription, SheetTitle } from "@/components/ui/sheet";
import { Textarea } from "@/components/ui/textarea";
import { useCreateJob, useDeleteJob, useMoveJob, useUpdateJob } from "@/hooks/use-jobs";
import { JOB_SOURCES, JOB_STAGES, type JobInput, type JobView } from "@/lib/api/jobs";
import { errorMessage } from "@/lib/errors";
import { EMPTY_JOB, canSave, fromLocalInput, inputOf, toLocalInput } from "./fields";
import { SOURCE_LABELS, STAGE_LABELS } from "./stages";

type Props = { job: JobView | null; onClose: () => void };

/** The form for one application; `job` is null when adding a new one. */
export function JobSheet({ job, onClose }: Props) {
  const [draft, setDraft] = useState<JobInput>(job ? inputOf(job) : EMPTY_JOB);
  const create = useCreateJob();
  const update = useUpdateJob();
  const move = useMoveJob();
  const remove = useDeleteJob();
  const change = (patch: Partial<JobInput>) => setDraft({ ...draft, ...patch });
  const error = create.error ?? update.error ?? move.error ?? remove.error;

  function save() {
    const done = { onSuccess: onClose };
    if (job) update.mutate({ id: job.id, fields: draft }, done);
    else create.mutate(draft, done);
  }

  return (
    <Sheet open onOpenChange={(open) => !open && onClose()}>
      <SheetContent className="flex flex-col gap-4">
        <div className="flex flex-col gap-1">
          <SheetTitle className="text-xl font-semibold tracking-tight">
            {job ? `${job.role} at ${job.company}` : "New application"}
          </SheetTitle>
          <SheetDescription className="text-sm text-muted-foreground">
            {job ? "Edit the details, or move it to another stage." : "Save a posting you found."}
          </SheetDescription>
        </div>
        {job && (
          <Field label="Stage">
            <Select
              aria-label="Stage"
              value={job.stage}
              options={JOB_STAGES.map((stage) => ({ value: stage, label: STAGE_LABELS[stage] }))}
              onChange={(stage) => move.mutate({ id: job.id, stage })}
            />
          </Field>
        )}
        <div className="grid grid-cols-2 gap-3">
          <Field label="Company">
            <Input value={draft.company} onChange={(e) => change({ company: e.target.value })} />
          </Field>
          <Field label="Role">
            <Input value={draft.role} onChange={(e) => change({ role: e.target.value })} />
          </Field>
          <Field label="Location">
            <Input value={draft.location} onChange={(e) => change({ location: e.target.value })} />
          </Field>
          <Field label="Salary range">
            <Input value={draft.salary} onChange={(e) => change({ salary: e.target.value })} />
          </Field>
          <Field label="Source">
            <Select
              aria-label="Source"
              value={draft.source}
              options={JOB_SOURCES.map((source) => ({
                value: source,
                label: SOURCE_LABELS[source],
              }))}
              onChange={(source) => change({ source })}
            />
          </Field>
          <Field label="Applied on">
            <Input
              type="date"
              value={draft.applied_on ?? ""}
              onChange={(e) => change({ applied_on: e.target.value || null })}
            />
          </Field>
          <Field label="Contact">
            <Input value={draft.contact} onChange={(e) => change({ contact: e.target.value })} />
          </Field>
          <Field label="Resume version">
            <Input
              value={draft.resume_version}
              onChange={(e) => change({ resume_version: e.target.value })}
            />
          </Field>
        </div>
        <Field label="Posting address">
          <div className="flex gap-2">
            <Input
              value={draft.url}
              placeholder="https://"
              onChange={(e) => change({ url: e.target.value })}
            />
            {job?.url && (
              <Button variant="outline" onClick={() => void openUrl(job.url)}>
                Open
              </Button>
            )}
          </div>
        </Field>
        <div className="grid grid-cols-2 gap-3">
          <Field label="Next step">
            <Input
              value={draft.next_step}
              onChange={(e) => change({ next_step: e.target.value })}
            />
          </Field>
          <Field label="Next step at">
            <Input
              type="datetime-local"
              value={toLocalInput(draft.next_step_at)}
              onChange={(e) => change({ next_step_at: fromLocalInput(e.target.value) })}
            />
          </Field>
        </div>
        <Field label="Notes">
          <Textarea
            className="min-h-40"
            value={draft.notes_md}
            onChange={(e) => change({ notes_md: e.target.value })}
          />
        </Field>
        {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
        <div className="mt-auto flex flex-wrap gap-2">
          <Button disabled={!canSave(draft) || create.isPending || update.isPending} onClick={save}>
            Save
          </Button>
          {job && (
            <Button
              variant="ghost"
              disabled={remove.isPending}
              onClick={() => remove.mutate(job.id, { onSuccess: onClose })}
            >
              Delete
            </Button>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
