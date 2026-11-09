import Markdown from "react-markdown";

type SavePointBodyProps = { title: string; source: string };

export function SavePointBody({ title, source }: SavePointBodyProps) {
  if (!source.trim()) return null;
  return (
    <div className="flex flex-col gap-1">
      <h4 className="text-xs font-medium uppercase text-muted-foreground">{title}</h4>
      <div className="markdown text-sm">
        <Markdown>{source}</Markdown>
      </div>
    </div>
  );
}
