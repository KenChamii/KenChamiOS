import { open } from "@tauri-apps/plugin-dialog";
import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useAddLink, useAddScreenshot, useAssets } from "@/hooks/use-assets";
import { errorMessage } from "@/lib/errors";
import { AssetRow } from "./asset-row";

const IMAGE_FILTER = { name: "Images", extensions: ["png", "jpg", "jpeg", "gif", "webp"] };

export function AssetsCard({ projectId }: { projectId: string }) {
  const assets = useAssets(projectId);
  const addScreenshot = useAddScreenshot(projectId);
  const addLink = useAddLink(projectId);
  const [url, setUrl] = useState("");
  const [caption, setCaption] = useState("");

  async function pickScreenshot() {
    const file = await open({ multiple: false, directory: false, filters: [IMAGE_FILTER] });
    if (file) addScreenshot.mutate(file);
  }

  function handleAddLink(event: FormEvent) {
    event.preventDefault();
    addLink.mutate(
      { url, caption },
      {
        onSuccess: () => {
          setUrl("");
          setCaption("");
        },
      },
    );
  }

  const error = assets.error ?? addScreenshot.error ?? addLink.error;
  const items = assets.data ?? [];

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-medium">Screenshots and links</h2>
      <p className="text-sm text-muted-foreground">
        Shown in the portfolio page. Screenshots are copied into the app, so the original can move.
      </p>
      {items.length > 0 && (
        <ul className="flex flex-col divide-y divide-border">
          {items.map((asset) => (
            <AssetRow key={`${asset.id}:${asset.caption}`} asset={asset} />
          ))}
        </ul>
      )}
      <div>
        <Button variant="outline" disabled={addScreenshot.isPending} onClick={pickScreenshot}>
          Add screenshot
        </Button>
      </div>
      <form onSubmit={handleAddLink} className="flex flex-wrap items-center gap-2">
        <Input
          className="max-w-xs flex-1"
          type="url"
          required
          placeholder="https://..."
          aria-label="Link address"
          value={url}
          onChange={(event) => setUrl(event.target.value)}
        />
        <Input
          className="max-w-xs flex-1"
          placeholder="Caption, for example Repository or Demo"
          aria-label="Link caption"
          value={caption}
          onChange={(event) => setCaption(event.target.value)}
        />
        <Button type="submit" variant="outline" disabled={addLink.isPending}>
          Add link
        </Button>
      </form>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
    </Card>
  );
}
