import { useTranslation } from "@/i18n";
import type { JoinStatus } from "@/types";
import { JOIN_STATUSES } from "@/constants/serverDetail";
import type {
  BoolFilterKey,
  FilterFieldKey,
  ServerFilterValue,
} from "@/constants/dashboardFilters";
import { TriStateSelect } from "./TriStateSelect";

interface ServerFilterFieldsProps {
  value: ServerFilterValue;
  /** Which fields to render, in order. */
  fields: FilterFieldKey[];
  onBoolChange: (key: BoolFilterKey, value: boolean | null) => void;
  onJoinStatusChange: (value: JoinStatus | null) => void;
  onQueryChange: (value: string) => void;
}

/**
 * Renders a subset of the server-property filters (free-text query, tri-state
 * booleans, and the join-status dropdown). Shared by the dashboard
 * `FilterSidebar` and the worker Search/Update config panels so the two never
 * drift on controls or labels.
 */
export const ServerFilterFields = ({
  value,
  fields,
  onBoolChange,
  onJoinStatusChange,
  onQueryChange,
}: ServerFilterFieldsProps) => {
  const { t } = useTranslation();

  return (
    <>
      {fields.map((key) => {
        if (key === "query") {
          return (
            <input
              key={key}
              type="text"
              value={value.query ?? ""}
              onChange={(e) => onQueryChange(e.target.value)}
              placeholder={t.dashboard.searchPlaceholder}
              className="w-full bg-surface border border-border rounded-lg px-3 py-2 text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
            />
          );
        }

        if (key === "join_status") {
          return (
            <div key={key} className="flex flex-col gap-1">
              <label className="text-xs text-slate-400">{t.joinStatus.label}</label>
              <select
                value={value.join_status ?? ""}
                onChange={(e) =>
                  onJoinStatusChange((e.target.value || null) as JoinStatus | null)
                }
                className="w-full bg-surface border border-border rounded-lg px-2 py-1.5 text-sm text-slate-200 focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
              >
                <option value="">{t.filters.triState.all}</option>
                {JOIN_STATUSES.map((status) => (
                  <option key={status} value={status}>
                    {t.joinStatus.values[status]}
                  </option>
                ))}
              </select>
            </div>
          );
        }

        const boolKey = key as BoolFilterKey;
        return (
          <div key={key} className="flex flex-col gap-1">
            <label className="text-xs text-slate-400">{t.filters.fields[boolKey]}</label>
            <TriStateSelect
              value={value[boolKey]}
              onChange={(next) => onBoolChange(boolKey, next)}
              labelAll={t.filters.triState.all}
              labelYes={t.filters.triState.yes}
              labelNo={t.filters.triState.no}
            />
          </div>
        );
      })}
    </>
  );
};
