import { useRef, useState, type ReactNode } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { formatDistanceToNow } from "date-fns";
import { Cpu, Circle } from "lucide-react";
import { workerApi, protoToFilterValue, type WorkerConfigInput } from "@/api/client";
import type { WorkerInfo } from "@/gen/api_pb";
import { Spinner, ToggleButton } from "@/components";
import { SegmentedSwitch } from "@/components/SegmentedSwitch";
import { ServerFilterFields } from "@/components/dashboard/ServerFilterFields";
import {
  SEARCH_FILTER_FIELDS,
  UPDATE_FILTER_FIELDS,
  type BoolFilterKey,
  type ServerFilterValue,
} from "@/constants/dashboardFilters";
import type { JoinStatus } from "@/types";
import { useTranslation } from "@/i18n";
import { cn } from "@/cn";

function formatUptime(secs: number, u: { h: string; m: string; s: string }): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}${u.h} ${m}${u.m}`;
  return `${m}${u.m} ${secs % 60}${u.s}`;
}

const Metric = ({
  label,
  value,
  accent,
}: {
  label: string;
  value: string;
  accent?: string;
}) => (
  <div className="flex flex-col">
    <span className="text-xs text-slate-500">{label}</span>
    <span className={cn("text-sm font-medium tabular-nums", accent ?? "text-slate-200")}>
      {value}
    </span>
  </div>
);

// Section title with a live state chip on the right (mirrors the online badge).
const SectionHeader = ({ label, chip }: { label: string; chip: ReactNode }) => (
  <div className="flex items-center justify-between mb-2">
    <span className="text-xs font-medium text-slate-400">{label}</span>
    {chip}
  </div>
);

// A module's on/off state, shown as a pill next to its section title. When active
// the dot fills and pulses so a running module reads at a glance.
const StateChip = ({
  active,
  label,
  tone,
}: {
  active: boolean;
  label: string;
  tone: "green" | "indigo";
}) => (
  <span
    className={cn(
      "flex items-center gap-1.5 text-[11px] px-2 py-0.5 rounded-md border",
      active
        ? tone === "green"
          ? "bg-green-900/30 text-green-400 border-green-700/40"
          : "bg-indigo-900/30 text-indigo-300 border-indigo-700/40"
        : "bg-surface text-slate-500 border-border",
    )}
  >
    <Circle className={cn("w-2 h-2", active && "fill-current animate-pulse")} />
    {label}
  </span>
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
    update_interval_secs: cfg?.updateIntervalSecs ?? 600,
    update_concurrency: cfg?.updateConcurrency ?? 50,
    update_filter: protoToFilterValue(cfg?.updateFilter),
    search_filter: protoToFilterValue(cfg?.searchFilter),
  }));

  // Which module's config panel is shown (view toggle only; both can run).
  const [module, setModule] = useState<"search" | "update">("search");

  // Merge a single field into one of the two filter objects. Split by handler so
  // the tri-state/dropdown/text callbacks stay strongly typed.
  const patchSearch = (patch: Partial<ServerFilterValue>) =>
    setForm((p) => ({ ...p, search_filter: { ...p.search_filter, ...patch } }));
  const patchUpdate = (patch: Partial<ServerFilterValue>) =>
    setForm((p) => ({ ...p, update_filter: { ...p.update_filter, ...patch } }));

  const mutation = useMutation({
    mutationFn: () => workerApi.updateWorkerConfig(worker.workerId, form),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["workers"] }),
  });

  const m = worker.metrics;

  // Search productivity: discoveries per IP probed. Typically a tiny fraction on
  // random-IP scanning, so keep extra precision below 1%.
  const ipsScanned = m ? Number(m.ipsScanned) : 0;
  const serversFound = m ? Number(m.serversFound) : 0;
  const hitRate =
    m && ipsScanned > 0
      ? (() => {
          const pct = (serversFound / ipsScanned) * 100;
          return `${pct >= 1 ? pct.toFixed(1) : pct.toPrecision(2)}%`;
        })()
      : "-";

  // Update-cycle progress → fill bar + percent.
  const updateDone = m ? Number(m.updateDone) : 0;
  const updateTotal = m ? Number(m.updateTotal) : 0;
  const updatePct = updateTotal > 0 ? (updateDone / updateTotal) * 100 : 0;

  // Double-click the header to rename. The draft lives in the uncontrolled input;
  // `cancelRef` lets Escape blur without committing the edit.
  const [editingName, setEditingName] = useState(false);
  const cancelRename = useRef(false);
  const rename = useMutation({
    mutationFn: (name: string) => workerApi.setWorkerName(worker.workerId, name),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["workers"] }),
  });

  const commitRename = (value: string) => {
    setEditingName(false);
    if (cancelRename.current) {
      cancelRename.current = false;
      return;
    }
    const next = value.trim();
    if (next !== (worker.name ?? "")) rename.mutate(next);
  };

  const control = useMutation({
    mutationFn: (fn: (id: string) => Promise<unknown>) => fn(worker.workerId),
    onSettled: () => qc.invalidateQueries({ queryKey: ["workers"] }),
  });
  const busy = control.isPending || !worker.online;

  return (
    <div className="p-4 bg-panel border border-border rounded-xl space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 min-w-0">
          <Cpu className="w-4 h-4 text-indigo-400 flex-shrink-0" />
          {editingName ? (
            <input
              autoFocus
              defaultValue={worker.name ?? ""}
              placeholder={worker.workerId}
              onKeyDown={(e) => {
                if (e.key === "Enter") e.currentTarget.blur();
                else if (e.key === "Escape") {
                  cancelRename.current = true;
                  e.currentTarget.blur();
                }
              }}
              onBlur={(e) => commitRename(e.target.value)}
              className="min-w-0 flex-1 px-2 py-0.5 bg-surface border border-indigo-500 rounded-md text-sm font-semibold text-slate-100 focus:outline-none focus:ring-1 focus:ring-indigo-500"
            />
          ) : (
            <span
              onDoubleClick={() => setEditingName(true)}
              title={t.workers.renameHint}
              className="text-sm font-semibold text-slate-100 truncate cursor-text"
            >
              {worker.name || worker.workerId}
            </span>
          )}
        </div>
        <span
          className={cn(
            "flex items-center gap-1.5 text-xs px-2 py-0.5 rounded-md border",
            worker.online
              ? "bg-green-900/30 text-green-400 border-green-700/40"
              : "bg-surface text-slate-500 border-border",
          )}
        >
          <Circle className="w-2 h-2 fill-current" />
          {worker.online ? t.workers.online : t.workers.offline}
        </span>
      </div>

      {rename.isError && <p className="text-xs text-red-400">{t.workers.renameError}</p>}

      {/* Tier 1 — worker-level facts (whole process, module-independent) */}
      <div className="grid grid-cols-3 gap-3 border-b border-border pb-3">
        <Metric label={t.workers.uptime} value={m ? formatUptime(Number(m.uptimeSecs), t.workers.uptimeUnits) : "-"} />
        <Metric label={t.workers.activeThreads} value={m ? String(m.activeThreads) : "-"} />
        <Metric label={t.workers.version} value={worker.version ? `v${worker.version}` : "-"} />
      </div>

      {/* Tier 2 — Search module */}
      <div>
        <SectionHeader
          label={t.workers.searchMetrics}
          chip={
            <StateChip
              active={!!m?.searching}
              tone="green"
              label={m?.searching ? t.workers.scanning : t.workers.paused}
            />
          }
        />
        <div className="grid grid-cols-2 gap-3">
          <Metric label={t.workers.serversFound} value={m ? serversFound.toLocaleString() : "-"} />
          <Metric label={t.workers.ipsScanned} value={m ? ipsScanned.toLocaleString() : "-"} />
          <Metric label={t.workers.scanRate} value={m ? `${m.scanRate.toFixed(1)}/s` : "-"} />
          <Metric label={t.workers.hitRate} value={hitRate} accent="text-slate-100" />
        </div>
      </div>

      {/* Tier 3 — Update module */}
      <div>
        <SectionHeader
          label={t.workers.updateMetrics}
          chip={
            <StateChip
              active={!!m?.updating}
              tone="indigo"
              label={m?.updating ? t.workers.updating : t.workers.idle}
            />
          }
        />
        <div className="mb-3">
          <div className="flex items-center justify-between text-xs mb-1">
            <span className="text-slate-500">{t.workers.updateProgress}</span>
            <span className="text-slate-300 tabular-nums">
              {m ? `${updateDone.toLocaleString()} / ${updateTotal.toLocaleString()}` : "-"}
              {m && updateTotal > 0 ? `  ·  ${updatePct.toFixed(0)}%` : ""}
            </span>
          </div>
          <div className="h-1.5 rounded-full bg-surface overflow-hidden">
            <div
              className="h-full bg-indigo-500 rounded-full transition-all"
              style={{ width: `${updatePct}%` }}
            />
          </div>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <Metric label={t.workers.updateRate} value={m ? `${m.updateRate.toFixed(1)}/s` : "-"} />
          <Metric
            label={t.workers.lastUpdate}
            value={
              m && Number(m.lastUpdateUnix) > 0
                ? formatDistanceToNow(new Date(Number(m.lastUpdateUnix) * 1000), {
                    addSuffix: true,
                    locale: t.dateFnsLocale,
                  })
                : t.workers.never
            }
          />
        </div>
      </div>

      {/* Controls */}
      <div className="border-t border-border pt-3 space-y-2">
        <div className="text-xs font-medium text-slate-400">{t.workers.controls}</div>
        <div className="grid grid-cols-2 gap-2">
          <button
            onClick={() =>
              control.mutate(m?.searching ? workerApi.pauseSearch : workerApi.resumeSearch)
            }
            disabled={busy}
            className="py-2 rounded-lg text-sm font-medium bg-surface hover:bg-border text-slate-300 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {m?.searching ? t.workers.pauseSearch : t.workers.resumeSearch}
          </button>
          <button
            onClick={() => control.mutate(workerApi.triggerUpdate)}
            disabled={busy || m?.updating}
            className="py-2 rounded-lg text-sm font-medium bg-indigo-600/20 border border-indigo-600/40 text-indigo-300 hover:bg-indigo-600/30 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {t.workers.triggerUpdate}
          </button>
          <button
            onClick={() => control.mutate(workerApi.abortUpdate)}
            disabled={control.isPending || !worker.online || !m?.updating}
            className="col-span-2 py-2 rounded-lg text-sm font-medium bg-red-600/20 border border-red-600/40 text-red-300 hover:bg-red-600/30 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {t.workers.abortUpdate}
          </button>
        </div>
        {control.isError && <p className="text-xs text-red-400 text-center">{t.workers.controlError}</p>}
      </div>

      {/* Config editor */}
      <div className="border-t border-border pt-3 space-y-2">
        <div className="text-xs font-medium text-slate-400">{t.workers.config}</div>

        <SegmentedSwitch
          value={module}
          onChange={setModule}
          options={[
            { value: "search", label: t.workers.searchTab },
            { value: "update", label: t.workers.updateTab },
          ]}
        />

        {module === "search" ? (
          <div className="space-y-2">
            <ToggleButton label={t.workers.searchModule} active={form.search_module} onClick={() => setForm({ ...form, search_module: !form.search_module })} />

            <label className="flex items-center justify-between gap-3">
              <span className="text-sm text-slate-400">{t.workers.threads}</span>
              <input
                type="number"
                min={0}
                value={form.threads}
                onChange={(e) => setForm({ ...form, threads: Number(e.target.value) })}
                className="w-24 px-2 py-1 bg-surface border border-border rounded-md text-sm text-slate-200 text-right focus:outline-none focus:ring-1 focus:ring-indigo-500"
              />
            </label>

            <div className="pt-1 space-y-2">
              <div className="text-xs font-medium text-slate-500">{t.workers.filters}</div>
              <ServerFilterFields
                value={form.search_filter}
                fields={SEARCH_FILTER_FIELDS}
                onBoolChange={(key: BoolFilterKey, val) => patchSearch({ [key]: val } as Partial<ServerFilterValue>)}
                onJoinStatusChange={(val: JoinStatus | null) => patchSearch({ join_status: val })}
                onQueryChange={(val) => patchSearch({ query: val || null })}
              />
            </div>
          </div>
        ) : (
          <div className="space-y-2">
            <ToggleButton label={t.workers.updateModule} active={form.update_module} onClick={() => setForm({ ...form, update_module: !form.update_module })} />
            <ToggleButton label={t.workers.withConnection} active={form.update_with_connection} onClick={() => setForm({ ...form, update_with_connection: !form.update_with_connection })} />

            <label className="flex items-center justify-between gap-3">
              <span className="text-sm text-slate-400">{t.workers.updateInterval}</span>
              <input
                type="number"
                min={0}
                value={form.update_interval_secs}
                onChange={(e) => setForm({ ...form, update_interval_secs: Number(e.target.value) })}
                className="w-24 px-2 py-1 bg-surface border border-border rounded-md text-sm text-slate-200 text-right focus:outline-none focus:ring-1 focus:ring-indigo-500"
              />
            </label>

            <label className="flex items-center justify-between gap-3">
              <span className="text-sm text-slate-400">{t.workers.updateConcurrency}</span>
              <input
                type="number"
                min={0}
                value={form.update_concurrency}
                onChange={(e) => setForm({ ...form, update_concurrency: Number(e.target.value) })}
                className="w-24 px-2 py-1 bg-surface border border-border rounded-md text-sm text-slate-200 text-right focus:outline-none focus:ring-1 focus:ring-indigo-500"
              />
            </label>

            <div className="pt-1 space-y-2">
              <div className="text-xs font-medium text-slate-500">{t.workers.filters}</div>
              <ServerFilterFields
                value={form.update_filter}
                fields={UPDATE_FILTER_FIELDS}
                onBoolChange={(key: BoolFilterKey, val) => patchUpdate({ [key]: val } as Partial<ServerFilterValue>)}
                onJoinStatusChange={(val: JoinStatus | null) => patchUpdate({ join_status: val })}
                onQueryChange={(val) => patchUpdate({ query: val || null })}
              />
            </div>
          </div>
        )}

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
