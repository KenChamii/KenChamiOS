import { Briefcase, Plus } from "lucide-react";
import { useState } from "react";
import { PageHeader } from "@/components/layout/page-header";
import { Button } from "@/components/ui/button";
import { EmptyState } from "@/components/ui/empty-state";
import { JobBoard } from "@/features/jobs/job-board";
import { JobSheet } from "@/features/jobs/job-sheet";
import { JobStats } from "@/features/jobs/job-stats";
import { useJobs } from "@/hooks/use-jobs";
import { errorMessage } from "@/lib/errors";

// "new" opens the empty form; an id opens that application
const NEW = "new";

export function JobsPage() {
  const jobs = useJobs();
  const [openId, setOpenId] = useState<string | null>(null);
  const all = jobs.data ?? [];
  const opened = all.find((job) => job.id === openId) ?? null;

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Jobs"
        subtitle="Every application, and what to do next"
        icon={Briefcase}
        area="projects"
      >
        <Button onClick={() => setOpenId(NEW)}>
          <Plus size={16} />
          Add application
        </Button>
      </PageHeader>
      {jobs.error && <p className="text-sm text-destructive">{errorMessage(jobs.error)}</p>}
      {jobs.data && all.length === 0 ? (
        <EmptyState
          icon={Briefcase}
          area="projects"
          line="No applications yet. Save the first posting you find."
          action={<Button onClick={() => setOpenId(NEW)}>Add application</Button>}
        />
      ) : (
        <>
          <JobBoard jobs={all} onOpen={setOpenId} />
          <JobStats />
        </>
      )}
      {openId === NEW && <JobSheet job={null} onClose={() => setOpenId(null)} />}
      {opened && <JobSheet key={opened.id} job={opened} onClose={() => setOpenId(null)} />}
    </div>
  );
}
