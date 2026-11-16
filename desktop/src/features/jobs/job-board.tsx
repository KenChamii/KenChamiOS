import {
  DndContext,
  DragOverlay,
  PointerSensor,
  pointerWithin,
  useDraggable,
  useDroppable,
  useSensor,
  useSensors,
  type DragEndEvent,
  type DragStartEvent,
} from "@dnd-kit/core";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { useMoveJob } from "@/hooks/use-jobs";
import type { JobView } from "@/lib/api/jobs";
import { cn } from "@/lib/cn";
import { errorMessage } from "@/lib/errors";
import { COLUMNS, STAGE_LABELS, groupByColumn, stageForDrop, type Column } from "./stages";

type Props = { jobs: JobView[]; onOpen: (id: string) => void };

export function JobBoard({ jobs, onOpen }: Props) {
  const move = useMoveJob();
  const [dragging, setDragging] = useState<JobView | null>(null);
  const columns = groupByColumn(jobs);
  // a small drag distance keeps plain clicks free to open the application
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));

  function handleStart(event: DragStartEvent) {
    setDragging(jobs.find((job) => job.id === event.active.id) ?? null);
  }

  function handleEnd(event: DragEndEvent) {
    setDragging(null);
    const job = jobs.find((candidate) => candidate.id === event.active.id);
    if (!job || !event.over) return;
    const stage = stageForDrop(event.over.id as Column, job.stage);
    if (stage !== job.stage) move.mutate({ id: job.id, stage });
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={pointerWithin}
      onDragStart={handleStart}
      onDragEnd={handleEnd}
      onDragCancel={() => setDragging(null)}
    >
      {move.error && <p className="text-sm text-destructive">{errorMessage(move.error)}</p>}
      <div className="grid grid-cols-6 gap-3">
        {COLUMNS.map(({ id, label }) => (
          <Lane key={id} id={id} label={label} jobs={columns[id]} onOpen={onOpen} />
        ))}
      </div>
      <DragOverlay>{dragging && <JobCard job={dragging} />}</DragOverlay>
    </DndContext>
  );
}

type LaneProps = { id: Column; label: string; jobs: JobView[]; onOpen: (id: string) => void };

function Lane({ id, label, jobs, onOpen }: LaneProps) {
  const { setNodeRef, isOver } = useDroppable({ id });
  return (
    <section
      ref={setNodeRef}
      className={cn(
        "flex min-h-48 flex-col gap-2 rounded-2xl bg-muted p-2",
        isOver && "ring-2 ring-primary",
      )}
    >
      <h3 className="px-1 text-sm font-medium">
        {label} <span className="text-muted-foreground">{jobs.length}</span>
      </h3>
      {jobs.map((job) => (
        <DraggableCard key={job.id} job={job} onOpen={onOpen} />
      ))}
    </section>
  );
}

function DraggableCard({ job, onOpen }: { job: JobView; onOpen: (id: string) => void }) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({ id: job.id });
  return (
    <div
      ref={setNodeRef}
      className={cn(isDragging && "opacity-40")}
      onClick={() => onOpen(job.id)}
      {...attributes}
      {...listeners}
    >
      <JobCard job={job} />
    </div>
  );
}

function JobCard({ job }: { job: JobView }) {
  return (
    <Card className="flex cursor-grab flex-col gap-1.5 p-3">
      <span className="text-sm font-medium">{job.role}</span>
      <span className="text-xs text-muted-foreground">{job.company}</span>
      <div className="flex flex-wrap gap-1">
        {job.due && <Badge className="text-destructive">Follow up</Badge>}
        {job.stale && <Badge>No news for a week</Badge>}
        {job.stage !== "saved" && job.stage !== "applied" && job.stage !== "screening" && (
          <Badge>{STAGE_LABELS[job.stage]}</Badge>
        )}
      </div>
    </Card>
  );
}
