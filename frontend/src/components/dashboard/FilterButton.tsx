import { cn } from "@/cn";
import { cycleTriState, TRI_STATE_LABEL } from "@/constants/dashboardFilters";

interface FilterButtonProps {
  label: string;
  value: boolean | null;
  onToggle: (next: boolean | null) => void;
}

export const FilterButton = ({ label, value, onToggle }: FilterButtonProps) => (
  <button
    onClick={() => onToggle(cycleTriState(value))}
    className={cn(
      "px-3 py-1 rounded text-sm font-medium transition",
      value === true && "bg-green-600 text-white",
      value === false && "bg-red-600 text-white",
      value === null && "bg-gray-700 text-gray-300",
    )}
  >
    {label}: {TRI_STATE_LABEL[String(value)]}
  </button>
);
