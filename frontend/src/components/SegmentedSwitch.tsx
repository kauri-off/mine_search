import { cn } from "@/cn";

interface SegmentedOption<T extends string> {
  value: T;
  label: string;
}

interface SegmentedSwitchProps<T extends string> {
  value: T;
  onChange: (value: T) => void;
  options: SegmentedOption<T>[];
}

/**
 * A 2+ segment view toggle styled like the dashboard's `TriStateSelect` (sliding
 * indigo pill). Purely presentational and controlled. Used for the worker
 * config editor's |Search|Update| switch.
 */
export function SegmentedSwitch<T extends string>({
  value,
  onChange,
  options,
}: SegmentedSwitchProps<T>) {
  const activeIdx = Math.max(
    0,
    options.findIndex((o) => o.value === value),
  );
  const width = 100 / options.length;

  return (
    <div className="relative flex w-full rounded-lg border border-border bg-surface overflow-hidden">
      <span
        aria-hidden="true"
        className="absolute inset-y-0 left-0 rounded-md bg-indigo-600/40 shadow-[0_0_8px_rgba(99,102,241,0.4)] transition-all duration-200 ease-in-out"
        style={{ width: `${width}%`, transform: `translateX(${activeIdx * 100}%)` }}
      />
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => onChange(option.value)}
          className={cn(
            "relative z-10 flex-1 py-1.5 text-xs font-medium transition-colors duration-150",
            "focus:outline-none focus-visible:ring-1 focus-visible:ring-inset focus-visible:ring-indigo-500",
            option.value === value ? "text-indigo-200" : "text-slate-500",
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}
