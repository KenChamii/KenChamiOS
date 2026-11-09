import { openUrl } from "@tauri-apps/plugin-opener";
import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useClearGamesKey, useGamesKeyStatus, useSetGamesKey } from "@/hooks/use-media";
import { errorMessage } from "@/lib/errors";

export function GamesKeyCard() {
  const status = useGamesKeyStatus();
  const save = useSetGamesKey();
  const clear = useClearGamesKey();
  const [key, setKey] = useState("");

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    save.mutate(key.trim(), { onSuccess: () => setKey("") });
  }

  const error = status.error ?? save.error ?? clear.error;

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-medium">Games search</h2>
      <p className="text-sm text-muted-foreground">
        Game search uses RAWG and needs a free API key. The key is kept in the Windows Credential
        Manager, never in the app database.{" "}
        <button
          type="button"
          className="underline"
          onClick={() => openUrl("https://rawg.io/apidocs")}
        >
          Get a key
        </button>
      </p>
      <p className="text-sm">
        {status.data ? "A key is saved." : "No key saved yet."}
        {status.data && (
          <Button
            variant="ghost"
            size="sm"
            className="ml-2"
            disabled={clear.isPending}
            onClick={() => clear.mutate()}
          >
            Remove key
          </Button>
        )}
      </p>
      <form onSubmit={handleSubmit} className="flex flex-wrap items-center gap-2">
        <Input
          className="max-w-sm flex-1"
          type="password"
          autoComplete="off"
          placeholder={status.data ? "Replace the saved key" : "RAWG API key"}
          aria-label="RAWG API key"
          value={key}
          onChange={(event) => setKey(event.target.value)}
        />
        <Button type="submit" disabled={key.trim() === "" || save.isPending}>
          Save key
        </Button>
      </form>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <p className="text-xs text-muted-foreground">
        Game data courtesy of{" "}
        <button type="button" className="underline" onClick={() => openUrl("https://rawg.io")}>
          RAWG
        </button>
        . Anime and manga data courtesy of{" "}
        <button type="button" className="underline" onClick={() => openUrl("https://anilist.co")}>
          AniList
        </button>
        .
      </p>
    </Card>
  );
}
