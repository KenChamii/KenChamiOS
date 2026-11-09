import { Copy, Pencil, Star, Trash2 } from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import { useDeletePrompt, useSetPromptFavorite } from "@/hooks/use-prompts";
import type { Prompt } from "@/lib/api/prompts";
import { cn } from "@/lib/cn";
import { timeAgo } from "@/lib/dates";
import { errorMessage } from "@/lib/errors";

type PromptCardProps = {
  prompt: Prompt;
  projectName: string | null;
  onCopy: (prompt: Prompt) => void;
  onEdit: (prompt: Prompt) => void;
};

export function PromptCard({ prompt, projectName, onCopy, onEdit }: PromptCardProps) {
  const favorite = useSetPromptFavorite();
  const remove = useDeletePrompt();
  const [confirmingDelete, setConfirmingDelete] = useState(false);
  const error = favorite.error ?? remove.error;

  return (
    <Card className="flex flex-col gap-2">
      <div className="flex items-start gap-2">
        <div className="flex min-w-0 flex-1 flex-col gap-1">
          <h3 className="font-medium">{prompt.title}</h3>
          <div className="flex flex-wrap items-center gap-1">
            <Badge>{projectName ?? "Global"}</Badge>
            {prompt.tags.map((tag) => (
              <Badge key={tag}>{tag}</Badge>
            ))}
            <span className="text-xs text-muted-foreground">
              {prompt.use_count === 0
                ? "Never used"
                : `Used ${prompt.use_count} times, last ${timeAgo(prompt.last_used_at ?? 0)}`}
            </span>
          </div>
        </div>
        <Button
          variant="ghost"
          size="icon"
          aria-label={prompt.favorite ? "Remove from favorites" : "Add to favorites"}
          aria-pressed={prompt.favorite}
          disabled={favorite.isPending}
          onClick={() => favorite.mutate({ id: prompt.id, favorite: !prompt.favorite })}
        >
          <Star size={16} className={cn(prompt.favorite && "fill-current")} />
        </Button>
      </div>
      <p className="line-clamp-3 whitespace-pre-wrap font-mono text-xs text-muted-foreground">
        {prompt.body_md}
      </p>
      {error && <p className="text-sm text-destructive">{errorMessage(error)}</p>}
      <div className="flex items-center gap-2">
        <Button size="sm" onClick={() => onCopy(prompt)}>
          <Copy size={14} />
          Copy
        </Button>
        <Button size="sm" variant="outline" onClick={() => onEdit(prompt)}>
          <Pencil size={14} />
          Edit
        </Button>
        {confirmingDelete ? (
          <>
            <Button
              size="sm"
              variant="destructive"
              disabled={remove.isPending}
              onClick={() => remove.mutate(prompt.id)}
            >
              Confirm delete
            </Button>
            <Button size="sm" variant="ghost" onClick={() => setConfirmingDelete(false)}>
              Cancel
            </Button>
          </>
        ) : (
          <Button size="sm" variant="ghost" onClick={() => setConfirmingDelete(true)}>
            <Trash2 size={14} />
            Delete
          </Button>
        )}
      </div>
    </Card>
  );
}
