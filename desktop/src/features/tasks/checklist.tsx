import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  useAddChecklistItem,
  useChecklist,
  useDeleteChecklistItem,
  useUpdateChecklistItem,
} from "@/hooks/use-checklist";
import { cn } from "@/lib/cn";
import { AddItemForm } from "./add-item-form";

export function Checklist({ taskId }: { taskId: string }) {
  const items = useChecklist(taskId);
  const add = useAddChecklistItem(taskId);
  const update = useUpdateChecklistItem(taskId);
  const remove = useDeleteChecklistItem(taskId);

  return (
    <section className="flex flex-col gap-2">
      <h3 className="text-sm font-medium">Checklist</h3>
      <ul className="flex flex-col gap-1">
        {items.data?.map((item) => (
          <li key={item.id} className="flex items-center gap-2 text-sm">
            <Checkbox
              aria-label={item.text}
              checked={item.done}
              onCheckedChange={(done) => update.mutate({ id: item.id, text: item.text, done })}
            />
            <span className={cn("flex-1", item.done && "text-muted-foreground line-through")}>
              {item.text}
            </span>
            <Button
              variant="ghost"
              size="icon"
              aria-label={`Remove ${item.text}`}
              onClick={() => remove.mutate(item.id)}
            >
              <X size={14} />
            </Button>
          </li>
        ))}
      </ul>
      <AddItemForm placeholder="Add a checklist item" pending={add.isPending} onAdd={add.mutate} />
    </section>
  );
}
