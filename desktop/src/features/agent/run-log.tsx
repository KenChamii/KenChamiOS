import { useEffect, useRef } from "react";
import type { StreamEvent } from "@/lib/api/agent";
import { cn } from "@/lib/cn";

function line(event: StreamEvent): { text: string; tone?: "error" | "muted" } | null {
  switch (event.kind) {
    case "init":
      return { text: `Started with ${event.model || "Claude"}`, tone: "muted" };
    case "text":
      return { text: event.text };
    case "tool_use":
      return { text: `${event.name} ${event.input}`, tone: "muted" };
    case "tool_result":
      return event.is_error ? { text: event.content, tone: "error" } : null;
    case "result":
      return { text: event.text, tone: event.is_error ? "error" : undefined };
    case "raw":
      return { text: event.text, tone: "muted" };
  }
}

export function RunLog({ events }: { events: StreamEvent[] }) {
  const end = useRef<HTMLDivElement>(null);
  useEffect(() => end.current?.scrollIntoView({ block: "end" }), [events.length]);
  const lines = events.map(line).filter((item) => item !== null);
  if (lines.length === 0) {
    return <p className="text-sm text-muted-foreground">Nothing yet.</p>;
  }
  return (
    <div className="flex max-h-72 flex-col gap-1.5 overflow-auto rounded-xl bg-muted p-3 text-sm">
      {lines.map((item, index) => (
        <p
          key={index}
          className={cn(
            "whitespace-pre-wrap break-words",
            item.tone === "muted" && "font-mono text-xs text-muted-foreground",
            item.tone === "error" && "text-destructive",
          )}
        >
          {item.text}
        </p>
      ))}
      <div ref={end} />
    </div>
  );
}
