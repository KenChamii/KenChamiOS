import { openUrl } from "@tauri-apps/plugin-opener";
import type { UseMutationResult, UseQueryResult } from "@tanstack/react-query";
import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { errorMessage } from "@/lib/errors";

type ApiKeyCardProps = {
  title: string;
  intro: string;
  helpUrl: string;
  /** what the secret is called, e.g. "RAWG API key" */
  label: string;
  status: UseQueryResult<boolean>;
  save: UseMutationResult<void, Error, string>;
  clear: UseMutationResult<void, Error, void>;
};

export function ApiKeyCard({ title, intro, helpUrl, label, status, save, clear }: ApiKeyCardProps) {
  const [value, setValue] = useState("");

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    save.mutate(value.trim(), { onSuccess: () => setValue("") });
  }

  const error = status.error ?? save.error ?? clear.error;

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-medium">{title}</h2>
      <p className="text-sm text-muted-foreground">
        {intro} It is kept in the Windows Credential Manager, never in the app database.{" "}
        <button type="button" className="underline" onClick={() => openUrl(helpUrl)}>
          Get one
        </button>
      </p>
      <p className="text-sm">
        {status.data ? "Saved." : "Nothing saved yet."}
        {status.data && (
          <Button
            variant="ghost"
            size="sm"
            className="ml-2"
            disabled={clear.isPending}
            onClick={() => clear.mutate()}
          >
            Remove
          </Button>
        )}
      </p>
      <form onSubmit={handleSubmit} className="flex flex-wrap items-center gap-2">
        <Input
          className="max-w-sm flex-1"
          type="password"
          autoComplete="off"
          placeholder={status.data ? "Replace the saved value" : label}
          aria-label={label}
          value={value}
          onChange={(event) => setValue(event.target.value)}
        />
        <Button type="submit" disabled={value.trim() === "" || save.isPending}>
          Save
        </Button>
      </form>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
    </Card>
  );
}
