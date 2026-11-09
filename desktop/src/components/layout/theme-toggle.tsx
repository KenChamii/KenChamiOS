import { Monitor, Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useThemeStore, type Theme } from "@/stores/theme";

const options: { value: Theme; label: string; icon: typeof Sun }[] = [
  { value: "light", label: "Light", icon: Sun },
  { value: "dark", label: "Dark", icon: Moon },
  { value: "system", label: "System", icon: Monitor },
];

export function ThemeToggle() {
  const { theme, setTheme } = useThemeStore();
  return (
    <div className="flex gap-1" role="group" aria-label="Theme">
      {options.map(({ value, label, icon: Icon }) => (
        <Button
          key={value}
          size="icon"
          variant={theme === value ? "outline" : "ghost"}
          aria-label={label}
          aria-pressed={theme === value}
          onClick={() => setTheme(value)}
        >
          <Icon size={16} />
        </Button>
      ))}
    </div>
  );
}
