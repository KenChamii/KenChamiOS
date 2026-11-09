import { Plus } from "lucide-react";
import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

type AddItemFormProps = {
  placeholder: string;
  pending: boolean;
  onAdd: (text: string) => void;
};

export function AddItemForm({ placeholder, pending, onAdd }: AddItemFormProps) {
  const [text, setText] = useState("");

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const trimmed = text.trim();
    if (!trimmed) return;
    onAdd(trimmed);
    setText("");
  }

  return (
    <form onSubmit={handleSubmit} className="flex gap-2">
      <Input placeholder={placeholder} value={text} onChange={(e) => setText(e.target.value)} />
      <Button type="submit" variant="outline" size="icon" aria-label="Add" disabled={pending}>
        <Plus size={16} />
      </Button>
    </form>
  );
}
