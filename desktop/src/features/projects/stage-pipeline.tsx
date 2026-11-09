import { Check } from "lucide-react";
import { STAGES, type Stage } from "@/lib/api/projects";
import { cn } from "@/lib/cn";
import { STAGE_LABELS } from "./meta";

type StagePipelineProps = {
  stage: Stage;
  disabled: boolean;
  onChange: (stage: Stage) => void;
};

export function StagePipeline({ stage, disabled, onChange }: StagePipelineProps) {
  const current = STAGES.indexOf(stage);
  return (
    <ol className="flex flex-wrap items-center gap-2" aria-label="Project stage">
      {STAGES.map((value, index) => (
        <li key={value}>
          <button
            type="button"
            disabled={disabled}
            aria-current={value === stage ? "step" : undefined}
            onClick={() => onChange(value)}
            className={cn(
              "flex items-center gap-1.5 rounded-full bg-muted px-3.5 py-1 text-sm transition-colors duration-(--duration-fast) hover:bg-border disabled:opacity-50",
              index < current && "text-muted-foreground",
              value === stage && "bg-primary font-medium text-primary-foreground",
            )}
          >
            {index < current && <Check size={14} />}
            {STAGE_LABELS[value]}
          </button>
        </li>
      ))}
    </ol>
  );
}
