import type { VaultReport } from "@/lib/api/vault";
import { describeChange } from "./describe";

export function VaultReportView({ report, applied }: { report: VaultReport; applied: boolean }) {
  return (
    <div className="flex flex-col gap-2 rounded-xl bg-muted p-3 text-sm">
      <p className="font-medium">
        {applied ? "Done" : "Preview"}: {report.changes.length} changed, {report.skipped} already in
        step
        {report.inbox_imported > 0 && `, ${report.inbox_imported} inbox lines imported`}
      </p>
      {report.changes.length > 0 && (
        <ul className="flex max-h-60 flex-col gap-1 overflow-auto">
          {report.changes.map((change) => (
            <li key={`${change.path}-${change.kind}`} className="flex gap-2">
              <span className="shrink-0 text-muted-foreground">{describeChange(change)}</span>
              <span className="truncate">{change.path}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
