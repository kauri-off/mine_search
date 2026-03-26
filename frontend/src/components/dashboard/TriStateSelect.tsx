import { cn } from "@/cn";

interface TriStateSelectProps {
  value: boolean | null;
  onChange: (next: boolean | null) => void;
  labelAll: string;
  labelYes: string;
  labelNo: string;
}

function valueToIndex(v: boolean | null): 0 | 1 | 2 {
  if (v === null) return 0;
  if (v === true) return 1;
  return 2;
}

const INDEX_TO_VALUE: Array<boolean | null> = [null, true, false];

const PILL_TRANSLATE = [
  "translate-x-0",
  "translate-x-[100%]",
  "translate-x-[200%]",
] as const;

const PILL_COLOR = [
  "bg-[#2a2a3a]",
  "bg-indigo-600/40 shadow-[0_0_8px_rgba(99,102,241,0.4)]",
  "bg-red-600/40 shadow-[0_0_8px_rgba(220,38,38,0.35)]",
] as const;

const BORDER_COLOR = [
  "border-[#2a2a3a]",
  "border-indigo-600/50",
  "border-red-600/50",
] as const;

const ACTIVE_TEXT = [
  "text-slate-200",
  "text-indigo-200",
  "text-red-200",
] as const;

const INACTIVE_TEXT = "text-slate-500";

export function TriStateSelect({
  value,
  onChange,
  labelAll,
  labelYes,
  labelNo,
}: TriStateSelectProps) {
  const activeIdx = valueToIndex(value);
  const labels = [labelAll, labelYes, labelNo];

  return (
    <div
      className={cn(
        "relative flex w-full rounded-lg border bg-[#1a1a24] overflow-hidden transition-colors duration-150",
        BORDER_COLOR[activeIdx],
      )}
    >
      <span
        aria-hidden="true"
        className={cn(
          "absolute inset-y-0 left-0 w-1/3 rounded-md",
          "transition-all duration-200 ease-in-out",
          PILL_TRANSLATE[activeIdx],
          PILL_COLOR[activeIdx],
        )}
      />
      {labels.map((label, idx) => (
        <button
          key={idx}
          type="button"
          onClick={() => onChange(INDEX_TO_VALUE[idx])}
          className={cn(
            "relative z-10 flex-1 py-1.5 text-xs font-medium transition-colors duration-150",
            "focus:outline-none focus-visible:ring-1 focus-visible:ring-inset focus-visible:ring-indigo-500",
            idx === activeIdx ? ACTIVE_TEXT[activeIdx] : INACTIVE_TEXT,
          )}
        >
          {label}
        </button>
      ))}
    </div>
  );
}
