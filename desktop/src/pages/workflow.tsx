import { GitBranch } from "lucide-react";
import { PageHeader } from "@/components/layout/page-header";
import { PromptLibrary } from "@/features/workflow/prompt-library";

export function WorkflowPage() {
  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Workflow"
        subtitle="Prompts and briefs for working with AI"
        icon={GitBranch}
        area="workflow"
      />
      <PromptLibrary />
    </div>
  );
}
