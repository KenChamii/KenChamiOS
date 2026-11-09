import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { useCreateTimeEntry, useUpdateTimeEntry } from "@/hooks/use-time-entries";
import type { TimeEntry } from "@/lib/api/time-entries";
import type { Task } from "@/lib/api/tasks";
import { errorMessage } from "@/lib/errors";
import { draftOf, inputOf } from "./entry-input";

type EntryDialogProps = {
  projectId: string;
  tasks: Task[];
  // null adds a new entry
  entry: TimeEntry | null;
  open: boolean;
  onClose: () => void;
};

export function EntryDialog({ open, onClose, ...form }: EntryDialogProps) {
  return (
    <Dialog open={open} onOpenChange={(next) => !next && onClose()}>
      <DialogContent aria-describedby={undefined}>
        <DialogTitle className="mb-4 text-lg font-semibold">
          {form.entry ? "Edit time entry" : "Add time entry"}
        </DialogTitle>
        {open && <EntryForm key={form.entry?.id ?? "new"} {...form} onDone={onClose} />}
      </DialogContent>
    </Dialog>
  );
}

type EntryFormProps = Omit<EntryDialogProps, "open" | "onClose"> & { onDone: () => void };

function EntryForm({ projectId, tasks, entry, onDone }: EntryFormProps) {
  const create = useCreateTimeEntry(projectId);
  const update = useUpdateTimeEntry(projectId);
  const [draft, setDraft] = useState(() => draftOf(entry, Date.now()));
  const input = inputOf(draft, entry);
  const pending = create.isPending || update.isPending;
  const error = create.error ?? update.error;

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (!input) return;
    if (entry) update.mutate({ id: entry.id, input }, { onSuccess: onDone });
    else create.mutate(input, { onSuccess: onDone });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-3">
      <div className="grid grid-cols-2 gap-3">
        <Field label="Date">
          <Input
            type="date"
            required
            value={draft.date}
            onChange={(e) => setDraft({ ...draft, date: e.target.value })}
          />
        </Field>
        <Field label="Minutes">
          <Input
            type="number"
            min={1}
            max={1440}
            required
            value={draft.minutes}
            onChange={(e) => setDraft({ ...draft, minutes: e.target.value })}
          />
        </Field>
      </div>
      <Field label="Task">
        <Select
          value={draft.taskId}
          onChange={(e) => setDraft({ ...draft, taskId: e.target.value })}
        >
          <option value="">No task</option>
          {tasks.map((task) => (
            <option key={task.id} value={task.id}>
              {task.title}
            </option>
          ))}
        </Select>
      </Field>
      <Field label="Note">
        <Input value={draft.note} onChange={(e) => setDraft({ ...draft, note: e.target.value })} />
      </Field>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <div className="flex justify-end gap-2">
        <Button type="button" variant="ghost" onClick={onDone}>
          Cancel
        </Button>
        <Button type="submit" disabled={!input || pending}>
          Save
        </Button>
      </div>
    </form>
  );
}
