import { Select } from "@/components/ui/select";
import { ESTIMATES, formatMinutes } from "./capacity";

const NONE = "none";

type EstimateSelectProps = {
  value: number | null;
  onChange: (minutes: number | null) => void;
  disabled?: boolean;
};

export function EstimateSelect({ value, onChange, disabled }: EstimateSelectProps) {
  const choices: number[] = [...ESTIMATES];
  // keep an odd length from a resized block selectable
  if (value !== null && !choices.includes(value)) choices.push(value);
  choices.sort((a, b) => a - b);

  return (
    <Select
      aria-label="Estimate"
      className="w-28 shrink-0"
      disabled={disabled}
      value={value === null ? NONE : String(value)}
      options={[
        { value: NONE, label: "No estimate" },
        ...choices.map((minutes) => ({ value: String(minutes), label: formatMinutes(minutes) })),
      ]}
      onChange={(next) => onChange(next === NONE ? null : Number(next))}
    />
  );
}
