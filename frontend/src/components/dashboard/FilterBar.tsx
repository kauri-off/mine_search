import type { Filters } from "@/constants/dashboardFilters";
import { useTranslation } from "@/i18n";
import { FilterButton } from "./FilterButton";

const FILTER_FIELD_KEYS = [
  "licensed",
  "checked",
  "spoofable",
  "crashed",
  "has_players",
  "online",
  "is_forge",
  "has_none_players",
] as const;

const ResetIcon = () => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    className="w-3.5 h-3.5"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth={2.5}
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
    <path d="M3 3v5h5" />
  </svg>
);

interface FilterBarProps {
  filters: Filters;
  filtersActive: boolean;
  onFilterChange: (field: keyof Filters, value: boolean | null) => void;
  onReset: () => void;
}

export const FilterBar = ({
  filters,
  filtersActive,
  onFilterChange,
  onReset,
}: FilterBarProps) => {
  const { t } = useTranslation();

  return (
    <div className="mb-6 p-4 bg-gray-800 rounded-lg flex flex-wrap gap-4 items-center">
      <span className="text-gray-400">{t.filters.label}</span>

      {FILTER_FIELD_KEYS.map((field) => (
        <FilterButton
          key={field}
          label={t.filters.fields[field]}
          value={filters[field]}
          onToggle={(next) => onFilterChange(field, next)}
        />
      ))}

      {filtersActive && (
        <button
          onClick={onReset}
          className="ml-auto px-3 py-1 rounded text-sm font-medium transition bg-gray-600 hover:bg-gray-500 text-gray-200 flex items-center gap-1.5"
          title={t.filters.reset}
        >
          <ResetIcon />
          {t.filters.reset}
        </button>
      )}
    </div>
  );
};
