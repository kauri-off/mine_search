import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { serverApi } from "../api/client";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { format } from "date-fns";
import type { UpdateServerRequest } from "../types/UpdateServerRequest";
import type { UpdatePlayerRequest } from "../types/UpdatePlayerRequest";
import type { PlayerStatus } from "../types/PlayerStatus";

const CopyButton = ({ text }: { text: string }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // fallback for older browsers
      const el = document.createElement("textarea");
      el.value = text;
      document.body.appendChild(el);
      el.select();
      document.execCommand("copy");
      document.body.removeChild(el);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <button
      onClick={handleCopy}
      title={copied ? "Copied!" : "Copy IP"}
      className={`
        relative inline-flex items-center justify-center
        w-7 h-7 rounded-md transition-all duration-200
        ${
          copied
            ? "bg-green-500/20 text-green-400 scale-95"
            : "bg-gray-700 hover:bg-gray-600 text-gray-400 hover:text-white"
        }
      `}
    >
      {copied ? (
        /* Checkmark icon */
        <svg
          xmlns="http://www.w3.org/2000/svg"
          className="w-4 h-4 animate-[pop_0.2s_ease-out]"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={2.5}
          strokeLinecap="round"
          strokeLinejoin="round"
          style={{ animation: "pop 0.2s ease-out" }}
        >
          <polyline points="20 6 9 17 4 12" />
        </svg>
      ) : (
        /* Clipboard icon */
        <svg
          xmlns="http://www.w3.org/2000/svg"
          className="w-4 h-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <rect x="9" y="2" width="6" height="4" rx="1" ry="1" />
          <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
        </svg>
      )}

      {/* Tooltip */}
      <span
        className={`
          absolute -top-8 left-1/2 -translate-x-1/2
          text-xs px-2 py-1 rounded bg-gray-900 border border-gray-700
          whitespace-nowrap pointer-events-none
          transition-opacity duration-150
          ${copied ? "opacity-100" : "opacity-0"}
        `}
      >
        Copied!
      </span>

      <style>{`
        @keyframes pop {
          0%   { transform: scale(0.6); opacity: 0.5; }
          60%  { transform: scale(1.2); }
          100% { transform: scale(1);   opacity: 1; }
        }
      `}</style>
    </button>
  );
};

export const ServerDetail = () => {
  const { ip } = useParams<{ ip: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  if (!ip) return null;

  const { data: server, isLoading: isInfoLoading } = useQuery({
    queryKey: ["server", ip],
    queryFn: () => serverApi.fetchServerInfo(ip),
  });

  const { data: history } = useQuery({
    queryKey: ["serverData", server?.id],
    queryFn: () =>
      serverApi.fetchServerData({ server_id: server!.id, limit: 100 }),
    enabled: !!server?.id,
  });

  const { data: players } = useQuery({
    queryKey: ["playerList", server?.id],
    queryFn: () => serverApi.fetchPlayerList({ server_id: server!.id }),
    enabled: !!server?.id,
  });

  const updatePlayerMutation = useMutation({
    mutationFn: (body: UpdatePlayerRequest) => serverApi.updatePlayer(body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["playerList", server?.id] });
    },
  });

  const updateMutation = useMutation({
    mutationFn: (body: UpdateServerRequest) => serverApi.updateServer(body),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["server", ip] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: number) => serverApi.deleteServer({ id }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["servers"] });
      navigate("/");
    },
  });

  const handleToggle = (field: "checked" | "spoofable" | "crashed") => {
    if (!server) return;

    const allFields = ["checked", "spoofable", "crashed"] as const;
    const resetFields = Object.fromEntries(
      allFields.filter((f) => f !== field).map((f) => [f, null]),
    ) as Record<(typeof allFields)[number], null>;

    updateMutation.mutate({
      server_ip: server.ip,
      ...resetFields,
      [field]: !server[field],
    });
  };

  const handleDeleteConfirm = () => {
    if (!server) return;
    deleteMutation.mutate(server.id);
  };

  if (isInfoLoading)
    return <div className="text-white text-center mt-20">Loading...</div>;
  if (!server)
    return (
      <div className="text-white text-center mt-20">Server is not found</div>
    );

  const chartData = history
    ?.map((d) => ({
      time: d.timestamp,
      online: d.online,
      formattedTime: format(new Date(d.timestamp), "HH:mm"),
    }))
    .reverse();

  return (
    <div className="p-6 max-w-7xl mx-auto text-white">
      <button
        onClick={() => navigate(-1)}
        className="mb-4 text-blue-400 hover:underline"
      >
        ← Back
      </button>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <div className="lg:col-span-1 space-y-6">
          <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
            {/* IP + Copy button */}
            <div className="flex items-center gap-2 mb-2">
              <h1 className="text-2xl font-bold break-all">{server.ip}</h1>
              <CopyButton text={server.ip} />
            </div>
            <p className="text-gray-400 mb-4">{server.version_name}</p>

            <div className="space-y-3">
              <div className="flex justify-between border-b border-gray-700 pb-2">
                <span>Status:</span>
                <span
                  className={
                    server.was_online ? "text-green-400" : "text-red-400"
                  }
                >
                  {server.was_online ? "Online" : "Offline"}
                </span>
              </div>
              <div className="flex justify-between border-b border-gray-700 pb-2">
                <span>Online:</span>
                <span>
                  {server.online} / {server.max}
                </span>
              </div>
              <div className="flex justify-between border-b border-gray-700 pb-2">
                <span>Licensed:</span>
                <span>{server.license ? "Yes" : "No"}</span>
              </div>
            </div>

            <div className="mt-6 space-y-2">
              <h3 className="font-semibold mb-2 text-gray-300">Management:</h3>
              <ToggleButton
                label="Checked"
                active={!!server.checked}
                onClick={() => handleToggle("checked")}
              />
              <ToggleButton
                label="Spoofable"
                active={!!server.spoofable}
                onClick={() => handleToggle("spoofable")}
              />
              <ToggleButton
                label="Crashed"
                active={!!server.crashed}
                onClick={() => handleToggle("crashed")}
                color="red"
              />
            </div>

            <div className="mt-4 pt-4 border-t border-gray-700">
              {!showDeleteConfirm ? (
                <button
                  onClick={() => setShowDeleteConfirm(true)}
                  className="w-full py-2 px-4 rounded font-medium transition bg-red-900 hover:bg-red-800 text-red-300 hover:text-white flex items-center justify-center gap-2"
                >
                  <span>🗑</span>
                  <span>Delete Server</span>
                </button>
              ) : (
                <div className="space-y-2">
                  <p className="text-sm text-red-400 text-center font-medium">
                    Are you sure you want to delete{" "}
                    <span className="font-bold text-white">{server.ip}</span>?
                  </p>
                  <p className="text-xs text-gray-500 text-center">
                    This action cannot be undone.
                  </p>
                  <div className="flex gap-2">
                    <button
                      onClick={() => setShowDeleteConfirm(false)}
                      disabled={deleteMutation.isPending}
                      className="flex-1 py-2 px-4 rounded font-medium transition bg-gray-700 hover:bg-gray-600 text-gray-300 disabled:opacity-50"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={handleDeleteConfirm}
                      disabled={deleteMutation.isPending}
                      className="flex-1 py-2 px-4 rounded font-medium transition bg-red-600 hover:bg-red-700 text-white disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {deleteMutation.isPending ? "Deleting..." : "Confirm"}
                    </button>
                  </div>
                </div>
              )}
            </div>
          </div>

          <div className="bg-gray-800 p-6 rounded-lg border border-gray-700 overflow-hidden">
            <h3 className="font-bold mb-4">MOTD</h3>
            <div
              className="prose prose-invert prose-sm max-w-none bg-gray-900 p-2 rounded"
              dangerouslySetInnerHTML={{ __html: server.description_html }}
            />
          </div>
          {server.disconnect_reason_html && (
            <div className="bg-gray-800 p-6 rounded-lg border border-gray-700 overflow-hidden">
              <h3 className="font-bold mb-4">Disconnect reason</h3>
              <div
                className="prose prose-invert prose-sm max-w-none bg-gray-900 p-2 rounded"
                dangerouslySetInnerHTML={{
                  __html: server.disconnect_reason_html,
                }}
              />
            </div>
          )}
        </div>

        <div className="lg:col-span-2 space-y-6">
          <div className="bg-gray-800 p-6 rounded-lg border border-gray-700 h-96">
            <h3 className="font-bold mb-4">Online graph</h3>
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData}>
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
          </div>

          <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
            <h3 className="font-bold mb-4">Players (All)</h3>
            {players && players.length > 0 ? (
              <div className="overflow-x-auto">
                <table className="w-full text-sm">
                  <thead>
                    <tr className="border-b border-gray-700 text-gray-400 text-left">
                      <th className="pb-2 font-medium">Name</th>
                      <th className="pb-2 font-medium text-right">Status</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-gray-700/50">
                    {players.map((player) => (
                      <tr key={player.id} className="group">
                        <td className="py-2.5 pr-4">
                          <div className="flex items-center gap-2">
                            <span className="text-white">{player.name}</span>
                            <span className="opacity-0 group-hover:opacity-100 transition-opacity">
                              <CopyButton text={player.name} />
                            </span>
                          </div>
                        </td>
                        <td className="py-2.5">
                          <div className="flex items-center justify-end gap-1.5">
                            {(
                              ["None", "Regular", "Admin"] as PlayerStatus[]
                            ).map((status) => (
                              <StatusBlock
                                key={status}
                                label={status}
                                active={player.status === status}
                                activeColor={
                                  status === "None"
                                    ? "gray"
                                    : status === "Regular"
                                      ? "blue"
                                      : "amber"
                                }
                                onClick={() => {
                                  if (player.status !== status) {
                                    updatePlayerMutation.mutate({
                                      id: player.id,
                                      status,
                                    });
                                  }
                                }}
                              />
                            ))}
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            ) : (
              <span className="text-gray-500">Empty</span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

const ToggleButton = ({ label, active, onClick, color = "blue" }: any) => (
  <button
    onClick={onClick}
    className={`w-full py-2 px-4 rounded font-medium transition flex justify-between items-center
            ${
              active
                ? color === "red"
                  ? "bg-red-600 hover:bg-red-700"
                  : "bg-blue-600 hover:bg-blue-700"
                : "bg-gray-700 hover:bg-gray-600"
            }`}
  >
    <span>{label}</span>
    <span className="text-xs uppercase bg-black/20 px-2 py-0.5 rounded">
      {active ? "ON" : "OFF"}
    </span>
  </button>
);

const colorMap: Record<string, { active: string; inactive: string }> = {
  gray: {
    active: "bg-gray-500/30 text-gray-300 border-gray-500",
    inactive: "bg-gray-800 text-gray-600 border-gray-700",
  },
  blue: {
    active: "bg-blue-500/20 text-blue-300 border-blue-500",
    inactive: "bg-gray-800 text-gray-600 border-gray-700",
  },
  amber: {
    active: "bg-amber-500/20 text-amber-300 border-amber-500",
    inactive: "bg-gray-800 text-gray-600 border-gray-700",
  },
};

const StatusBlock = ({
  label,
  active,
  activeColor,
  onClick,
}: {
  label: string;
  active: boolean;
  activeColor: string;
  onClick?: () => void;
}) => {
  const colors = colorMap[activeColor] ?? colorMap.gray;
  return (
    <span
      onClick={onClick}
      className={`px-2 py-0.5 rounded border text-xs font-medium transition-colors select-none
        ${active ? `${colors.active} cursor-default` : `${colors.inactive} cursor-pointer hover:border-gray-500 hover:text-gray-400`}
      `}
    >
      {label}
    </span>
  );
};
