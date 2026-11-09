import { useQuery } from "@tanstack/react-query";
import { Card } from "@/components/ui/card";
import { getAppInfo } from "@/lib/api";

export function SettingsPage() {
  const { data, error } = useQuery({ queryKey: ["app-info"], queryFn: getAppInfo });

  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-2xl font-semibold">Settings</h1>
      <Card>
        <h2 className="mb-3 font-medium">About</h2>
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
    </div>
  );
}
