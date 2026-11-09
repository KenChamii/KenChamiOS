import { Image, Link as LinkIcon, Trash2 } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useDeleteAsset, useSetAssetCaption } from "@/hooks/use-assets";
import type { ProjectAsset } from "@/lib/api/assets";

export function AssetRow({ asset }: { asset: ProjectAsset }) {
  const setCaption = useSetAssetCaption(asset.project_id);
  const remove = useDeleteAsset(asset.project_id);
  const [caption, setDraft] = useState(asset.caption);
  const Icon = asset.kind === "screenshot" ? Image : LinkIcon;

  return (
    <li className="flex items-center gap-3 py-2">
      <Icon size={16} className="shrink-0 text-muted-foreground" />
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        <Input
          aria-label="Caption"
          placeholder="Caption"
          value={caption}
          onChange={(event) => setDraft(event.target.value)}
          onBlur={() => caption !== asset.caption && setCaption.mutate({ id: asset.id, caption })}
        />
        {asset.kind === "link" && (
          <span className="truncate text-xs text-muted-foreground">{asset.path_or_url}</span>
        )}
      </div>
      <Button
        size="icon"
        variant="ghost"
        aria-label="Remove"
        title="Remove"
        disabled={remove.isPending}
        onClick={() => remove.mutate(asset.id)}
      >
        <Trash2 size={16} />
      </Button>
    </li>
  );
}
