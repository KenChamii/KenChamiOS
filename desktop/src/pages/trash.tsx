import { RotateCcw, Trash2 } from "lucide-react";
import { PageHeader } from "@/components/layout/page-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { useRestore, useTrash } from "@/hooks/use-trash";
import type { TrashItem, TrashKind } from "@/lib/api/trash";
import { timeAgo } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import { stagger } from "@/lib/motion";

const KIND_LABELS: Record<TrashKind, string> = {
  project: "Project",
  task: "Task",
  prompt: "Prompt",
  save_point: "Save point",
  media: "Media",
  habit: "Habit",
};

export function TrashPage() {
  const trash = useTrash();
  const restore = useRestore();

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Trash"
        subtitle="Deleted items, ready to bring back. Restoring a project brings back its tasks and docs."
        icon={Trash2}
      />
      {trash.error && <p className="text-sm text-destructive">{errorMessage(trash.error)}</p>}
      {restore.error && <p className="text-sm text-destructive">{errorMessage(restore.error)}</p>}
      {trash.data && trash.data.length === 0 ? (
        <EmptyState icon={Trash2} line="The trash is empty." />
      ) : (
        <ul className="flex flex-col divide-y divide-border rounded-2xl bg-card shadow-soft">
          {trash.data?.map((item, index) => (
            <TrashRow
              key={`${item.kind}:${item.id}`}
              item={item}
              index={index}
              pending={restore.isPending}
              onRestore={() => restore.mutate({ kind: item.kind, id: item.id })}
            />
          ))}
        </ul>
      )}
    </div>
  );
}

type TrashRowProps = { item: TrashItem; index: number; pending: boolean; onRestore: () => void };

function TrashRow({ item, index, pending, onRestore }: TrashRowProps) {
  return (
    <li className="stagger-in flex items-center gap-3 px-4 py-2" style={stagger(index)}>
      <Badge>{KIND_LABELS[item.kind]}</Badge>
      <span className="min-w-0 flex-1 truncate text-sm">
        {item.title}
        {item.detail && <span className="text-muted-foreground"> · {item.detail}</span>}
      </span>
      <span className="shrink-0 text-xs text-muted-foreground">{timeAgo(item.deleted_at)}</span>
      <Button size="sm" variant="outline" disabled={pending} onClick={onRestore}>
        <RotateCcw size={14} />
        Restore
      </Button>
    </li>
  );
}
