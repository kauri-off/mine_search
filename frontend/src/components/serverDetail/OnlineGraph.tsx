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
    <div className="bg-[#111118] border border-[#2a2a3a] rounded-xl p-5 h-80">
      <h3 className="text-sm font-semibold text-slate-300 mb-4">{t.onlineGraph.title}</h3>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart data={isLoading || data === undefined ? [] : data}>
          <defs>
            <linearGradient id="colorOnline" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#6366f1" stopOpacity={0.4} />
              <stop offset="95%" stopColor="#6366f1" stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="#1a1a24" />
          <XAxis dataKey="formattedTime" stroke="#3a3a4a" tick={{ fill: "#64748b", fontSize: 11 }} />
          <YAxis stroke="#3a3a4a" tick={{ fill: "#64748b", fontSize: 11 }} allowDecimals={false} />
          <Tooltip
            contentStyle={{
              backgroundColor: "#111118",
              border: "1px solid #2a2a3a",
              borderRadius: "8px",
            }}
            itemStyle={{ color: "#a5b4fc" }}
            labelStyle={{ color: "#64748b" }}
          />
          <Area
            type="monotone"
            dataKey="online"
            stroke="#6366f1"
            strokeWidth={1.5}
            fillOpacity={1}
            fill="url(#colorOnline)"
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
};
