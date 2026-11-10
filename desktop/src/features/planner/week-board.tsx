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
import { X } from "lucide-react";
import { useRef, useState, type PointerEvent } from "react";
import { useCandidates, useFocusBlocks, usePlanned, useSetPlan } from "@/hooks/use-planner";
import type { FocusBlock, PlannedTask, TaskPlan } from "@/lib/api/planner";
import { cn } from "@/lib/cn";
import { fromDateInput } from "@/lib/dates";
import type { Day } from "@/lib/days";
import { errorMessage } from "@/lib/errors";
import { DEFAULT_ESTIMATE_MIN, formatMinutes } from "./capacity";
import {
  FIRST_HOUR,
  HOUR_PX,
  LAST_HOUR,
  PX_PER_MIN,
  SLOT_COUNT,
  SLOT_MIN,
  UNSCHEDULED_ID,
  blockLayout,
  dayBounds,
  dayId,
  parseDrop,
  resizedDuration,
  slotId,
  slotStart,
} from "./week-grid";

const GRID_HEIGHT = (LAST_HOUR - FIRST_HOUR) * HOUR_PX;
const SLOT_HEIGHT = SLOT_MIN * PX_PER_MIN;
const HOURS = Array.from({ length: LAST_HOUR - FIRST_HOUR }, (_, index) => FIRST_HOUR + index);

type Layout = { top: number; height: number };

function taskLayout(task: PlannedTask): Layout | null {
  if (task.planned_on === null || task.scheduled_at === null) return null;
  const minutes = task.duration_min ?? DEFAULT_ESTIMATE_MIN;
  return blockLayout(task.planned_on, task.scheduled_at, task.scheduled_at + minutes * 60_000);
}

function planOf(task: PlannedTask, changes: Partial<TaskPlan>): TaskPlan {
  return {
    planned_on: task.planned_on,
    scheduled_at: task.scheduled_at,
    duration_min: task.duration_min,
    ...changes,
  };
}

type WeekBoardProps = { week: Day[]; today: Day };

/** Drag tasks onto days and hours; recorded focus sessions show beside them. */
export function WeekBoard({ week, today }: WeekBoardProps) {
  const from = dayBounds(week[0]).from;
  const to = dayBounds(week[6]).to;
  const planned = usePlanned(week[0], week[6]);
  const candidates = useCandidates(today, to, "");
  const focus = useFocusBlocks(from, to);
  const setPlan = useSetPlan();
  const [dragging, setDragging] = useState<PlannedTask | null>(null);
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 5 } }));

  const plannedTasks = planned.data ?? [];
  const plannedIds = new Set(plannedTasks.map((task) => task.id));
  const loose = [
    ...plannedTasks.filter((task) => taskLayout(task) === null),
    ...(candidates.data ?? []).filter((task) => !plannedIds.has(task.id)),
  ];
  const byId = new Map([...plannedTasks, ...loose].map((task) => [task.id, task]));
  const error = planned.error ?? candidates.error ?? focus.error ?? setPlan.error;

  function save(task: PlannedTask, changes: Partial<TaskPlan>) {
    setPlan.mutate({ id: task.id, plan: planOf(task, changes) });
  }

  function handleDragStart(event: DragStartEvent) {
    setDragging(byId.get(String(event.active.id)) ?? null);
  }

  function handleDragEnd(event: DragEndEvent) {
    setDragging(null);
    const task = byId.get(String(event.active.id));
    const drop = event.over ? parseDrop(String(event.over.id)) : null;
    if (!task || !drop) return;
    const duration_min = task.duration_min ?? DEFAULT_ESTIMATE_MIN;
    if (drop.kind === "slot") {
      save(task, {
        planned_on: drop.day,
        scheduled_at: slotStart(drop.day, drop.slot),
        duration_min,
      });
    } else if (drop.kind === "day") {
      save(task, { planned_on: drop.day, scheduled_at: null, duration_min });
    } else if (task.scheduled_at !== null) {
      save(task, { scheduled_at: null });
    }
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={pointerWithin}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={() => setDragging(null)}
    >
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <div className="grid grid-cols-[14rem_1fr] items-start gap-4">
        <LooseList tasks={loose} today={today} />
        <div className="grid grid-cols-[2.5rem_repeat(7,minmax(0,1fr))] gap-x-1 rounded-2xl bg-card p-3 shadow-soft">
          <span />
          {week.map((day) => (
            <DayHeader key={day} day={day} today={today} />
          ))}
          <div className="relative" style={{ height: GRID_HEIGHT }}>
            {HOURS.map((hour, index) => (
              <span
                key={hour}
                className="absolute right-1 -translate-y-1/2 text-xs tabular-nums text-muted-foreground"
                style={{ top: index * HOUR_PX }}
              >
                {index > 0 && `${hour}:00`}
              </span>
            ))}
          </div>
          {week.map((day) => (
            <DayColumn
              key={day}
              day={day}
              today={today}
              tasks={plannedTasks.filter((task) => task.planned_on === day)}
              sessions={focus.data ?? []}
              onResize={(task, duration_min) => save(task, { duration_min })}
              onUnschedule={(task) => save(task, { scheduled_at: null })}
            />
          ))}
        </div>
      </div>
      <DragOverlay>{dragging && <TaskChip task={dragging} />}</DragOverlay>
    </DndContext>
  );
}

function LooseList({ tasks, today }: { tasks: PlannedTask[]; today: Day }) {
  const { setNodeRef, isOver } = useDroppable({ id: UNSCHEDULED_ID });
  return (
    <section
      ref={setNodeRef}
      className={cn(
        "flex min-h-40 flex-col gap-2 rounded-2xl bg-muted p-2",
        isOver && "ring-2 ring-primary",
      )}
    >
      <h3 className="px-1 text-sm font-medium">Not on the grid</h3>
      <p className="px-1 text-xs text-muted-foreground">
        Planned this week without a time, due, in progress or in the Inbox. Drag onto a day or an
        hour.
      </p>
      {tasks.map((task) => (
        <DraggableTask key={task.id} task={task} today={today} />
      ))}
    </section>
  );
}

function DraggableTask({ task, today }: { task: PlannedTask; today: Day }) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({ id: task.id });
  return (
    <div
      ref={setNodeRef}
      className={cn("cursor-grab", isDragging && "opacity-40")}
      {...attributes}
      {...listeners}
    >
      <TaskChip task={task} today={today} />
    </div>
  );
}

function dayLabel(day: Day, options: Intl.DateTimeFormatOptions): string {
  return new Date(fromDateInput(day) ?? 0).toLocaleDateString(undefined, options);
}

function TaskChip({ task, today }: { task: PlannedTask; today?: Day }) {
  const when =
    task.planned_on === null
      ? null
      : task.planned_on === today
        ? "Today"
        : dayLabel(task.planned_on, { weekday: "short" });
  return (
    <div className="flex flex-col gap-0.5 rounded-xl bg-card p-2 text-sm shadow-soft">
      <span className="truncate">{task.title}</span>
      <span className="truncate text-xs text-muted-foreground">
        {[task.project_name, when, task.duration_min && formatMinutes(task.duration_min)]
          .filter(Boolean)
          .join(" · ")}
      </span>
    </div>
  );
}

function DayHeader({ day, today }: { day: Day; today: Day }) {
  const { setNodeRef, isOver } = useDroppable({ id: dayId(day) });
  return (
    <div
      ref={setNodeRef}
      className={cn(
        "mb-2 flex flex-col items-center rounded-xl py-1 text-sm",
        day === today && "text-focus",
        isOver && "bg-muted ring-2 ring-primary",
      )}
    >
      <span className="text-xs uppercase tracking-wider text-muted-foreground">
        {dayLabel(day, { weekday: "short" })}
      </span>
      <span className="font-semibold tabular-nums">{Number(day.slice(8))}</span>
    </div>
  );
}

type DayColumnProps = {
  day: Day;
  today: Day;
  tasks: PlannedTask[];
  sessions: FocusBlock[];
  onResize: (task: PlannedTask, minutes: number) => void;
  onUnschedule: (task: PlannedTask) => void;
};

function DayColumn({ day, today, tasks, sessions, onResize, onUnschedule }: DayColumnProps) {
  return (
    <div
      className={cn("relative rounded-lg", day === today && "bg-focus-soft")}
      style={{ height: GRID_HEIGHT }}
    >
      {Array.from({ length: SLOT_COUNT }, (_, slot) => (
        <Slot key={slot} day={day} slot={slot} />
      ))}
      {sessions.map((session) => {
        const layout = blockLayout(day, session.started_at, session.ended_at);
        return layout && <SessionBlock key={session.id} session={session} layout={layout} />;
      })}
      {tasks.map((task) => {
        const layout = taskLayout(task);
        return (
          layout && (
            <TaskBlock
              key={task.id}
              task={task}
              layout={layout}
              onResize={(minutes) => onResize(task, minutes)}
              onUnschedule={() => onUnschedule(task)}
            />
          )
        );
      })}
    </div>
  );
}

function Slot({ day, slot }: { day: Day; slot: number }) {
  const { setNodeRef, isOver } = useDroppable({ id: slotId(day, slot) });
  return (
    <div
      ref={setNodeRef}
      className={cn(
        "absolute inset-x-0",
        slot % 2 === 0 && "border-t border-border",
        isOver && "bg-primary/15",
      )}
      style={{ top: slot * SLOT_HEIGHT, height: SLOT_HEIGHT }}
    />
  );
}

function SessionBlock({ session, layout }: { session: FocusBlock; layout: Layout }) {
  const title = session.task_title ?? session.project_name;
  return (
    <div
      title={`Focus: ${title}`}
      aria-label={`Focus session: ${title}`}
      className="absolute right-0 w-[26%] overflow-hidden rounded-md bg-focus/30"
      style={{
        top: layout.top,
        height: layout.height,
        backgroundColor: session.project_color ?? undefined,
        opacity: session.project_color ? 0.4 : undefined,
      }}
    />
  );
}

type TaskBlockProps = {
  task: PlannedTask;
  layout: Layout;
  onResize: (minutes: number) => void;
  onUnschedule: () => void;
};

function TaskBlock({ task, layout, onResize, onUnschedule }: TaskBlockProps) {
  const { attributes, listeners, setNodeRef, isDragging } = useDraggable({ id: task.id });
  const [preview, setPreview] = useState<number | null>(null);
  const resize = useRef<{ y: number; minutes: number } | null>(null);
  const minutes = task.duration_min ?? DEFAULT_ESTIMATE_MIN;

  function startResize(event: PointerEvent<HTMLDivElement>) {
    // the handle resizes, so the block must not start a drag
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    resize.current = { y: event.clientY, minutes };
    setPreview(minutes);
  }

  function moveResize(event: PointerEvent<HTMLDivElement>) {
    if (!resize.current) return;
    setPreview(resizedDuration(resize.current.minutes, event.clientY - resize.current.y));
  }

  function endResize(event: PointerEvent<HTMLDivElement>) {
    if (!resize.current) return;
    const next = resizedDuration(resize.current.minutes, event.clientY - resize.current.y);
    resize.current = null;
    setPreview(null);
    if (next !== minutes) onResize(next);
  }

  const height = preview === null ? layout.height : preview * PX_PER_MIN;
  return (
    <div
      ref={setNodeRef}
      className={cn(
        "absolute left-0 right-[28%] flex cursor-grab flex-col overflow-hidden rounded-md bg-primary px-1.5 py-1 text-xs text-primary-foreground shadow-soft",
        task.status === "done" && "opacity-60 line-through",
        isDragging && "opacity-40",
      )}
      style={{ top: layout.top, height: Math.max(height, SLOT_HEIGHT / 2) }}
      {...attributes}
      {...listeners}
    >
      <div className="flex items-start justify-between gap-1">
        <span className="truncate font-medium">{task.title}</span>
        <button
          type="button"
          aria-label={`Take ${task.title} off the grid`}
          className="shrink-0 cursor-pointer rounded opacity-70 hover:opacity-100"
          onPointerDown={(event) => event.stopPropagation()}
          onClick={onUnschedule}
        >
          <X size={12} />
        </button>
      </div>
      <span className="truncate opacity-80">
        {formatMinutes(preview ?? minutes)} · {task.project_name}
      </span>
      <div
        role="separator"
        aria-label={`Resize ${task.title}`}
        className="absolute inset-x-0 bottom-0 h-2 cursor-ns-resize"
        onPointerDown={startResize}
        onPointerMove={moveResize}
        onPointerUp={endResize}
        onPointerCancel={() => {
          resize.current = null;
          setPreview(null);
        }}
      />
    </div>
  );
}
