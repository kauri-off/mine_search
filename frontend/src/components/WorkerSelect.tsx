import { useQuery } from "@tanstack/react-query";
import { workerApi } from "@/api/client";
import { useTranslation } from "@/i18n";

interface WorkerSelectProps {
  value: string;
  onChange: (workerId: string) => void;
}

// Dropdown of currently-online workers. The operator must pick the worker that
// will run a scan/import; the backend fails fast if the chosen worker is offline.
export const WorkerSelect = ({ value, onChange }: WorkerSelectProps) => {
  const { t } = useTranslation();
  const { data: workers } = useQuery({
    queryKey: ["workers"],
    queryFn: workerApi.listWorkers,
    refetchInterval: 3000,
  });
  const online = (workers ?? []).filter((w) => w.online);

  return (
    <div className="space-y-1.5">
      <label className="text-xs text-slate-500">{t.workerSelect.label}</label>
      {online.length === 0 ? (
        <p className="text-xs text-slate-600">{t.workerSelect.none}</p>
      ) : (
        <select
          value={value}
          onChange={(e) => onChange(e.target.value)}
          className="w-full bg-surface border border-border rounded-lg px-3 py-2.5 text-sm text-slate-200 focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 transition-colors"
        >
          <option value="" disabled>
            {t.workerSelect.placeholder}
          </option>
          {online.map((w) => (
            <option key={w.workerId} value={w.workerId}>
              {w.name || w.workerId}
            </option>
          ))}
        </select>
      )}
    </div>
  );
};
