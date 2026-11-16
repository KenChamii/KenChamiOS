import { openUrl } from "@tauri-apps/plugin-opener";
import { Waypoints } from "lucide-react";
import { useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { PageHeader } from "@/components/layout/page-header";
import { Chip } from "@/components/ui/chips";
import { EmptyState } from "@/components/ui/empty-state";
import { Input } from "@/components/ui/input";
import { GraphCanvas, type Effects } from "@/features/graph/graph-canvas";
import { InfoPanel } from "@/features/graph/info-panel";
import { KINDS, obsidianUrl, visibleGraph } from "@/features/graph/model";
import { useGraph, useVaultStatus } from "@/hooks/use-vault";
import type { GraphKind, GraphNode } from "@/lib/api/vault";
import { errorMessage } from "@/lib/errors";

const LOCAL_DEPTH = 2;

function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

const EFFECT_NAMES: { id: keyof Effects; label: string }[] = [
  { id: "glow", label: "Glow" },
  { id: "particles", label: "Flow" },
  { id: "pulse", label: "Pulse" },
];

export function GraphPage() {
  const navigate = useNavigate();
  const graph = useGraph();
  const vault = useVaultStatus();
  const [kinds, setKinds] = useState<Set<GraphKind>>(() => new Set(KINDS.map((kind) => kind.id)));
  const [query, setQuery] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [local, setLocal] = useState(false);
  const [effects, setEffects] = useState<Effects>(() => {
    const calm = prefersReducedMotion();
    return { glow: !calm, particles: !calm, pulse: !calm };
  });

  const shown = useMemo(
    () =>
      graph.data
        ? visibleGraph(graph.data, {
            kinds,
            query,
            focusId: local ? selectedId : null,
            depth: LOCAL_DEPTH,
          })
        : { nodes: [], edges: [] },
    [graph.data, kinds, query, local, selectedId],
  );
  const selected = graph.data?.nodes.find((node) => node.id === selectedId) ?? null;

  function toggleKind(kind: GraphKind) {
    const next = new Set(kinds);
    if (!next.delete(kind)) next.add(kind);
    setKinds(next);
  }

  function open(node: GraphNode) {
    if (node.kind === "project") navigate(`/projects/${node.id.slice("project:".length)}`);
    else if (node.kind === "job") navigate("/jobs");
    else if (node.kind === "daily") navigate(`/notes?day=${node.label}`);
    else if (node.kind === "note" && node.path && vault.data) {
      void openUrl(obsidianUrl(vault.data.path, node.path));
    }
  }

  return (
    <div className="flex h-full flex-col gap-4">
      <PageHeader
        title="Graph"
        subtitle="How your notes, projects and days connect"
        icon={Waypoints}
        area="media"
      >
        <Input
          aria-label="Search the graph"
          className="w-56"
          placeholder="Search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
        />
      </PageHeader>
      <div className="flex flex-wrap items-center gap-2">
        {KINDS.map((kind) => (
          <Chip key={kind.id} selected={kinds.has(kind.id)} onClick={() => toggleKind(kind.id)}>
            {kind.label}
          </Chip>
        ))}
        <span className="mx-2 h-5 w-px bg-border" />
        {EFFECT_NAMES.map((effect) => (
          <Chip
            key={effect.id}
            selected={effects[effect.id]}
            onClick={() => setEffects({ ...effects, [effect.id]: !effects[effect.id] })}
          >
            {effect.label}
          </Chip>
        ))}
      </div>
      {graph.error && <p className="text-sm text-destructive">{errorMessage(graph.error)}</p>}
      {graph.data && graph.data.nodes.length === 0 ? (
        <EmptyState
          icon={Waypoints}
          area="media"
          line="Nothing to draw yet. Set your Obsidian vault in Settings, or add projects."
        />
      ) : (
        <div className="grid min-h-0 flex-1 grid-cols-[1fr_18rem] gap-4">
          <GraphCanvas
            graph={shown}
            selectedId={selectedId}
            effects={effects}
            onSelect={setSelectedId}
          />
          {selected ? (
            <InfoPanel
              node={selected}
              graph={graph.data ?? { nodes: [], edges: [] }}
              onSelect={setSelectedId}
              onOpen={() => open(selected)}
              onLocal={() => setLocal(!local)}
              isLocal={local}
            />
          ) : (
            <p className="text-sm text-muted-foreground">
              Click a dot to see what it connects to. Hover to light up its links.
            </p>
          )}
        </div>
      )}
    </div>
  );
}
