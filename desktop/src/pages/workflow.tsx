import { PromptLibrary } from "@/features/workflow/prompt-library";

export function WorkflowPage() {
  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-2xl font-semibold">Workflow</h1>
      <PromptLibrary />
    </div>
  );
}
