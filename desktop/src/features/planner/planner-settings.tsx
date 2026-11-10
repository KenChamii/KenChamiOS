import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Field } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { useAvailableMinutes, useSetAvailableMinutes } from "@/hooks/use-planner";
import { errorMessage } from "@/lib/errors";
import { parseHours } from "./capacity";

export function PlannerSettingsCard() {
  const minutes = useAvailableMinutes();
  if (minutes.error) {
    return <p className="text-sm text-destructive">{errorMessage(minutes.error)}</p>;
  }
  if (minutes.data === undefined) return null;
  return <PlannerSettingsForm saved={minutes.data} />;
}

function PlannerSettingsForm({ saved }: { saved: number }) {
  const save = useSetAvailableMinutes();
  const [draft, setDraft] = useState(String(saved / 60));
  const parsed = parseHours(draft);

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-semibold tracking-tight">Planner</h2>
      <p className="text-sm text-muted-foreground">
        Plan my day compares the estimates of today's tasks with the hours you have for focused
        work, and warns when the plan does not fit.
      </p>
      <form
        className="flex flex-wrap items-end gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (parsed !== null) save.mutate(parsed);
        }}
      >
        <Field label="Hours you have each day">
          <Input
            type="number"
            min={0.5}
            max={24}
            step={0.5}
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
