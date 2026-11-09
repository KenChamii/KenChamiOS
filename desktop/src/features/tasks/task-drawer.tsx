import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetTitle } from "@/components/ui/sheet";
import { CopyTaskPromptButton } from "@/features/workflow/copy-task-prompt-button";
import { useDeleteTask, useUpdateTask } from "@/hooks/use-tasks";
import type { Task } from "@/lib/api/tasks";
import { errorMessage } from "@/lib/errors";
import { Checklist } from "./checklist";
import { Subtasks } from "./subtasks";
import { TaskForm } from "./task-form";

type TaskDrawerProps = {
  task: Task | null;
  subtasks: Task[];
  onClose: () => void;
};

export function TaskDrawer({ task, subtasks, onClose }: TaskDrawerProps) {
  return (
    <Sheet open={task !== null} onOpenChange={(open) => !open && onClose()}>
      <SheetContent aria-describedby={undefined}>
        <SheetTitle className="mb-4 text-lg font-semibold">Task</SheetTitle>
        {task && <TaskDetail key={task.id} task={task} subtasks={subtasks} onClose={onClose} />}
      </SheetContent>
    </Sheet>
  );
}

type TaskDetailProps = { task: Task; subtasks: Task[]; onClose: () => void };

function TaskDetail({ task, subtasks, onClose }: TaskDetailProps) {
  const update = useUpdateTask(task.project_id);
  const remove = useDeleteTask(task.project_id);
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  return (
    <div className="flex flex-col gap-6">
      <div>
        <CopyTaskPromptButton taskId={task.id} showLabel />
      </div>
      <TaskForm
        task={task}
        pending={update.isPending}
        error={update.error ? errorMessage(update.error) : null}
        onSave={(input) => update.mutate({ id: task.id, input })}
      />
      <Subtasks parent={task} subtasks={subtasks} />
      <Checklist taskId={task.id} />
      <div className="flex gap-2">
        {confirmingDelete ? (
          <>
            <Button
              variant="destructive"
              disabled={remove.isPending}
              onClick={() => remove.mutate(task.id, { onSuccess: onClose })}
            >
              Confirm delete
            </Button>
            <Button variant="ghost" onClick={() => setConfirmingDelete(false)}>
              Cancel
            </Button>
          </>
        ) : (
          <Button variant="ghost" onClick={() => setConfirmingDelete(true)}>
            Delete task
          </Button>
        )}
      </div>
    </div>
  );
}
