import type { RunReview } from "@/lib/api/agent";
import { cn } from "@/lib/cn";

const STATUS_LABEL = { added: "A", deleted: "D", renamed: "R", modified: "M" } as const;

function lineTone(line: string) {
  if (line.startsWith("+") && !line.startsWith("+++")) return "text-accent";
  if (line.startsWith("-") && !line.startsWith("---")) return "text-destructive";
  if (line.startsWith("@@") || line.startsWith("diff ")) return "text-muted-foreground";
  return "";
}

export function RunDiffView({ review }: { review: RunReview }) {
  const { diff, test_output } = review;
  if (diff.files.length === 0) {
    return <p className="text-sm text-muted-foreground">The agent changed no files.</p>;
  }
  return (
    <div className="flex flex-col gap-3">
      {diff.secret && (
        <p className="rounded-xl bg-muted p-3 text-sm text-destructive">
          Looks like a {diff.secret.rule} in {diff.secret.path}. Approve is blocked until it is
          removed.
        </p>
      )}
      <ul className="flex flex-col gap-1 text-sm">
        {diff.files.map((file) => (
          <li key={file.path} className="flex items-center gap-2">
            <span className="w-4 font-mono text-xs text-muted-foreground">
              {STATUS_LABEL[file.status]}
            </span>
            <span className="truncate">{file.path}</span>
          </li>
        ))}
      </ul>
      <pre className="max-h-96 overflow-auto rounded-xl bg-muted p-3 font-mono text-xs">
        {diff.patch.split("\n").map((line, index) => (
          <div key={index} className={cn("whitespace-pre-wrap break-all", lineTone(line))}>
            {line || " "}
          </div>
        ))}
        {diff.truncated && <div className="text-muted-foreground">... patch cut short</div>}
      </pre>
      {test_output !== null && (
        <div className="flex flex-col gap-1">
          <h3 className="text-label text-muted-foreground">Test output</h3>
          <pre className="max-h-60 overflow-auto whitespace-pre-wrap rounded-xl bg-muted p-3 font-mono text-xs">
            {test_output || "No output."}
          </pre>
        </div>
      )}
    </div>
  );
}
