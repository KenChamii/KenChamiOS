import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { PRIORITIES, type Priority, type Task, type TaskUpdate } from "@/lib/api/tasks";
import { fromDateInput, toDateInput } from "@/lib/dates";
import { formatTags, parseTags } from "@/lib/tags";
import { PRIORITY_LABELS } from "./meta";

type TaskFormProps = {
  task: Task;
  pending: boolean;
  error: string | null;
  onSave: (input: TaskUpdate) => void;
};

export function TaskForm({ task, pending, error, onSave }: TaskFormProps) {
  const [title, setTitle] = useState(task.title);
  const [body, setBody] = useState(task.body_md);
  const [acceptance, setAcceptance] = useState(task.acceptance_md);
  const [priority, setPriority] = useState<Priority>(task.priority);
  const [due, setDue] = useState(toDateInput(task.due_at));
  const [tags, setTags] = useState(formatTags(task.tags));
  const [files, setFiles] = useState(task.related_files.join("\n"));

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    onSave({
      title,
      body_md: body,
      acceptance_md: acceptance,
      related_files: files.split("\n"),
      priority,
      due_at: fromDateInput(due),
      tags: parseTags(tags),
    });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-3">
      <Field label="Title">
        <Input value={title} onChange={(e) => setTitle(e.target.value)} required />
      </Field>
      <Field label="Description">
        <Textarea value={body} onChange={(e) => setBody(e.target.value)} />
      </Field>
      <Field label="Acceptance criteria">
        <Textarea value={acceptance} onChange={(e) => setAcceptance(e.target.value)} />
      </Field>
      <div className="grid grid-cols-2 gap-3">
        <Field label="Priority">
          <Select
            value={priority}
            options={PRIORITIES.map((value) => ({ value, label: PRIORITY_LABELS[value] }))}
            onChange={setPriority}
          />
        </Field>
        <Field label="Due date">
          <Input type="date" value={due} onChange={(e) => setDue(e.target.value)} />
        </Field>
      </div>
      <Field label="Tags (comma separated)">
        <Input value={tags} onChange={(e) => setTags(e.target.value)} />
      </Field>
      <Field label="Related files (one per line)">
        <Textarea value={files} onChange={(e) => setFiles(e.target.value)} />
      </Field>
      {error && <p className="text-sm text-destructive">{error}</p>}
      <Button type="submit" disabled={pending || !title.trim()}>
        Save task
      </Button>
    </form>
  );
}
