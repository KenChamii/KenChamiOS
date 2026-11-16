import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import {
  useBriefs,
  useMcpStatus,
  useNewMcpToken,
  useRunBrief,
  useSetBrief,
  useSetMcpEnabled,
  useUndoMcpCall,
} from "@/hooks/use-mcp";
import type { BriefKind } from "@/lib/api/mcp";
import { timeAgo } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";
import { claudeCommand, toolLabel } from "./snippet";

const BRIEF_LABELS: Record<BriefKind, { name: string; when: string }> = {
  morning: { name: "Morning brief", when: "every day, after 05:00 UTC" },
  review: { name: "Weekly review", when: "Sundays (UTC)" },
  jobs: { name: "Job search summary", when: "Mondays (UTC)" },
};

export function AgentsCard() {
  return (
    <>
      <ServerCard />
      <BriefsCard />
    </>
  );
}

function ServerCard() {
  const status = useMcpStatus();
  const toggle = useSetMcpEnabled();
  const newToken = useNewMcpToken();
  const undo = useUndoMcpCall();
  const [copied, setCopied] = useState(false);
  const data = status.data;
  const error = status.error ?? toggle.error ?? newToken.error ?? undo.error;

  async function copy(text: string) {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <Card className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <h2 className="font-semibold tracking-tight">Let Claude Code use the app</h2>
        <p className="text-sm text-muted-foreground">
          A small server on this computer only. Claude Code can read today, add tasks and inbox
          items, log time and move stages. Every call is listed below and most can be undone.
        </p>
      </div>
      <label className="flex items-center justify-between gap-3 text-sm">
        <span>
          Allow it{" "}
          {data?.running && <span className="text-muted-foreground">(port {data.port})</span>}
        </span>
        <Switch
          aria-label="Allow Claude Code"
          checked={data?.enabled ?? false}
          disabled={!data || toggle.isPending}
          onCheckedChange={(enabled) => toggle.mutate(enabled)}
        />
      </label>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      {data?.enabled && data.token && (
        <div className="flex flex-col gap-2">
          <p className="text-sm text-muted-foreground">
            Run this once in a terminal to connect Claude Code. The token acts like a password.
          </p>
          <pre className="overflow-auto whitespace-pre-wrap break-all rounded-xl bg-muted p-3 font-mono text-xs">
            {claudeCommand(data.port, data.token)}
          </pre>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => void copy(claudeCommand(data.port, data.token ?? ""))}
            >
              {copied ? "Copied" : "Copy command"}
            </Button>
            <Button
              variant="ghost"
              size="sm"
              disabled={newToken.isPending}
              onClick={() => newToken.mutate(undefined)}
            >
              New token
            </Button>
          </div>
        </div>
      )}
      {data && data.calls.length > 0 && (
        <div className="flex flex-col gap-1">
          <h3 className="text-label text-muted-foreground">Recent calls</h3>
          <ul className="flex flex-col divide-y divide-border text-sm">
            {data.calls.map((call) => (
              <li key={call.id} className="flex items-center justify-between gap-3 py-1.5">
                <span className="flex min-w-0 flex-col">
                  <span className={call.ok ? "" : "text-destructive"}>
                    {toolLabel(call.tool)}
                    {call.undone && <span className="text-muted-foreground"> (undone)</span>}
                  </span>
                  <span className="truncate text-xs text-muted-foreground">{call.summary}</span>
                </span>
                <span className="flex shrink-0 items-center gap-2">
                  <span className="text-xs text-muted-foreground">{timeAgo(call.at)}</span>
                  {call.can_undo && (
                    <Button
                      variant="ghost"
                      size="sm"
                      disabled={undo.isPending}
                      onClick={() => undo.mutate(call.id)}
                    >
                      Undo
                    </Button>
                  )}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </Card>
  );
}

function BriefsCard() {
  const briefs = useBriefs();
  const set = useSetBrief();
  const run = useRunBrief();
  const [done, setDone] = useState<string | null>(null);

  return (
    <Card className="flex flex-col gap-4">
      <div className="flex flex-col gap-1">
        <h2 className="font-semibold tracking-tight">Briefs in the vault</h2>
        <p className="text-sm text-muted-foreground">
          Claude Code writes a note into <code>KenChamiOS/Briefs</code> of your Obsidian vault while
          the app is open. It uses your Claude login.
        </p>
      </div>
      {(briefs.data ?? []).map(({ kind, enabled }) => (
        <div key={kind} className="flex items-center justify-between gap-3 text-sm">
          <span className="flex flex-col">
            {BRIEF_LABELS[kind].name}
            <span className="text-xs text-muted-foreground">{BRIEF_LABELS[kind].when}</span>
          </span>
          <span className="flex items-center gap-3">
            <Button
              variant="outline"
              size="sm"
              disabled={run.isPending}
              onClick={() => run.mutate(kind, { onSuccess: setDone })}
            >
              Write now
            </Button>
            <Switch
              aria-label={BRIEF_LABELS[kind].name}
              checked={enabled}
              onCheckedChange={(on) => set.mutate({ kind, enabled: on })}
            />
          </span>
        </div>
      ))}
      {run.isPending && <p className="text-sm text-muted-foreground">This can take a minute.</p>}
      {done && !run.isPending && <p className="text-sm text-muted-foreground">Wrote {done}</p>}
      {(run.error ?? set.error) && (
        <p className="text-sm text-destructive">{errorMessage(run.error ?? set.error)}</p>
      )}
    </Card>
  );
}
