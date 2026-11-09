import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { useIdleDays, useSetIdleDays } from "@/hooks/use-reminders";
import { errorMessage } from "@/lib/errors";
import { MAX_IDLE_DAYS, MIN_IDLE_DAYS, parseIdleDays } from "./idle-days";

export function ReminderSettingsCard() {
  const days = useIdleDays();
  if (days.error) return <p className="text-sm text-destructive">{errorMessage(days.error)}</p>;
  if (days.data === undefined) return null;
  return <ReminderSettingsForm saved={days.data} />;
}

function ReminderSettingsForm({ saved }: { saved: number }) {
  const save = useSetIdleDays();
  const [draft, setDraft] = useState(String(saved));
  const parsed = parseIdleDays(draft);

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-medium">Reminders</h2>
      <p className="text-sm text-muted-foreground">
        The dashboard suggests a save point for a project in Plan, Build or Review after this many
        days without work and without a save point. Snoozing hides a project for the same time.
      </p>
      <form
        className="flex flex-wrap items-end gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (parsed !== null) save.mutate(parsed);
        }}
      >
        <Field label="Days of quiet">
          <Input
            type="number"
            min={MIN_IDLE_DAYS}
            max={MAX_IDLE_DAYS}
            required
            className="w-40"
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
          />
        </Field>
        <Button type="submit" disabled={save.isPending || parsed === null}>
          Save
        </Button>
        {save.isSuccess && <span className="text-sm text-muted-foreground">Saved</span>}
      </form>
      {save.error && <p className="text-sm text-destructive">{errorMessage(save.error)}</p>}
    </Card>
  );
}
