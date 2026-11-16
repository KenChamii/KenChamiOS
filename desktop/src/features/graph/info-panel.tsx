import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import type { GraphData, GraphNode } from "@/lib/api/vault";
import { neighborsOf, nodeTitle } from "./model";

const KIND_LABEL = {
  note: "Note",
  project: "Project",
  daily: "Day",
  tag: "Tag",
  job: "Application",
  company: "Company",
} as const;

type Props = {
  node: GraphNode;
  graph: GraphData;
  onSelect: (id: string) => void;
  onOpen: () => void;
  onLocal: () => void;
  isLocal: boolean;
};

export function InfoPanel({ node, graph, onSelect, onOpen, onLocal, isLocal }: Props) {
  const byId = new Map(graph.nodes.map((item) => [item.id, item]));
  const linked = [...(neighborsOf(graph.edges).get(node.id) ?? [])]
    .map((id) => byId.get(id))
    .filter((item) => item !== undefined);

  return (
    <Card className="flex max-h-full flex-col gap-3 overflow-auto">
      <div className="flex flex-col gap-1">
        <span className="text-label text-muted-foreground">{KIND_LABEL[node.kind]}</span>
        <h2 className="break-words font-semibold tracking-tight">{nodeTitle(node)}</h2>
        <span className="text-sm text-muted-foreground">{linked.length} connections</span>
      </div>
      <div className="flex flex-wrap gap-2">
        {node.kind !== "tag" && node.kind !== "company" && (
          <Button size="sm" onClick={onOpen}>
            {node.kind === "note" ? "Open in Obsidian" : "Open"}
          </Button>
        )}
        <Button size="sm" variant="outline" onClick={onLocal}>
          {isLocal ? "Show everything" : "Local graph"}
        </Button>
      </div>
      {linked.length > 0 && (
        <ul className="flex flex-col gap-1 text-sm">
          {linked.map((item) => (
            <li key={item.id}>
              <button
                type="button"
                onClick={() => onSelect(item.id)}
                className="w-full truncate rounded-lg px-2 py-1 text-left hover:bg-muted"
              >
                {nodeTitle(item)}
              </button>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}
