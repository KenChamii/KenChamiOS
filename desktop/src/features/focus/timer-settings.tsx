import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { useSetTimerSettings, useTimerSettings } from "@/hooks/use-timer";
import type { TimerSettings } from "@/lib/api/timer";
import { errorMessage } from "@/lib/errors";

const FIELDS: { key: keyof TimerSettings; label: string }[] = [
  { key: "focus_min", label: "Focus (minutes)" },
  { key: "break_min", label: "Break (minutes)" },
  { key: "long_break_min", label: "Long break (minutes)" },
];

export function TimerSettingsCard() {
  const settings = useTimerSettings();
  if (settings.error) {
    return <p className="text-sm text-destructive">{errorMessage(settings.error)}</p>;
  }
  if (!settings.data) return null;
  return <TimerSettingsForm saved={settings.data} />;
}

function TimerSettingsForm({ saved }: { saved: TimerSettings }) {
  const save = useSetTimerSettings();
  const [draft, setDraft] = useState<Record<keyof TimerSettings, string>>({
    focus_min: String(saved.focus_min),
    break_min: String(saved.break_min),
    long_break_min: String(saved.long_break_min),
  });

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-semibold tracking-tight">Focus timer</h2>
      <p className="text-sm text-muted-foreground">
        A long break replaces every fourth break. Changes apply from the next phase.
      </p>
      <form
        className="flex flex-wrap items-end gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          save.mutate({
            focus_min: Number(draft.focus_min),
            break_min: Number(draft.break_min),
            long_break_min: Number(draft.long_break_min),
          });
        }}
      >
        {FIELDS.map(({ key, label }) => (
          <Field key={key} label={label}>
            <Input
              type="number"
              min={1}
              max={180}
              required
              className="w-40"
              value={draft[key]}
              onChange={(event) => setDraft({ ...draft, [key]: event.target.value })}
            />
          </Field>
        ))}
        <Button type="submit" disabled={save.isPending}>
          Save
        </Button>
        {save.isSuccess && <span className="text-sm text-muted-foreground">Saved</span>}
      </form>
      {save.error && <p className="text-sm text-destructive">{errorMessage(save.error)}</p>}
    </Card>
  );
}
