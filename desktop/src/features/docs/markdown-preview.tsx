import Markdown from "react-markdown";

export function MarkdownPreview({ source }: { source: string }) {
  return (
    <div className="markdown min-h-80 rounded-xl bg-muted p-4">
      {source.trim() ? (
        <Markdown>{source}</Markdown>
      ) : (
        <p className="text-sm text-muted-foreground">Nothing to preview yet.</p>
      )}
    </div>
  );
}
