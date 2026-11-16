import ForceGraph2D, { type ForceGraphMethods } from "react-force-graph-2d";
import { useEffect, useMemo, useRef, useState } from "react";
import type { GraphData, GraphKind } from "@/lib/api/vault";
import { degrees, neighborsOf } from "./model";

export type Effects = { glow: boolean; particles: boolean; pulse: boolean };

const KIND_VARS: Record<GraphKind, string> = {
  note: "--workflow",
  project: "--projects",
  daily: "--focus",
  tag: "--media",
  job: "--accent",
  company: "--muted-foreground",
};

function kindColor(kind: GraphKind): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(KIND_VARS[kind]);
  return value.trim() || "#888";
}

type Props = {
  graph: GraphData;
  selectedId: string | null;
  effects: Effects;
  onSelect: (id: string | null) => void;
};

type DrawnNode = {
  id?: string | number;
  x?: number;
  y?: number;
  kind: GraphKind;
  label: string;
};

export function GraphCanvas({ graph, selectedId, effects, onSelect }: Props) {
  const ref = useRef<ForceGraphMethods<DrawnNode> | undefined>(undefined);
  const box = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ width: 600, height: 500 });
  const [hoverId, setHoverId] = useState<string | null>(null);
  const fitted = useRef(false);

  useEffect(() => {
    const element = box.current;
    if (!element) return;
    const watch = new ResizeObserver(([entry]) =>
      setSize({ width: entry.contentRect.width, height: entry.contentRect.height }),
    );
    watch.observe(element);
    return () => watch.disconnect();
  }, []);

  // a new set of nodes gets laid out and fitted again
  useEffect(() => {
    fitted.current = false;
  }, [graph]);

  const data = useMemo(
    () => ({
      nodes: graph.nodes.map((node) => ({ ...node })),
      links: graph.edges.map((edge) => ({ ...edge })),
    }),
    [graph],
  );
  const counts = useMemo(() => degrees(graph.edges), [graph.edges]);
  const neighbors = useMemo(() => neighborsOf(graph.edges), [graph.edges]);
  const lit = hoverId ?? selectedId;
  const near = lit === null ? null : new Set([lit, ...(neighbors.get(lit) ?? [])]);

  function paintNode(node: DrawnNode, ctx: CanvasRenderingContext2D, scale: number) {
    // the first frame comes before the layout has given the node a place
    if (node.x === undefined || node.y === undefined) return;
    const id = String(node.id);
    const radius = 3 + Math.sqrt(counts.get(id) ?? 0) * 1.6;
    const color = kindColor(node.kind);
    const dim = near !== null && !near.has(id);
    ctx.globalAlpha = dim ? 0.15 : 1;
    ctx.beginPath();
    ctx.arc(node.x, node.y, radius, 0, 2 * Math.PI);
    ctx.fillStyle = color;
    if (effects.glow && (counts.get(id) ?? 0) > 3) {
      ctx.shadowColor = color;
      ctx.shadowBlur = 12;
    }
    ctx.fill();
    ctx.shadowBlur = 0;
    if (effects.pulse && id === selectedId) {
      const wave = (Date.now() % 1200) / 1200;
      ctx.beginPath();
      ctx.arc(node.x, node.y, radius + 3 + wave * 8, 0, 2 * Math.PI);
      ctx.strokeStyle = color;
      ctx.globalAlpha = 1 - wave;
      ctx.lineWidth = 1.5 / scale;
      ctx.stroke();
      ctx.globalAlpha = dim ? 0.15 : 1;
    }
    if (scale > 1.6 || id === lit) {
      ctx.font = `${11 / scale}px sans-serif`;
      ctx.fillStyle = "#e8e8f0";
      ctx.textAlign = "center";
      ctx.fillText(node.label, node.x, node.y + radius + 10 / scale);
    }
    ctx.globalAlpha = 1;
  }

  const linked = (link: { source?: unknown; target?: unknown }) => {
    const end = (value: unknown) =>
      typeof value === "object" && value !== null
        ? String((value as { id: string }).id)
        : String(value);
    return lit !== null && (end(link.source) === lit || end(link.target) === lit);
  };

  return (
    <div ref={box} className="h-full min-h-96 w-full overflow-hidden rounded-2xl bg-card">
      <ForceGraph2D
        ref={ref}
        width={size.width}
        height={size.height}
        graphData={data}
        backgroundColor="rgba(0,0,0,0)"
        nodeRelSize={4}
        nodeCanvasObject={paintNode}
        nodeCanvasObjectMode={() => "replace"}
        nodePointerAreaPaint={(node: DrawnNode, color, ctx) => {
          if (node.x === undefined || node.y === undefined) return;
          ctx.fillStyle = color;
          ctx.beginPath();
          ctx.arc(node.x, node.y, 6, 0, 2 * Math.PI);
          ctx.fill();
        }}
        linkColor={(link) => (linked(link) ? "rgba(200,200,255,0.7)" : "rgba(150,150,170,0.25)")}
        linkWidth={(link) => (linked(link) ? 1.5 : 0.6)}
        linkCurvature={0.12}
        linkDirectionalParticles={(link) => (effects.particles && linked(link) ? 3 : 0)}
        linkDirectionalParticleSpeed={0.008}
        autoPauseRedraw={!(effects.pulse || effects.particles)}
        onNodeHover={(node) => setHoverId(node ? String(node.id) : null)}
        onNodeClick={(node) => onSelect(String(node.id))}
        onBackgroundClick={() => onSelect(null)}
        onEngineStop={() => {
          if (fitted.current) return;
          fitted.current = true;
          ref.current?.zoomToFit(400, 60);
        }}
      />
    </div>
  );
}
