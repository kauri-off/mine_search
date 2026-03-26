import { useTranslation } from "@/i18n";
import type { Filters } from "@/constants/dashboardFilters";
import { X } from "lucide-react";
import { TriStateSelect } from "./TriStateSelect";

type BoolFilterKey = keyof Omit<Filters, "limit" | "ip_contains">;

const BOOL_FILTERS: BoolFilterKey[] = [
  "online",
  "licensed",
  "checked",
  "spoofable",
  "crashed",
  "is_forge",
  "has_players",
  "has_none_players",
];

interface FilterSidebarProps {
  filters: Filters;
  filtersActive: boolean;
  onBoolChange: (key: BoolFilterKey, value: boolean | null) => void;
  onIpChange: (value: string) => void;
  onReset: () => void;
  onClose?: () => void;
}

export const FilterSidebar = ({
  filters,
  filtersActive,
  onBoolChange,
  onIpChange,
  onReset,
  onClose,
}: FilterSidebarProps) => {
  const { t } = useTranslation();

  return (
    <aside className="w-52 flex-shrink-0 bg-[#111118] border-r border-[#2a2a3a] flex flex-col overflow-y-auto">
      <div className="px-4 py-4 border-b border-[#2a2a3a]">
        <div className="flex items-center justify-between mb-3">
          <p className="text-xs font-semibold text-slate-400 uppercase tracking-wider">
            {t.filters.label}
          </p>
          {onClose && (
            <button onClick={onClose} className="p-1 text-slate-500 hover:text-slate-300 transition-colors">
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
        {/* IP search */}
        <input
          type="text"
          value={filters.ip_contains ?? ""}
          onChange={(e) => onIpChange(e.target.value)}
          placeholder={t.dashboard.searchPlaceholder}
          className="w-full bg-[#1a1a24] border border-[#2a2a3a] rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
        />
      </div>

      <div className="flex-1 px-4 py-3 space-y-3">
        {BOOL_FILTERS.map((key) => {
          const current = filters[key];
          return (
            <div key={key} className="flex flex-col gap-1">
              <label className="text-xs text-slate-400">
                {t.filters.fields[key]}
              </label>
              <TriStateSelect
                value={current}
                onChange={(next) => onBoolChange(key, next)}
                labelAll={t.filters.triState.all}
                labelYes={t.filters.triState.yes}
                labelNo={t.filters.triState.no}
              />
            </div>
          );
        })}
      </div>

      {filtersActive && (
        <div className="px-4 py-3 border-t border-[#2a2a3a]">
          <button
            onClick={onReset}
            className="w-full text-xs text-slate-400 hover:text-red-400 transition-colors py-1"
          >
            {t.filters.reset}
          </button>
        </div>
      )}
    </aside>
  );
};
