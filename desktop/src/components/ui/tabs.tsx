import { cn } from "@/lib/cn";

type Tab<T extends string> = { id: T; label: string };

type TabsProps<T extends string> = {
  tabs: Tab<T>[];
  value: T;
  onChange: (id: T) => void;
};

export function Tabs<T extends string>({ tabs, value, onChange }: TabsProps<T>) {
  return (
    <div role="tablist" className="flex gap-1 border-b border-border">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          role="tab"
          type="button"
          aria-selected={tab.id === value}
          onClick={() => onChange(tab.id)}
          className={cn(
            "-mb-px border-b-2 border-transparent px-3 py-2 text-sm text-muted-foreground hover:text-foreground",
            tab.id === value && "border-primary font-medium text-foreground",
          )}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
