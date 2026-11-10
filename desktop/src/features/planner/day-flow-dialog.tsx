import { useState } from "react";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { endOfDay } from "@/features/dashboard/ranges";
import { toDateInput } from "@/lib/dates";
import { useOverlays, type DayFlow } from "@/stores/overlays";
import { PlanDay } from "./plan-day";
import { ReviewDay } from "./review-day";

const TITLES: Record<DayFlow, string> = { plan: "Plan my day", review: "Review my day" };

export function DayFlowDialog() {
  const { dayFlow, closeDayFlow } = useOverlays();
  return (
    <Dialog open={dayFlow !== null} onOpenChange={(open) => !open && closeDayFlow()}>
      <DialogContent aria-describedby={undefined} className="max-h-[85vh] max-w-2xl overflow-auto">
        {dayFlow && <DayFlowBody flow={dayFlow} onClose={closeDayFlow} />}
      </DialogContent>
    </Dialog>
  );
}

function DayFlowBody({ flow, onClose }: { flow: DayFlow; onClose: () => void }) {
  // the day the dialog opened on, so a plan made around midnight stays on one day
  const [now] = useState(() => Date.now());
  const today = toDateInput(now);
  return (
    <>
      <div className="mb-4 flex items-baseline justify-between gap-2">
        <DialogTitle className="text-lg font-semibold">{TITLES[flow]}</DialogTitle>
        <span className="text-sm text-muted-foreground">
          {new Date(now).toLocaleDateString(undefined, {
            weekday: "long",
            day: "numeric",
            month: "long",
          })}
        </span>
      </div>
      {flow === "plan" ? (
        <PlanDay today={today} endOfToday={endOfDay(now)} />
      ) : (
        <ReviewDay today={today} onClose={onClose} />
      )}
    </>
  );
}
