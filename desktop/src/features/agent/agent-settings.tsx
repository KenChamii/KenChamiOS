import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { useAgentSettings, useSaveAgentSettings } from "@/hooks/use-agent-runs";
import { AGENT_TOOLS, type AgentSettings, type PermissionMode } from "@/lib/api/agent";
import { errorMessage } from "@/lib/errors";

const MODES: { value: PermissionMode; label: string }[] = [
  { value: "dontAsk", label: "Only what is allowed below" },
  { value: "acceptEdits", label: "Accept file edits" },
];

export function AgentSettingsCard({ projectId }: { projectId: string }) {
  const settings = useAgentSettings(projectId);
  if (!settings.data) return null;
  return <SettingsForm projectId={projectId} saved={settings.data} />;
}

function SettingsForm({ projectId, saved }: { projectId: string; saved: AgentSettings }) {
  const save = useSaveAgentSettings(projectId);
  const [draft, setDraft] = useState(saved);
  const change = (patch: Partial<AgentSettings>) => setDraft({ ...draft, ...patch });
  const toggleTool = (tool: string, on: boolean) =>
    change({ tools: on ? [...draft.tools, tool] : draft.tools.filter((t) => t !== tool) });

  return (
    <Card className="flex flex-col gap-4">
      <h2 className="font-semibold tracking-tight">Agent settings</h2>
      <Field label="Test command">
        <Input
          value={draft.test_command}
          placeholder="npm test"
          onChange={(event) => change({ test_command: event.target.value })}
        />
      </Field>
      <p className="-mt-2 text-xs text-muted-foreground">
        Runs after every run, and is the only command the agent may use. Leave it empty to give the
        agent no shell at all.
      </p>
      <Field label="Rules">
        <Textarea
          value={draft.rules}
          placeholder="Empty uses the default rules"
          onChange={(event) => change({ rules: event.target.value })}
        />
      </Field>
      <fieldset className="flex flex-wrap gap-4">
        <legend className="mb-1 text-label text-muted-foreground">Tools</legend>
        {AGENT_TOOLS.map((tool) => (
          <label key={tool} className="flex items-center gap-2 text-sm">
            <Checkbox
              aria-label={tool}
              checked={draft.tools.includes(tool)}
              onCheckedChange={(on) => toggleTool(tool, on)}
            />
            {tool}
          </label>
        ))}
      </fieldset>
      <div className="grid gap-3 sm:grid-cols-3">
        <Field label="Permissions">
          <Select
            aria-label="Permissions"
            value={draft.permission_mode}
            options={MODES}
            onChange={(permission_mode) => change({ permission_mode })}
          />
        </Field>
        <Field label="Timeout (minutes)">
          <Input
            type="number"
            min={1}
            max={240}
            value={draft.timeout_min}
            onChange={(event) => change({ timeout_min: Number(event.target.value) })}
          />
        </Field>
        <Field label="Turn limit">
          <Input
            type="number"
            min={1}
            max={200}
            value={draft.max_turns}
            onChange={(event) => change({ max_turns: Number(event.target.value) })}
          />
        </Field>
      </div>
      {save.error && <p className="text-sm text-destructive">{errorMessage(save.error)}</p>}
      <div>
        <Button disabled={save.isPending} onClick={() => save.mutate(draft)}>
          {save.isSuccess && !save.isPending ? "Saved" : "Save settings"}
        </Button>
      </div>
    </Card>
  );
}
