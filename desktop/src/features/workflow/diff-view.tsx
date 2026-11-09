import type { DiffLine } from "@/lib/api/brief";
import { cn } from "@/lib/cn";

const MARKERS = { equal: " ", insert: "+", delete: "-" } as const;

export function DiffView({ lines }: { lines: DiffLine[] }) {
  return (
    <pre className="max-h-96 overflow-auto rounded-md border border-border bg-muted p-2 font-mono text-xs">
      {lines.map((line, index) => (
        <div
          key={index}
          className={cn(
            "whitespace-pre-wrap",
            line.kind === "insert" && "bg-primary/15",
            line.kind === "delete" && "bg-destructive/15",
          )}
        >
          {MARKERS[line.kind]} {line.text}
        </div>
      ))}
    </pre>
  );
}
