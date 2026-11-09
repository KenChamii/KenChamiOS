import { Check } from "lucide-react";
import { cn } from "@/lib/cn";

type CheckboxProps = {
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
  "aria-label"?: string;
};

export function Checkbox({
  checked,
  onCheckedChange,
  disabled,
  className,
  "aria-label": ariaLabel,
}: CheckboxProps) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      aria-label={ariaLabel}
      disabled={disabled}
      onClick={(event) => {
        // rows behind the box often open something on click
        event.stopPropagation();
        onCheckedChange(!checked);
      }}
      className={cn(
        "inline-flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded-md border-2 border-border transition-[background-color,border-color,transform] duration-(--duration-fast) ease-(--ease-out) active:scale-90 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:pointer-events-none disabled:opacity-50",
        checked && "border-primary bg-primary text-primary-foreground",
        className,
      )}
    >
      {checked && <Check size={13} strokeWidth={3} className="animate-tick" />}
    </button>
  );
}
