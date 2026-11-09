import { Badge } from "@/components/ui/badge";
import { Card } from "@/components/ui/card";
import { TYPE_LABELS } from "@/features/media/meta";
import { useRecommendations } from "@/hooks/use-media";
import { errorMessage } from "@/lib/errors";

export function RecommendationsCard() {
  const recommendations = useRecommendations(null);

  return (
    <Card className="flex flex-col gap-3">
      <h2 className="font-medium">From your plans</h2>
      <p className="text-sm text-muted-foreground">
        Planned titles that match what you rated well. Worked out on this computer from your own
        ratings, genres and tags.
      </p>
      {recommendations.error && (
        <p className="text-sm text-destructive">{errorMessage(recommendations.error)}</p>
      )}
      {recommendations.data?.length === 0 && (
        <p className="text-sm">Rate a few finished titles and plan some more to see matches.</p>
      )}
      <ul className="flex flex-col gap-2">
        {recommendations.data?.map((item) => (
          <li key={item.entry_id} className="flex flex-col">
            <span className="flex items-center gap-2 text-sm font-medium">
              <span className="truncate">{item.title}</span>
              <Badge>{TYPE_LABELS[item.media_type]}</Badge>
            </span>
            <span className="text-xs text-muted-foreground">{item.reason}</span>
          </li>
        ))}
      </ul>
    </Card>
  );
}
