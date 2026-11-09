import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Textarea } from "@/components/ui/textarea";
import { usePreviewTemplate, useResetTemplate, useUpdateTemplate } from "@/hooks/use-templates";
import type { ExportTemplate } from "@/lib/api/templates";
import { errorMessage } from "@/lib/errors";
import { TEMPLATE_HELP, TEMPLATE_TITLES } from "./meta";

export function TemplateEditor({ template }: { template: ExportTemplate }) {
  const [body, setBody] = useState(template.body);
  const update = useUpdateTemplate();
  const reset = useResetTemplate();
  const preview = usePreviewTemplate();

  const error = update.error ?? reset.error ?? preview.error;

  return (
    <Card className="flex flex-col gap-3">
      <h3 className="font-medium">{TEMPLATE_TITLES[template.kind]}</h3>
      <p className="text-sm text-muted-foreground">{TEMPLATE_HELP[template.kind]}</p>
      <Textarea
        value={body}
        onChange={(event) => setBody(event.target.value)}
        className="min-h-64 font-mono"
        aria-label={`${TEMPLATE_TITLES[template.kind]} template`}
        spellCheck={false}
      />
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <div className="flex flex-wrap gap-2">
        <Button
          disabled={update.isPending || body === template.body}
          onClick={() => update.mutate({ id: template.id, body })}
        >
          Save
        </Button>
        <Button variant="outline" disabled={preview.isPending} onClick={() => preview.mutate(body)}>
          Preview
        </Button>
        <Button
          variant="ghost"
          disabled={reset.isPending}
          onClick={() => reset.mutate(template.id)}
        >
          Reset to default
        </Button>
      </div>
      {preview.data !== undefined && (
        <div className="flex flex-col gap-1">
          <span className="text-sm font-medium">Preview with a sample project</span>
          <pre className="max-h-80 overflow-auto whitespace-pre-wrap rounded-xl bg-muted p-3 text-xs">
            {preview.data}
          </pre>
        </div>
      )}
    </Card>
  );
}
