import { useState } from "react";
import { Button } from "@/components/ui/button";
import { useDoc, useSaveDoc } from "@/hooks/use-docs";
import type { DocKind } from "@/lib/api/docs";
import { errorMessage } from "@/lib/errors";
import { MarkdownEditor } from "./markdown-editor";
import { MarkdownPreview } from "./markdown-preview";

type DocTabProps = { projectId: string; kind: DocKind };

export function DocTab({ projectId, kind }: DocTabProps) {
  const doc = useDoc(projectId, kind);
  if (doc.isPending) return <p className="text-sm text-muted-foreground">Loading...</p>;
  if (doc.error) return <p className="text-sm text-destructive">{errorMessage(doc.error)}</p>;
  return <DocEditor projectId={projectId} kind={kind} initialBody={doc.data?.body_md ?? ""} />;
}

type DocEditorProps = DocTabProps & { initialBody: string };

function DocEditor({ projectId, kind, initialBody }: DocEditorProps) {
  const save = useSaveDoc(projectId, kind);
  const [draft, setDraft] = useState(initialBody);
  const [previewing, setPreviewing] = useState(false);
  const [savedBody, setSavedBody] = useState(initialBody);
  const dirty = draft !== savedBody;

  function handleSave() {
    save.mutate(draft, { onSuccess: (doc) => setSavedBody(doc.body_md) });
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center gap-2">
        <Button variant={previewing ? "outline" : "primary"} onClick={() => setPreviewing(false)}>
          Edit
        </Button>
        <Button variant={previewing ? "primary" : "outline"} onClick={() => setPreviewing(true)}>
          Preview
        </Button>
        <span className="ml-auto text-sm text-muted-foreground">
          {save.error ? errorMessage(save.error) : dirty ? "Unsaved changes" : "Saved"}
        </span>
        <Button onClick={handleSave} disabled={!dirty || save.isPending}>
          Save
        </Button>
      </div>
      {previewing ? (
        <MarkdownPreview source={draft} />
      ) : (
        <MarkdownEditor initialValue={draft} onChange={setDraft} />
      )}
    </div>
  );
}
