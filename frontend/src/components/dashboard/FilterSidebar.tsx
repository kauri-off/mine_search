import { useTranslation } from "@/i18n";
import type {
  BoolFilterKey,
  FilterFieldKey,
  Filters,
} from "@/constants/dashboardFilters";
import type { JoinStatus } from "@/types";
import { X } from "lucide-react";
import { ServerFilterFields } from "./ServerFilterFields";

const SIDEBAR_FIELDS: FilterFieldKey[] = [
  "online",
  "licensed",
  "checked",
  "crashed",
  "requires_mods",
  "has_players",
  "has_none_players",
  "join_status",
];

interface FilterSidebarProps {
  filters: Filters;
  filtersActive: boolean;
  onBoolChange: (key: BoolFilterKey, value: boolean | null) => void;
  onJoinStatusChange: (value: JoinStatus | null) => void;
  onQueryChange: (value: string) => void;
  onReset: () => void;
  onClose?: () => void;
}

export const FilterSidebar = ({
  filters,
  filtersActive,
  onBoolChange,
  onJoinStatusChange,
  onQueryChange,
  onReset,
  onClose,
}: FilterSidebarProps) => {
  const { t } = useTranslation();

  return (
    <aside className="w-52 flex-shrink-0 bg-panel border-r border-border flex flex-col overflow-y-auto">
      <div className="px-4 py-4 border-b border-border">
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
        {/* Free-text search: IP, MOTD, version */}
        <input
          type="text"
          value={filters.query ?? ""}
          onChange={(e) => onQueryChange(e.target.value)}
          placeholder={t.dashboard.searchPlaceholder}
          className="w-full bg-surface border border-border rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
        />
      </div>

      <div className="flex-1 px-4 py-3 space-y-3">
        <ServerFilterFields
          value={filters}
          fields={SIDEBAR_FIELDS}
          onBoolChange={onBoolChange}
          onJoinStatusChange={onJoinStatusChange}
          onQueryChange={onQueryChange}
        />
      </div>

      {filtersActive && (
        <div className="px-4 py-3 border-t border-border">
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
