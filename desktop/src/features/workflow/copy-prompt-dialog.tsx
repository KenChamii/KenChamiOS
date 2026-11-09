import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { useCopyPrompt } from "@/hooks/use-prompts";
import type { Prompt } from "@/lib/api/prompts";
import type { ProjectSummary } from "@/lib/api/projects";
import { errorMessage } from "@/lib/errors";

type CopyPromptDialogProps = {
  prompt: Prompt | null;
  /** the variables the app cannot fill in itself */
  variables: string[];
  projects: ProjectSummary[];
  onClose: () => void;
};

export function CopyPromptDialog({ prompt, ...formProps }: CopyPromptDialogProps) {
  return (
    <Dialog open={prompt !== null} onOpenChange={(open) => !open && formProps.onClose()}>
      <DialogContent aria-describedby={undefined}>
        <DialogTitle className="mb-4 text-lg font-semibold">Copy prompt</DialogTitle>
        {prompt && <CopyPromptForm key={prompt.id} prompt={prompt} {...formProps} />}
      </DialogContent>
    </Dialog>
  );
}

type CopyPromptFormProps = Omit<CopyPromptDialogProps, "prompt"> & { prompt: Prompt };

function CopyPromptForm({ prompt, variables, projects, onClose }: CopyPromptFormProps) {
  const copy = useCopyPrompt();
  const [projectId, setProjectId] = useState(prompt.project_id);
  const [values, setValues] = useState<Record<string, string>>({});

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    copy.mutate({ id: prompt.id, projectId, variables: values }, { onSuccess: onClose });
  }

  return (
    <form onSubmit={handleSubmit} className="flex flex-col gap-4">
      <p className="text-sm text-muted-foreground">{prompt.title}</p>
      {prompt.project_id === null && (
        <Field label="Project (optional)">
          <Select
            value={projectId ?? ""}
            options={[
              { value: "", label: "No project" },
              ...projects.map((project) => ({ value: project.id, label: project.name })),
            ]}
            onChange={(id) => setProjectId(id || null)}
          />
        </Field>
      )}
      {variables.map((name, index) => (
        <Field key={name} label={name}>
          <Input
            value={values[name] ?? ""}
            onChange={(e) => setValues({ ...values, [name]: e.target.value })}
            autoFocus={index === 0}
          />
        </Field>
      ))}
      {copy.error && <p className="text-sm text-destructive">{errorMessage(copy.error)}</p>}
      <Button type="submit" disabled={copy.isPending}>
        Copy to clipboard
      </Button>
    </form>
  );
}
