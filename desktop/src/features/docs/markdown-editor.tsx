import { markdown } from "@codemirror/lang-markdown";
import { EditorView, minimalSetup } from "codemirror";
import { useEffect, useRef, useState } from "react";

const theme = EditorView.theme({
  "&": { backgroundColor: "var(--card)", color: "var(--foreground)", minHeight: "20rem" },
  ".cm-content": { caretColor: "var(--foreground)", padding: "0.75rem" },
  ".cm-cursor": { borderLeftColor: "var(--foreground)" },
  "&.cm-focused": { outline: "2px solid var(--primary)" },
  ".cm-activeLine": { backgroundColor: "transparent" },
});

type MarkdownEditorProps = {
  initialValue: string;
  onChange: (value: string) => void;
};

// uncontrolled: the editor owns the text after mount, so the cursor never jumps
export function MarkdownEditor({ initialValue, onChange }: MarkdownEditorProps) {
  const container = useRef<HTMLDivElement>(null);
  const onChangeRef = useRef(onChange);
  const [startValue] = useState(initialValue);

  useEffect(() => {
    onChangeRef.current = onChange;
  });

  useEffect(() => {
    if (!container.current) return;
    const view = new EditorView({
      doc: startValue,
      parent: container.current,
      extensions: [
        minimalSetup,
        markdown(),
        EditorView.lineWrapping,
        theme,
        EditorView.updateListener.of((update) => {
          if (update.docChanged) onChangeRef.current(update.state.doc.toString());
        }),
      ],
    });
    return () => view.destroy();
  }, [startValue]);

  return <div ref={container} className="overflow-hidden rounded-xl" />;
}
