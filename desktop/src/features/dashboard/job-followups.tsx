import { Link } from "react-router-dom";
import { Card } from "@/components/ui/card";
import { useJobsNeedingAttention } from "@/hooks/use-jobs";

/** Applications whose next step is due or that nobody touched for a week. Hidden when none. */
export function JobFollowups() {
  const jobs = useJobsNeedingAttention();
  const list = jobs.data ?? [];
  if (list.length === 0) return null;

  return (
    <Card className="flex flex-col gap-2">
      <h2 className="flex items-center justify-between font-semibold tracking-tight">
        Job follow-ups
        <Link
          to="/jobs"
          className="text-sm font-normal text-muted-foreground hover:text-foreground"
        >
          Open board
        </Link>
      </h2>
      <ul className="flex flex-col gap-1 text-sm">
        {list.map((job) => (
          <li key={job.id} className="flex justify-between gap-3">
            <span className="truncate">
              {job.role} at {job.company}
            </span>
            <span className="shrink-0 text-muted-foreground">
              {job.due ? job.next_step || "Follow up" : "No news for a week"}
            </span>
          </li>
        ))}
      </ul>
    </Card>
  );
}
