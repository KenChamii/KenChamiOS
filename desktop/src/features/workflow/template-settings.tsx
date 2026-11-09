import { useTemplates } from "@/hooks/use-templates";
import { errorMessage } from "@/lib/errors";
import { TemplateEditor } from "./template-editor";

export function TemplateSettings() {
  const templates = useTemplates();

  return (
    <section className="flex flex-col gap-4">
      <h2 className="text-lg font-semibold">Templates</h2>
      {templates.error && (
        <p className="text-sm text-destructive">{errorMessage(templates.error)}</p>
      )}
      {/* keyed by the saved body, so a save or reset restarts the editor from the new text */}
      {templates.data?.map((template) => (
        <TemplateEditor key={`${template.id}:${template.body}`} template={template} />
      ))}
    </section>
  );
}
