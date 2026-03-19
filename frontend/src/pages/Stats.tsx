import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";
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
import { serverApi } from "@/api/client";
import { Spinner as PageSpinner } from "@/components";
import { useTranslation } from "@/i18n";

const COLORS = [
  "#60a5fa",
  "#f97316",
  "#34d399",
  "#f87171",
  "#a78bfa",
  "#fbbf24",
];

function StatCard({
  label,
  value,
  color = "text-white",
}: {
  label: string;
  value: string | number;
  color?: string;
}) {
  return (
    <div className="bg-gray-800 rounded-xl p-5 flex flex-col gap-1">
      <span className="text-xs text-gray-400 uppercase tracking-wide">
        {label}
      </span>
      <span className={`text-2xl font-bold ${color}`}>{value}</span>
    </div>
  );
}

function Spinner() {
  return (
    <svg
      className="animate-spin h-4 w-4"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
      />
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
      />
    </svg>
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
    <div className="relative inline-flex">
      {isPending && (
        <span className="absolute inset-0 rounded-lg animate-ping bg-blue-500 opacity-20" />
      )}
      <button
        onClick={onClick}
        disabled={isPending}
        className="relative flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 disabled:cursor-not-allowed rounded-lg text-sm font-medium transition-colors"
      >
        {isPending ? (
          <>
            <Spinner />
            {loadingLabel}
          </>
        ) : (
          label
        )}
      </button>
    </div>
  );
}

function formatMb(mb: number): string {
  return `${mb.toFixed(2)} MB`;
}

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
      <div className="min-h-screen flex items-center justify-center">
        <PageSpinner className="w-10 h-10" />
      </div>
    );
  }

  const licenseData = [
    {
      name: t.stats.licensed,
      value: stats.total_servers - stats.cracked_servers,
    },
    { name: t.stats.cracked, value: stats.cracked_servers },
  ];

  const onlineData = [
    { name: t.stats.online, value: stats.online_servers },
    {
      name: t.stats.offline,
      value: stats.total_servers - stats.online_servers,
    },
  ];

  const versionData = stats.version_distribution.map((v) => ({
    name: v.version || "(unknown)",
    count: v.count,
  }));

  const avgPingDisplay =
    stats.avg_ping != null ? `${Math.round(stats.avg_ping)} ms` : "N/A";

  return (
    <div className="p-6 max-w-screen-2xl mx-auto text-white">
      <header className="mb-8 flex items-center gap-4">
        <Link
          to="/"
          className="text-sm text-gray-400 hover:text-gray-200 transition-colors"
        >
          ← {t.stats.back}
        </Link>
        <h1 className="text-3xl font-bold">{t.stats.title}</h1>
      </header>

      {/* Stat cards */}
      <section className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-4 mb-10">
        <StatCard
          label={t.stats.totalServers}
          value={stats.total_servers}
          color="text-blue-400"
        />
        <StatCard
          label={t.stats.online}
          value={stats.online_servers}
          color="text-green-400"
        />
        <StatCard
          label={t.stats.cracked}
          value={stats.cracked_servers}
          color="text-orange-400"
        />
        <StatCard
          label={t.stats.crashed}
          value={stats.crashed_servers}
          color="text-red-400"
        />
        <StatCard
          label={t.stats.forge}
          value={stats.forge_servers}
          color="text-purple-400"
        />
        <StatCard
          label={t.stats.spoofable}
          value={stats.spoofable_servers}
          color="text-yellow-400"
        />
        <StatCard
          label={t.stats.totalPlayers}
          value={stats.total_players}
          color="text-cyan-400"
        />
        <StatCard
          label={t.stats.adminPlayers}
          value={stats.admin_players}
          color="text-pink-400"
        />
        <StatCard
          label={t.stats.avgPing}
          value={avgPingDisplay}
          color="text-indigo-400"
        />
        <StatCard
          label={t.stats.dbSize}
          value={formatMb(stats.db_size_mb)}
          color="text-teal-400"
        />
        <StatCard
          label={t.stats.faviconSize}
          value={formatMb(stats.favicon_size_mb)}
          color="text-lime-400"
        />
      </section>

      {/* Charts row */}
      <section className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-10">
        {/* Licensed vs Cracked */}
        <div className="bg-gray-800 rounded-xl p-5">
          <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-4">
            {t.stats.licensedVsCracked}
          </h2>
          <ResponsiveContainer width="100%" height={220}>
            <PieChart>
              <Pie
                data={licenseData}
                cx="50%"
                cy="50%"
                innerRadius={55}
                outerRadius={85}
                dataKey="value"
                isAnimationActive={false}
              >
                <Cell fill="#60a5fa" />
                <Cell fill="#f97316" />
              </Pie>
              <Tooltip
                contentStyle={{
                  backgroundColor: "#1f2937",
                  border: "none",
                  borderRadius: 8,
                }}
                itemStyle={{ color: "#e5e7eb" }}
              />
              <Legend
                formatter={(value) => (
                  <span style={{ color: "#9ca3af", fontSize: 12 }}>
                    {value}
                  </span>
                )}
              />
            </PieChart>
          </ResponsiveContainer>
        </div>

        {/* Online vs Offline */}
        <div className="bg-gray-800 rounded-xl p-5">
          <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-4">
            {t.stats.onlineVsOffline}
          </h2>
          <ResponsiveContainer width="100%" height={220}>
            <PieChart>
              <Pie
                data={onlineData}
                cx="50%"
                cy="50%"
                innerRadius={55}
                outerRadius={85}
                dataKey="value"
                isAnimationActive={false}
              >
                <Cell fill="#34d399" />
                <Cell fill="#6b7280" />
              </Pie>
              <Tooltip
                contentStyle={{
                  backgroundColor: "#1f2937",
                  border: "none",
                  borderRadius: 8,
                }}
                itemStyle={{ color: "#e5e7eb" }}
              />
              <Legend
                formatter={(value) => (
                  <span style={{ color: "#9ca3af", fontSize: 12 }}>
                    {value}
                  </span>
                )}
              />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </section>

      {/* Top versions bar chart */}
      {versionData.length > 0 && (
        <div className="bg-gray-800 rounded-xl p-5 mb-10">
          <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-4">
            {t.stats.topVersions}
          </h2>
          <ResponsiveContainer width="100%" height={320}>
            <BarChart
              data={versionData}
              layout="vertical"
              margin={{ top: 0, right: 24, bottom: 0, left: 8 }}
            >
              <XAxis
                type="number"
                tick={{ fill: "#9ca3af", fontSize: 12 }}
                axisLine={false}
                tickLine={false}
              />
              <YAxis
                type="category"
                dataKey="name"
                width={140}
                tick={{ fill: "#d1d5db", fontSize: 12 }}
                axisLine={false}
                tickLine={false}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: "#1f2937",
                  border: "none",
                  borderRadius: 8,
                }}
                itemStyle={{ color: "#e5e7eb" }}
                cursor={{ fill: "rgba(255,255,255,0.04)" }}
              />
              <Bar
                dataKey="count"
                radius={[0, 4, 4, 0]}
                isAnimationActive={false}
              >
                {versionData.map((_, index) => (
                  <Cell key={index} fill={COLORS[index % COLORS.length]} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}

      {/* Maintenance */}
      <div className="bg-gray-800 rounded-xl p-5">
        <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-4">
          {t.stats.maintenance}
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
              <span className="text-sm text-gray-400">
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
              <span className="text-sm text-gray-400">
                {t.stats.cleanedRows(faviconResult)}
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
