import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  PieChart,
  Pie,
  Cell,
  Tooltip,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  ResponsiveContainer,
  Legend,
} from "recharts";
import {
  Server,
  Wifi,
  Unlock,
  Skull,
  Wrench,
  Users,
  ShieldCheck,
  Activity,
  Database,
  Image,
  Zap,
} from "lucide-react";
import { serverApi } from "@/api/client";
import { Spinner } from "@/components";
import { useTranslation } from "@/i18n";

const CHART_COLORS = [
  "#6366f1",
  "#f97316",
  "#34d399",
  "#f87171",
  "#a78bfa",
  "#fbbf24",
  "#38bdf8",
  "#fb7185",
  "#4ade80",
  "#e879f9",
];

function StatCard({
  label,
  value,
  icon: Icon,
  iconClass = "text-indigo-400",
}: {
  label: string;
  value: string | number;
  icon: React.FC<{ className?: string }>;
  iconClass?: string;
}) {
  return (
    <div className="bg-[#111118] border border-[#2a2a3a] rounded-xl p-5">
      <div className="flex items-center justify-between mb-3">
        <span className="text-xs text-slate-500 uppercase tracking-wide font-medium">
          {label}
        </span>
        <Icon className={`w-4 h-4 flex-shrink-0 ${iconClass}`} />
      </div>
      <span className="text-3xl font-bold text-slate-100">{value}</span>
    </div>
  );
}

function CleanButton({
  label,
  loadingLabel,
  isPending,
  onClick,
}: {
  label: string;
  loadingLabel: string;
  isPending: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      disabled={isPending}
      className="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium bg-[#1a1a24] border border-[#2a2a3a] text-slate-400 hover:text-slate-200 hover:border-[#3a3a4a] disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
    >
      {isPending ? (
        <Spinner className="w-3.5 h-3.5 text-indigo-400" />
      ) : (
        <Wrench className="w-3.5 h-3.5" />
      )}
      {isPending ? loadingLabel : label}
    </button>
  );
}

function formatMb(mb: number): string {
  return `${mb.toFixed(2)} MB`;
}

const tooltipStyle = {
  contentStyle: {
    backgroundColor: "#111118",
    border: "1px solid #2a2a3a",
    borderRadius: "8px",
  },
  itemStyle: { color: "#94a3b8" },
  labelStyle: { color: "#64748b" },
};

export const Stats = () => {
  const { t } = useTranslation();
  const queryClient = useQueryClient();

  const { data: stats, isLoading } = useQuery({
    queryKey: ["stats"],
    queryFn: serverApi.fetchStats,
    staleTime: Infinity,
  });

  const [snapshotResult, setSnapshotResult] = useState<number | null>(null);
  const [faviconResult, setFaviconResult] = useState<number | null>(null);

  const cleanSnapshotsMutation = useMutation({
    mutationFn: serverApi.cleanSnapshots,
    onSuccess: (data) => {
      setSnapshotResult(data.deleted);
      queryClient.invalidateQueries({ queryKey: ["stats"] });
    },
  });

  const cleanFaviconsMutation = useMutation({
    mutationFn: serverApi.cleanFavicons,
    onSuccess: (data) => {
      setFaviconResult(data.deleted);
      queryClient.invalidateQueries({ queryKey: ["stats"] });
    },
  });

  if (isLoading || !stats) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <Spinner className="w-8 h-8 text-indigo-500" />
      </div>
    );
  }

  const licenseData = [
    { name: t.stats.licensed, value: stats.total_servers - stats.cracked_servers },
    { name: t.stats.cracked, value: stats.cracked_servers },
  ];

  const onlineData = [
    { name: t.stats.online, value: stats.online_servers },
    { name: t.stats.offline, value: stats.total_servers - stats.online_servers },
  ];

  const versionData = stats.version_distribution.map((v) => ({
    name: v.version || "(unknown)",
    count: v.count,
  }));

  const avgPingDisplay =
    stats.avg_ping != null ? `${Math.round(stats.avg_ping)} ms` : "N/A";

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="px-3 py-3 sm:px-6 sm:py-5 max-w-screen-2xl mx-auto">
        <h1 className="text-xl font-bold text-slate-100 mb-6">{t.stats.title}</h1>

        {/* Stat cards */}
        <section className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-3 mb-8">
          <StatCard label={t.stats.totalServers} value={stats.total_servers} icon={Server} iconClass="text-indigo-400" />
          <StatCard label={t.stats.online} value={stats.online_servers} icon={Wifi} iconClass="text-green-400" />
          <StatCard label={t.stats.cracked} value={stats.cracked_servers} icon={Unlock} iconClass="text-orange-400" />
          <StatCard label={t.stats.crashed} value={stats.crashed_servers} icon={Skull} iconClass="text-red-400" />
          <StatCard label={t.stats.forge} value={stats.forge_servers} icon={Wrench} iconClass="text-purple-400" />
          <StatCard label={t.stats.spoofable} value={stats.spoofable_servers} icon={ShieldCheck} iconClass="text-yellow-400" />
          <StatCard label={t.stats.totalPlayers} value={stats.total_players} icon={Users} iconClass="text-sky-400" />
          <StatCard label={t.stats.adminPlayers} value={stats.admin_players} icon={ShieldCheck} iconClass="text-pink-400" />
          <StatCard label={t.stats.avgPing} value={avgPingDisplay} icon={Activity} iconClass="text-emerald-400" />
          <StatCard label={t.stats.dbSize} value={formatMb(stats.db_size_mb)} icon={Database} iconClass="text-slate-400" />
          <StatCard label={t.stats.faviconSize} value={formatMb(stats.favicon_size_mb)} icon={Image} iconClass="text-slate-400" />
        </section>

        {/* Charts */}
        <section className="grid grid-cols-1 lg:grid-cols-2 gap-4 mb-6">
          <div className="bg-[#111118] border border-[#2a2a3a] rounded-xl p-5">
            <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-4">
              {t.stats.licensedVsCracked}
            </h2>
            <ResponsiveContainer width="100%" height={200}>
              <PieChart>
                <Pie data={licenseData} cx="50%" cy="50%" innerRadius={50} outerRadius={75} dataKey="value" isAnimationActive={false}>
                  <Cell fill="#6366f1" />
                  <Cell fill="#f97316" />
                </Pie>
                <Tooltip {...tooltipStyle} />
                <Legend formatter={(v) => <span style={{ color: "#64748b", fontSize: 12 }}>{v}</span>} />
              </PieChart>
            </ResponsiveContainer>
          </div>

          <div className="bg-[#111118] border border-[#2a2a3a] rounded-xl p-5">
            <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-4">
              {t.stats.onlineVsOffline}
            </h2>
            <ResponsiveContainer width="100%" height={200}>
              <PieChart>
                <Pie data={onlineData} cx="50%" cy="50%" innerRadius={50} outerRadius={75} dataKey="value" isAnimationActive={false}>
                  <Cell fill="#22c55e" />
                  <Cell fill="#2a2a3a" />
                </Pie>
                <Tooltip {...tooltipStyle} />
                <Legend formatter={(v) => <span style={{ color: "#64748b", fontSize: 12 }}>{v}</span>} />
              </PieChart>
            </ResponsiveContainer>
          </div>
        </section>

        {/* Versions bar chart */}
        {versionData.length > 0 && (
          <div className="bg-[#111118] border border-[#2a2a3a] rounded-xl p-5 mb-6">
            <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-4">
              {t.stats.topVersions}
            </h2>
            <ResponsiveContainer width="100%" height={Math.max(200, versionData.length * 28)}>
              <BarChart data={versionData} layout="vertical" margin={{ top: 0, right: 24, bottom: 0, left: 8 }}>
                <XAxis type="number" tick={{ fill: "#64748b", fontSize: 11 }} axisLine={false} tickLine={false} />
                <YAxis type="category" dataKey="name" width={140} tick={{ fill: "#94a3b8", fontSize: 11 }} axisLine={false} tickLine={false} />
                <Tooltip {...tooltipStyle} cursor={{ fill: "rgba(255,255,255,0.02)" }} />
                <Bar dataKey="count" radius={[0, 4, 4, 0]} isAnimationActive={false}>
                  {versionData.map((_, index) => (
                    <Cell key={index} fill={CHART_COLORS[index % CHART_COLORS.length]} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        )}

        {/* Maintenance */}
        <div className="bg-[#111118] border border-[#2a2a3a] rounded-xl p-5">
          <h2 className="text-xs font-semibold text-slate-500 uppercase tracking-wide mb-4">
            <span className="flex items-center gap-2">
              <Zap className="w-3.5 h-3.5" />
              {t.stats.maintenance}
            </span>
          </h2>
          <div className="flex flex-wrap gap-4">
            <div className="flex items-center gap-3">
              <CleanButton
                label={t.stats.cleanSnapshots}
                loadingLabel={t.stats.cleaning}
                isPending={cleanSnapshotsMutation.isPending}
                onClick={() => {
                  setSnapshotResult(null);
                  cleanSnapshotsMutation.mutate();
                }}
              />
              {snapshotResult !== null && (
                <span className="text-sm text-slate-500">
                  {t.stats.cleanedRows(snapshotResult)}
                </span>
              )}
            </div>

            <div className="flex items-center gap-3">
              <CleanButton
                label={t.stats.cleanFavicons}
                loadingLabel={t.stats.cleaning}
                isPending={cleanFaviconsMutation.isPending}
                onClick={() => {
                  setFaviconResult(null);
                  cleanFaviconsMutation.mutate();
                }}
              />
              {faviconResult !== null && (
                <span className="text-sm text-slate-500">
                  {t.stats.cleanedRows(faviconResult)}
                </span>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
