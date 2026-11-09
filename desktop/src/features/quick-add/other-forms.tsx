import { useState, type FormEvent } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { ChipGroup } from "@/components/ui/chips";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { manualInputOf } from "@/features/media/manual-input";
import { STATUS_LABELS, STATUSES, TOTAL_LABELS, TYPES } from "@/features/media/meta";
import { EMPTY_PROJECT } from "@/features/projects/project-dialog";
import { useAddManualMedia } from "@/hooks/use-media";
import { useCreateProject } from "@/hooks/use-projects";
import { useCreatePrompt } from "@/hooks/use-prompts";
import type { MediaStatus, MediaType } from "@/lib/api/media";
import type { ProjectSummary } from "@/lib/api/projects";
import { errorMessage } from "@/lib/errors";

export function QuickIdeaForm({ onDone }: { onDone: () => void }) {
  const navigate = useNavigate();
  const create = useCreateProject();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    create.mutate(
      { ...EMPTY_PROJECT, name: name.trim(), description },
      {
        onSuccess: (project) => {
          onDone();
          navigate(`/projects/${project.id}`);
        },
      },
    );
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <Field label="Idea">
        <Input value={name} onChange={(e) => setName(e.target.value)} autoFocus required />
      </Field>
      <Field label="Description">
        <Textarea value={description} onChange={(e) => setDescription(e.target.value)} />
      </Field>
      {create.error && <p className="text-sm text-destructive">{errorMessage(create.error)}</p>}
      <Button type="submit" disabled={create.isPending || !name.trim()}>
        Add idea
      </Button>
    </form>
  );
}

type QuickPromptFormProps = {
  projects: ProjectSummary[];
  defaultProjectId: string | null;
  onDone: () => void;
};

export function QuickPromptForm({ projects, defaultProjectId, onDone }: QuickPromptFormProps) {
  const create = useCreatePrompt();
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [projectId, setProjectId] = useState(defaultProjectId ?? "");

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    create.mutate(
      { project_id: projectId || null, title: title.trim(), body_md: body, tags: [] },
      { onSuccess: onDone },
    );
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <Field label="Title">
        <Input value={title} onChange={(e) => setTitle(e.target.value)} autoFocus required />
      </Field>
      <Field label="Prompt">
        <Textarea
          value={body}
          onChange={(e) => setBody(e.target.value)}
          className="min-h-28 font-mono"
        />
      </Field>
      <Field label="Project">
        <Select
          value={projectId}
          options={[
            { value: "", label: "None" },
            ...projects.map((project) => ({ value: project.id, label: project.name })),
          ]}
          onChange={setProjectId}
        />
      </Field>
      {create.error && <p className="text-sm text-destructive">{errorMessage(create.error)}</p>}
      <Button type="submit" disabled={create.isPending || !title.trim()}>
        Add prompt
      </Button>
    </form>
  );
}

export function QuickMediaForm({ onDone }: { onDone: () => void }) {
  const add = useAddManualMedia();
  const [title, setTitle] = useState("");
  const [type, setType] = useState<MediaType>("anime");
  const [status, setStatus] = useState<MediaStatus>("planned");
  const [total, setTotal] = useState("");
  const totalLabel = TOTAL_LABELS[type];
  const input = manualInputOf({ type, status, title, year: "", total, genres: [] });

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    if (input) add.mutate(input, { onSuccess: onDone });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <Field label="Title">
        <Input value={title} onChange={(e) => setTitle(e.target.value)} autoFocus required />
      </Field>
      <ChipGroup
        aria-label="Type"
        value={type}
        options={TYPES.map((option) => ({ value: option.id, label: option.label }))}
        onChange={setType}
      />
      <ChipGroup
        aria-label="Status"
        value={status}
        options={STATUSES.map((value) => ({ value, label: STATUS_LABELS[type][value] }))}
        onChange={setStatus}
      />
      {totalLabel && (
        <Field label={`${totalLabel} (optional)`}>
          <Input
            type="number"
            min={0}
            className="w-40"
            value={total}
            onChange={(e) => setTotal(e.target.value)}
          />
        </Field>
      )}
      {add.error && <p className="text-sm text-destructive">{errorMessage(add.error)}</p>}
      <Button type="submit" disabled={add.isPending || input === null}>
        Add to library
      </Button>
    </form>
  );
}
