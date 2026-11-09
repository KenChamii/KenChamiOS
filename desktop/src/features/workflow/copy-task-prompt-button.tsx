import { Check, ClipboardCopy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useCopyTaskPrompt } from "@/hooks/use-prompts";
import { cn } from "@/lib/cn";
import { errorMessage } from "@/lib/errors";

type CopyTaskPromptButtonProps = { taskId: string; showLabel?: boolean };

export function CopyTaskPromptButton({ taskId, showLabel = false }: CopyTaskPromptButtonProps) {
  const copy = useCopyTaskPrompt();
  const label = copy.error ? errorMessage(copy.error) : "Copy as prompt";

  return (
    <Button
      type="button"
      variant={showLabel ? "outline" : "ghost"}
      size={showLabel ? "sm" : "icon"}
      className={cn(copy.error && "text-destructive")}
      aria-label={label}
      title={label}
      disabled={copy.isPending}
      // the card and the row behind the button open the task on click
      onClick={(event) => {
        event.stopPropagation();
        copy.mutate(taskId);
      }}
    >
      {copy.isSuccess ? <Check size={16} /> : <ClipboardCopy size={16} />}
      {showLabel && "Copy as prompt"}
    </Button>
  );
}
