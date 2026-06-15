import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Cpu, Circle } from "lucide-react";
import { workerApi, type WorkerConfigInput } from "@/api/client";
import type { WorkerInfo } from "@/gen/api_pb";
import { Spinner, ToggleButton } from "@/components";
import { useTranslation } from "@/i18n";
import { cn } from "@/cn";

function formatUptime(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m ${secs % 60}s`;
}

const Metric = ({ label, value }: { label: string; value: string }) => (
  <div className="flex flex-col">
    <span className="text-xs text-slate-500">{label}</span>
    <span className="text-sm text-slate-200 font-medium tabular-nums">{value}</span>
  </div>
);

const WorkerCard = ({ worker }: { worker: WorkerInfo }) => {
  const { t } = useTranslation();
  const qc = useQueryClient();
  const cfg = worker.config;

  // Initialised once per mount (cards are keyed by workerId), so live refetches
  // do not clobber an operator's in-progress edits.
  const [form, setForm] = useState<WorkerConfigInput>(() => ({
    threads: cfg?.threads ?? 0,
    search_module: cfg?.searchModule ?? false,
    update_module: cfg?.updateModule ?? false,
    update_with_connection: cfg?.updateWithConnection ?? false,
    only_update_spoofable: cfg?.onlyUpdateSpoofable ?? false,
    only_update_cracked: cfg?.onlyUpdateCracked ?? false,
  }));

  const mutation = useMutation({
    mutationFn: () => workerApi.updateWorkerConfig(worker.workerId, form),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["workers"] }),
  });

  const m = worker.metrics;

  return (
    <div className="p-4 bg-[#111118] border border-[#2a2a3a] rounded-xl space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 min-w-0">
          <Cpu className="w-4 h-4 text-indigo-400 flex-shrink-0" />
          <span className="text-sm font-semibold text-slate-100 truncate">
            {worker.name || worker.workerId}
          </span>
        </div>
        <span
          className={cn(
            "flex items-center gap-1.5 text-xs px-2 py-0.5 rounded-md border",
            worker.online
              ? "bg-green-900/30 text-green-400 border-green-700/40"
              : "bg-[#1a1a24] text-slate-500 border-[#2a2a3a]",
          )}
        >
          <Circle className="w-2 h-2 fill-current" />
          {worker.online ? t.workers.online : t.workers.offline}
        </span>
      </div>

      <div className="text-xs text-slate-600 truncate">
        {t.workers.version}: {worker.version || "?"} · {worker.workerId}
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-3 gap-3">
        <Metric label={t.workers.serversFound} value={m ? Number(m.serversFound).toLocaleString() : "-"} />
        <Metric label={t.workers.scanRate} value={m ? `${m.scanRate.toFixed(1)}/s` : "-"} />
        <Metric label={t.workers.uptime} value={m ? formatUptime(Number(m.uptimeSecs)) : "-"} />
        <Metric label={t.workers.activeThreads} value={m ? String(m.activeThreads) : "-"} />
        <Metric label={t.workers.searching} value={m?.searching ? "✓" : "—"} />
        <Metric label={t.workers.updating} value={m?.updating ? "✓" : "—"} />
      </div>

      {/* Config editor */}
      <div className="border-t border-[#2a2a3a] pt-3 space-y-2">
        <div className="text-xs font-medium text-slate-400">{t.workers.config}</div>

        <label className="flex items-center justify-between gap-3">
          <span className="text-sm text-slate-400">{t.workers.threads}</span>
          <input
            type="number"
            min={0}
            value={form.threads}
            onChange={(e) => setForm({ ...form, threads: Number(e.target.value) })}
            className="w-24 px-2 py-1 bg-[#1a1a24] border border-[#2a2a3a] rounded-md text-sm text-slate-200 text-right focus:outline-none focus:ring-1 focus:ring-indigo-500"
          />
        </label>

        <ToggleButton label={t.workers.searchModule} active={form.search_module} onClick={() => setForm({ ...form, search_module: !form.search_module })} />
        <ToggleButton label={t.workers.updateModule} active={form.update_module} onClick={() => setForm({ ...form, update_module: !form.update_module })} />
        <ToggleButton label={t.workers.withConnection} active={form.update_with_connection} onClick={() => setForm({ ...form, update_with_connection: !form.update_with_connection })} />
        <ToggleButton label={t.workers.onlySpoofable} active={form.only_update_spoofable} onClick={() => setForm({ ...form, only_update_spoofable: !form.only_update_spoofable })} />
        <ToggleButton label={t.workers.onlyCracked} active={form.only_update_cracked} onClick={() => setForm({ ...form, only_update_cracked: !form.only_update_cracked })} />

        <button
          onClick={() => mutation.mutate()}
          disabled={mutation.isPending || !worker.online}
          className="w-full mt-1 py-2 rounded-lg text-sm font-semibold bg-indigo-600 hover:bg-indigo-500 text-white disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
        >
          {mutation.isPending ? t.workers.saving : t.workers.save}
        </button>
        {mutation.isSuccess && <p className="text-xs text-green-400 text-center">{t.workers.saved}</p>}
        {mutation.isError && <p className="text-xs text-red-400 text-center">{t.workers.saveError}</p>}
      </div>
    </div>
  );
};

export const Workers = () => {
  const { t } = useTranslation();
  const { data: workers, isLoading } = useQuery({
    queryKey: ["workers"],
    queryFn: workerApi.listWorkers,
    refetchInterval: 3000,
  });

  return (
    <div className="flex-1 overflow-y-auto p-4 md:p-6">
      <h1 className="text-lg font-semibold text-slate-100 mb-4">{t.workers.title}</h1>

      {isLoading ? (
        <div className="flex justify-center py-16">
          <Spinner className="w-8 h-8" />
        </div>
      ) : !workers || workers.length === 0 ? (
        <p className="text-sm text-slate-500">{t.workers.empty}</p>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
          {workers.map((w) => (
            <WorkerCard key={w.workerId} worker={w} />
          ))}
        </div>
      )}
    </div>
  );
};
