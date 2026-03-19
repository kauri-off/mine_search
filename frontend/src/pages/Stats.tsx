import { useQuery } from "@tanstack/react-query";
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
import { Spinner } from "@/components";
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

export const Stats = () => {
  const { t } = useTranslation();

  const { data: stats, isLoading } = useQuery({
    queryKey: ["stats"],
    queryFn: serverApi.fetchStats,
  });

  if (isLoading || !stats) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <Spinner className="w-10 h-10" />
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

  const playerData = [
    { name: t.stats.admin, value: stats.admin_players },
    { name: t.stats.other, value: stats.total_players - stats.admin_players },
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
      </section>

      {/* Charts row */}
      <section className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-10">
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

        {/* Player breakdown */}
        <div className="bg-gray-800 rounded-xl p-5">
          <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-wide mb-4">
            {t.stats.playerBreakdown}
          </h2>
          <ResponsiveContainer width="100%" height={220}>
            <PieChart>
              <Pie
                data={playerData}
                cx="50%"
                cy="50%"
                innerRadius={55}
                outerRadius={85}
                dataKey="value"
                isAnimationActive={false}
              >
                <Cell fill="#f472b6" />
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
        <div className="bg-gray-800 rounded-xl p-5">
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
    </div>
  );
};
