import { useQuery } from "@tanstack/react-query";
import {
  Bell,
  DatabaseBackup,
  FileText,
  Film,
  Info,
  Settings,
  SlidersHorizontal,
  Smartphone,
  type LucideIcon,
} from "lucide-react";
import type { ReactNode } from "react";
import { NavLink, useParams } from "react-router-dom";
import { PageHeader } from "@/components/layout/page-header";
import { ThemeToggle } from "@/components/layout/theme-toggle";
import { Card } from "@/components/ui/card";
import { BackupCard } from "@/features/backups/backup-card";
import { TimerSettingsCard } from "@/features/focus/timer-settings";
import { GamesKeyCard } from "@/features/media/games-key-card";
import { SourceCredits } from "@/features/media/source-credits";
import { TmdbTokenCard } from "@/features/media/tmdb-token-card";
import { ReminderSettingsCard } from "@/features/reminders/reminder-settings";
import { SyncCard } from "@/features/sync/sync-card";
import { TemplateSettings } from "@/features/workflow/template-settings";
import { getAppInfo } from "@/lib/api/app";
import { cn } from "@/lib/cn";

type Section = { id: string; label: string; icon: LucideIcon; content: () => ReactNode };

const SECTIONS: Section[] = [
  { id: "general", label: "General", icon: SlidersHorizontal, content: () => <General /> },
  {
    id: "media-sources",
    label: "Media sources",
    icon: Film,
    content: () => (
      <>
        <GamesKeyCard />
        <TmdbTokenCard />
        <Card>
          <SourceCredits />
        </Card>
      </>
    ),
  },
  { id: "sync", label: "Sync", icon: Smartphone, content: () => <SyncCard /> },
  { id: "reminders", label: "Reminders", icon: Bell, content: () => <ReminderSettingsCard /> },
  { id: "backups", label: "Backups", icon: DatabaseBackup, content: () => <BackupCard /> },
  { id: "templates", label: "Templates", icon: FileText, content: () => <TemplateSettings /> },
  { id: "about", label: "About", icon: Info, content: () => <About /> },
];

export function SettingsPage() {
  const { section: sectionId } = useParams();
  const section = SECTIONS.find((candidate) => candidate.id === sectionId) ?? SECTIONS[0];

  return (
    <div className="flex flex-col gap-6">
      <PageHeader
        title="Settings"
        subtitle="Sync, backups, timers, reminders and API keys"
        icon={Settings}
      />
      <div className="grid grid-cols-[12rem_1fr] items-start gap-8">
        <nav className="sticky top-0 flex flex-col gap-1" aria-label="Settings sections">
          {SECTIONS.map(({ id, label, icon: Icon }) => (
            <NavLink
              key={id}
              to={`/settings/${id}`}
              className={cn(
                "flex items-center gap-3 rounded-xl px-3 py-2 text-sm text-muted-foreground transition-[background-color,color] duration-(--duration-fast) hover:text-foreground",
                id === section.id && "bg-card font-medium text-foreground shadow-soft",
              )}
            >
              <Icon size={16} />
              {label}
            </NavLink>
          ))}
        </nav>
        <div key={section.id} className="flex min-w-0 animate-fade-in flex-col gap-6">
          {section.content()}
        </div>
      </div>
    </div>
  );
}

function General() {
  return (
    <>
      <Card className="flex items-center justify-between gap-4">
        <div className="flex flex-col gap-0.5">
          <h2 className="font-semibold tracking-tight">Theme</h2>
          <p className="text-sm text-muted-foreground">Light, dark, or follow the system.</p>
        </div>
        <ThemeToggle />
      </Card>
      <TimerSettingsCard />
    </>
  );
}

function About() {
  const { data, error } = useQuery({ queryKey: ["app-info"], queryFn: getAppInfo });
  return (
    <Card>
      <h2 className="mb-3 font-semibold tracking-tight">About</h2>
      {error ? (
        <p className="text-sm text-destructive">Could not load app info.</p>
      ) : (
        <dl className="grid grid-cols-[8rem_1fr] gap-2 text-sm">
          <dt className="text-muted-foreground">Version</dt>
          <dd>{data?.version}</dd>
          <dt className="text-muted-foreground">Device ID</dt>
          <dd className="break-all">{data?.device_id}</dd>
          <dt className="text-muted-foreground">Database</dt>
          <dd className="break-all">{data?.db_path}</dd>
        </dl>
      )}
    </Card>
  );
}
