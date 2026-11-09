import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { useCreatePrompt, useUpdatePrompt } from "@/hooks/use-prompts";
import type { Prompt, PromptInput } from "@/lib/api/prompts";
import type { ProjectSummary } from "@/lib/api/projects";
import { errorMessage } from "@/lib/errors";
import { formatTags, parseTags } from "@/lib/tags";

type PromptDialogProps = {
  /** the prompt to edit, or null to create one */
  prompt: Prompt | null;
  /** project preselected for a new prompt */
  defaultProjectId: string | null;
  projects: ProjectSummary[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
};

export function PromptDialog({ open, onOpenChange, ...formProps }: PromptDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent aria-describedby={undefined} className="max-w-xl">
        <DialogTitle className="mb-4 text-lg font-semibold">
          {formProps.prompt ? "Edit prompt" : "New prompt"}
        </DialogTitle>
        <PromptForm {...formProps} onDone={() => onOpenChange(false)} />
      </DialogContent>
    </Dialog>
  );
}

type PromptFormProps = Omit<PromptDialogProps, "open" | "onOpenChange"> & { onDone: () => void };

function PromptForm({ prompt, defaultProjectId, projects, onDone }: PromptFormProps) {
  const create = useCreatePrompt();
  const update = useUpdatePrompt();
  const [title, setTitle] = useState(prompt?.title ?? "");
  const [projectId, setProjectId] = useState(prompt ? prompt.project_id : defaultProjectId);
  const [tags, setTags] = useState(formatTags(prompt?.tags ?? []));
  const [body, setBody] = useState(prompt?.body_md ?? "");

  const pending = create.isPending || update.isPending;
  const error = create.error ?? update.error;

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const input: PromptInput = {
      project_id: projectId,
      title,
      body_md: body,
      tags: parseTags(tags),
    };
    if (prompt) update.mutate({ id: prompt.id, input }, { onSuccess: onDone });
    else create.mutate(input, { onSuccess: onDone });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <Field label="Title">
        <Input value={title} onChange={(e) => setTitle(e.target.value)} autoFocus required />
      </Field>
      <Field label="Project">
        <Select
          value={projectId ?? ""}
          options={[
            { value: "", label: "Global" },
            ...projects.map((project) => ({ value: project.id, label: project.name })),
          ]}
          onChange={(id) => setProjectId(id || null)}
        />
      </Field>
      <Field label="Tags (comma separated)">
        <Input value={tags} onChange={(e) => setTags(e.target.value)} />
      </Field>
      <Field label="Prompt">
        <Textarea
          value={body}
          onChange={(e) => setBody(e.target.value)}
          className="min-h-40 font-mono"
          placeholder="Use {{ project.name }}, {{ git.branch }} or your own {{ variables }}"
        />
      </Field>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <Button type="submit" disabled={pending || !title.trim()}>
        {prompt ? "Save" : "Create prompt"}
      </Button>
    </form>
  );
}
