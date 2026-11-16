import type { GraphData, GraphEdge, GraphKind, GraphNode } from "@/lib/api/vault";

export const KINDS: { id: GraphKind; label: string }[] = [
  { id: "note", label: "Notes" },
  { id: "project", label: "Projects" },
  { id: "daily", label: "Days" },
  { id: "tag", label: "Tags" },
  { id: "job", label: "Applications" },
  { id: "company", label: "Companies" },
];

export type ViewOptions = {
  kinds: Set<GraphKind>;
  query: string;
  /** with a node selected, only it and what is within `depth` steps */
  focusId: string | null;
  depth: number;
};

export type Neighbors = Map<string, Set<string>>;

export function neighborsOf(edges: GraphEdge[]): Neighbors {
  const map: Neighbors = new Map();
  const add = (from: string, to: string) => {
    const set = map.get(from) ?? new Set<string>();
    set.add(to);
    map.set(from, set);
  };
  for (const edge of edges) {
    add(edge.source, edge.target);
    add(edge.target, edge.source);
  }
  return map;
}

/** Ids within `depth` steps of `start`, the start included. */
export function withinSteps(neighbors: Neighbors, start: string, depth: number): Set<string> {
  const seen = new Set([start]);
  let frontier = [start];
  for (let step = 0; step < depth; step += 1) {
    const next: string[] = [];
    for (const id of frontier) {
      for (const other of neighbors.get(id) ?? []) {
        if (!seen.has(other)) {
          seen.add(other);
          next.push(other);
        }
      }
    }
    frontier = next;
  }
  return seen;
}

/** The nodes and edges to draw: by kind, by the local focus, and by the search text. */
export function visibleGraph(data: GraphData, options: ViewOptions): GraphData {
  const local =
    options.focusId === null
      ? null
      : withinSteps(neighborsOf(data.edges), options.focusId, options.depth);
  const query = options.query.trim().toLowerCase();
  const nodes = data.nodes.filter(
    (node) =>
      options.kinds.has(node.kind) &&
      (local === null || local.has(node.id)) &&
      (query === "" || node.label.toLowerCase().includes(query)),
  );
  const ids = new Set(nodes.map((node) => node.id));
  return { nodes, edges: data.edges.filter((e) => ids.has(e.source) && ids.has(e.target)) };
}

/** How many links each node has, for sizing the hubs. */
export function degrees(edges: GraphEdge[]): Map<string, number> {
  const counts = new Map<string, number>();
  for (const edge of edges) {
    counts.set(edge.source, (counts.get(edge.source) ?? 0) + 1);
    counts.set(edge.target, (counts.get(edge.target) ?? 0) + 1);
  }
  return counts;
}

/** The obsidian:// address of a note in the vault whose folder is `vaultPath`. */
export function obsidianUrl(vaultPath: string, notePath: string): string {
  const vault =
    vaultPath
      .replace(/[\\/]+$/, "")
      .split(/[\\/]/)
      .pop() ?? "";
  const file = notePath.replace(/\.md$/, "");
  return `obsidian://open?vault=${encodeURIComponent(vault)}&file=${encodeURIComponent(file)}`;
}

export function nodeTitle(node: GraphNode): string {
  return node.kind === "tag" ? `#${node.label}` : node.label;
}
