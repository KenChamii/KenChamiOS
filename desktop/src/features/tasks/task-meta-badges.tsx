import { Badge } from "@/components/ui/badge";
import type { Task } from "@/lib/api/tasks";
import { cn } from "@/lib/cn";
import { PRIORITY_LABELS } from "./meta";

export function TaskMetaBadges({ task }: { task: Task }) {
  return (
    <>
      <Badge className={cn(task.priority === "high" && "text-destructive")}>
        {PRIORITY_LABELS[task.priority]}
      </Badge>
      {task.due_at !== null && <Badge>{new Date(task.due_at).toLocaleDateString()}</Badge>}
      {task.tags.map((tag) => (
        <Badge key={tag}>{tag}</Badge>
      ))}
    </>
  );
}
