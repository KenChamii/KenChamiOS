import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen } from "lucide-react";
import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import type { ProjectInput } from "@/lib/api/projects";
import { cn } from "@/lib/cn";
import { formatTags, parseTags } from "@/lib/tags";
import { PROJECT_COLORS } from "./meta";

type ProjectFormProps = {
  initial: ProjectInput;
  submitLabel: string;
  pending: boolean;
  error: string | null;
  onSubmit: (input: ProjectInput) => void;
};

export function ProjectForm({ initial, submitLabel, pending, error, onSubmit }: ProjectFormProps) {
  const [name, setName] = useState(initial.name);
  const [description, setDescription] = useState(initial.description);
  const [repoPath, setRepoPath] = useState(initial.repo_path ?? "");
  const [tags, setTags] = useState(formatTags(initial.tags));
  const [color, setColor] = useState(initial.color);

  async function pickFolder() {
    const folder = await open({ directory: true, multiple: false });
    if (folder) setRepoPath(folder);
  }

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    onSubmit({
      name,
      description,
      repo_path: repoPath.trim() || null,
      color,
      tags: parseTags(tags),
      archived: initial.archived,
    });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <Field label="Name">
        <Input value={name} onChange={(e) => setName(e.target.value)} autoFocus required />
      </Field>
      <Field label="Description">
        <Textarea value={description} onChange={(e) => setDescription(e.target.value)} />
      </Field>
      <Field label="Repository folder">
        <div className="flex gap-2">
          <Input value={repoPath} onChange={(e) => setRepoPath(e.target.value)} />
          <Button
            type="button"
            variant="outline"
            size="icon"
            aria-label="Browse"
            onClick={pickFolder}
          >
            <FolderOpen size={16} />
          </Button>
        </div>
      </Field>
      <Field label="Tags (comma separated)">
        <Input value={tags} onChange={(e) => setTags(e.target.value)} />
      </Field>
      <div className="flex flex-col gap-1 text-sm">
        <span className="font-medium">Color</span>
        <div className="flex gap-2">
          {PROJECT_COLORS.map((value) => (
            <button
              key={value}
              type="button"
              aria-label={value}
              aria-pressed={color === value}
              onClick={() => setColor(color === value ? null : value)}
              className={cn(
                "h-6 w-6 rounded-full border-2 border-transparent",
                color === value && "border-foreground",
              )}
              style={{ backgroundColor: value }}
            />
          ))}
        </div>
      </div>
      {error && <p className="text-sm text-destructive">{error}</p>}
      <Button type="submit" disabled={pending || !name.trim()}>
        {submitLabel}
      </Button>
    </form>
  );
}
