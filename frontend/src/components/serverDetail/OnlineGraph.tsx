import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { useTranslation } from "@/i18n";

interface ChartDataPoint {
  time: string;
  online: number;
  formattedTime: string;
}

interface OnlineGraphProps {
  data: ChartDataPoint[] | undefined;
  isLoading?: boolean;
}

export const OnlineGraph = ({ data, isLoading }: OnlineGraphProps) => {
  const { t } = useTranslation();

  return (
    <div
      className="bg-panel border border-border rounded-xl p-5 h-80 select-none"
      onMouseDown={(e) => e.preventDefault()}
    >
      <h3 className="text-sm font-semibold text-slate-300 mb-4">{t.onlineGraph.title}</h3>
      {isLoading || data === undefined ? (
        <div className="h-[calc(100%-2.25rem)] animate-pulse flex flex-col justify-between pb-1">
          <div className="flex-1 bg-surface rounded-lg" />
          <div className="flex justify-between mt-3 px-8">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="h-2.5 w-10 bg-surface rounded" />
            ))}
          </div>
        </div>
      ) : (
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={data} style={{ userSelect: "none" }}>
          <defs>
            <linearGradient id="colorOnline" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="var(--color-accent)" stopOpacity={0.4} />
              <stop offset="95%" stopColor="var(--color-accent)" stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--color-surface)" />
          <XAxis dataKey="formattedTime" stroke="var(--color-border-hover)" tick={{ fill: "var(--color-muted)", fontSize: 11 }} />
          <YAxis stroke="var(--color-border-hover)" tick={{ fill: "var(--color-muted)", fontSize: 11 }} allowDecimals={false} />
          <Tooltip
            contentStyle={{
              backgroundColor: "var(--color-panel)",
              border: "1px solid var(--color-border)",
              borderRadius: "8px",
            }}
            itemStyle={{ color: "#a5b4fc" }}
            labelStyle={{ color: "var(--color-muted)" }}
          />
          <Area
            type="monotone"
            dataKey="online"
            name={t.onlineGraph.online}
            stroke="var(--color-accent)"
            strokeWidth={1.5}
            fillOpacity={1}
            fill="url(#colorOnline)"
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
      )}
    </div>
  );
};
