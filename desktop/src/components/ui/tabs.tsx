import { cn } from "@/lib/cn";

type Tab<T extends string> = { id: T; label: string };

type TabsProps<T extends string> = {
  tabs: Tab<T>[];
  value: T;
  onChange: (id: T) => void;
};

export function Tabs<T extends string>({ tabs, value, onChange }: TabsProps<T>) {
  return (
    <div
      role="tablist"
      className="flex w-fit max-w-full gap-1 overflow-x-auto rounded-xl bg-muted p-1"
    >
      {tabs.map((tab) => (
        <button
          key={tab.id}
          role="tab"
          type="button"
          aria-selected={tab.id === value}
          onClick={() => onChange(tab.id)}
          className={cn(
            "cursor-pointer rounded-lg px-3.5 py-1.5 text-sm text-muted-foreground transition-[background-color,color,box-shadow] duration-(--duration-fast) ease-(--ease-out) hover:text-foreground",
            tab.id === value && "bg-card font-medium text-foreground shadow-soft",
          )}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
