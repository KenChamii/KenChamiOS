import Markdown from "react-markdown";

export function MarkdownPreview({ source }: { source: string }) {
  return (
    <div className="markdown min-h-80 rounded-md border border-border bg-card p-4">
      {source.trim() ? (
        <Markdown>{source}</Markdown>
      ) : (
        <p className="text-sm text-muted-foreground">Nothing to preview yet.</p>
      )}
    </div>
  );
}
