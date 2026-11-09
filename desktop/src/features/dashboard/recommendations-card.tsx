import { Link } from "react-router-dom";
import { Card } from "@/components/ui/card";
import { TypeBadge } from "@/features/media/type-badge";
import { useRecommendations } from "@/hooks/use-media";
import { errorMessage } from "@/lib/errors";

/** Planned titles that match what was rated well; hidden when there are none. */
export function RecommendationsCard() {
  const recommendations = useRecommendations(null);
  if (!recommendations.error && !recommendations.data?.length) return null;

  return (
    <Card className="flex flex-col gap-3">
      <div className="flex flex-col gap-0.5">
        <h2 className="font-semibold tracking-tight">From your plans</h2>
        <p className="text-sm text-muted-foreground">
          Matched on this computer from your own ratings, genres and tags.
        </p>
      </div>
      {recommendations.error && (
        <p className="text-sm text-destructive">{errorMessage(recommendations.error)}</p>
      )}
      <ul className="grid gap-x-6 gap-y-1 sm:grid-cols-2">
        {recommendations.data?.map((item) => (
          <li key={item.entry_id}>
            <Link
              to={`/media?open=${item.entry_id}`}
              className="flex flex-col rounded-xl px-2 py-1.5 transition-colors duration-(--duration-fast) hover:bg-muted"
            >
              <span className="flex items-center gap-2 text-sm font-medium">
                <span className="truncate">{item.title}</span>
                <TypeBadge type={item.media_type} />
              </span>
              <span className="text-xs text-muted-foreground">{item.reason}</span>
            </Link>
          </li>
        ))}
      </ul>
    </Card>
  );
}
