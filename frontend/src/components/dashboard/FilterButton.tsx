import { cn } from "@/cn";
import { cycleTriState } from "@/constants/dashboardFilters";
import { useTranslation } from "@/i18n";

interface FilterButtonProps {
  label: string;
  value: boolean | null;
  onToggle: (next: boolean | null) => void;
}

export const FilterButton = ({ label, value, onToggle }: FilterButtonProps) => {
  const { t } = useTranslation();
  const stateLabel =
    value === null
      ? t.filters.triState.all
      : value
        ? t.filters.triState.yes
        : t.filters.triState.no;

  return (
    <button
      onClick={() => onToggle(cycleTriState(value))}
      aria-pressed={value === null ? "mixed" : value}
      aria-label={`${label}: ${stateLabel}`}
      className={cn(
        "px-3 py-1 rounded text-sm font-medium transition",
        value === true && "bg-green-600 text-white",
        value === false && "bg-red-600 text-white",
        value === null && "bg-gray-700 text-gray-300",
      )}
    >
      {label}: {stateLabel}
    </button>
  );
};
