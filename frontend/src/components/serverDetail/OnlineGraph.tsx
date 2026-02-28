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
    <div className="bg-gray-800 p-6 rounded-lg border border-gray-700 h-96">
      <h3 className="font-bold mb-4">{t.onlineGraph.title}</h3>
      {isLoading || data === undefined ? (
        <div className="flex flex-col justify-end h-4/5 animate-pulse">
          <div className="flex items-end gap-1 h-full px-2 pb-1">
            {[35, 55, 40, 70, 50, 85, 45, 75, 60, 90, 55, 65].map((h, i) => (
              <div
                key={i}
                className="flex-1 bg-blue-500/20 rounded-t"
                style={{ height: `${h}%` }}
              />
            ))}
          </div>
          <div className="h-px bg-gray-700 mx-2" />
        </div>
      ) : (
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={data}>
            <defs>
              <linearGradient id="colorOnline" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.8} />
                <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
              </linearGradient>
            </defs>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
            <XAxis dataKey="formattedTime" stroke="#9ca3af" />
            <YAxis stroke="#9ca3af" />
            <Tooltip
              contentStyle={{
                backgroundColor: "#1f2937",
                border: "none",
                borderRadius: "8px",
              }}
              itemStyle={{ color: "#fff" }}
            />
            <Area
              type="monotone"
              dataKey="online"
              stroke="#3b82f6"
              fillOpacity={1}
              fill="url(#colorOnline)"
            />
          </AreaChart>
        </ResponsiveContainer>
      )}
    </div>
  );
};
