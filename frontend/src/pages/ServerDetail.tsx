import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
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
import type {
  PlayerStatus,
  PlayerResponse,
  ServerInfoResponse,
  UpdatePlayerRequest,
  UpdateServerRequest,
} from "@/types";
import { serverApi } from "@/api/client";
import { CopyButton, StatusBlock, ToggleButton } from "@/components";

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const PLAYER_STATUSES: PlayerStatus[] = ["None", "Regular", "Admin"];

const PLAYER_STATUS_COLOR: Record<PlayerStatus, string> = {
  None: "gray",
  Regular: "blue",
  Admin: "amber",
};

/** Fields that are mutually exclusive when toggling server flags. */
const EXCLUSIVE_TOGGLE_FIELDS = ["checked", "spoofable", "crashed"] as const;
type ToggleField = (typeof EXCLUSIVE_TOGGLE_FIELDS)[number];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Builds an UpdateServerRequest that flips `field` and resets all other
 * exclusive fields to null (the API treats them as mutually exclusive).
 */
function buildToggleUpdate(
  serverIp: string,
  field: ToggleField,
  currentValue: boolean | null,
): UpdateServerRequest {
  const resets = Object.fromEntries(
    EXCLUSIVE_TOGGLE_FIELDS.filter((f) => f !== field).map((f) => [f, null]),
  ) as Record<ToggleField, null>;

  return { server_ip: serverIp, ...resets, [field]: !currentValue };
}

// ---------------------------------------------------------------------------
// ServerDetail
// ---------------------------------------------------------------------------

export const ServerDetail = () => {
  const { ip } = useParams<{ ip: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [pingCountdown, setPingCountdown] = useState<number | null>(null);

  if (!ip) return null;

  // -- Queries ---------------------------------------------------------------

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

  // -- Mutations -------------------------------------------------------------

  const updateMutation = useMutation({
    mutationFn: (body: UpdateServerRequest) => serverApi.updateServer(body),
    onMutate: async (body: UpdateServerRequest) => {
      // Cancel any in-flight refetches so they don't overwrite our optimistic update
      await queryClient.cancelQueries({ queryKey: ["server", ip] });

      // Snapshot the previous value for rollback
      const previousServer = queryClient.getQueryData<ServerInfoResponse>(["server", ip]);

      // Optimistically apply the new toggle values
      queryClient.setQueryData<ServerInfoResponse>(["server", ip], (old) => {
        if (!old) return old;
        return {
          ...old,
          checked: body.checked ?? null,
          spoofable: body.spoofable ?? null,
          crashed: body.crashed ?? null,
        };
      });

      return { previousServer };
    },
    onError: (_err, _body, context) => {
      // Roll back to the previous server state on any error (non-200 response)
      if (context?.previousServer) {
        queryClient.setQueryData(["server", ip], context.previousServer);
      }
    },
    // Always refetch after error or success to stay in sync
    onSettled: () => {
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

  const updatePlayerMutation = useMutation({
    mutationFn: (body: UpdatePlayerRequest) => serverApi.updatePlayer(body),
    onMutate: async (body: UpdatePlayerRequest) => {
      const queryKey = ["playerList", server?.id];

      // Cancel any in-flight refetches
      await queryClient.cancelQueries({ queryKey });

      // Snapshot for rollback
      const previousPlayers = queryClient.getQueryData<PlayerResponse[]>(queryKey);

      // Optimistically update the player's status in the list
      queryClient.setQueryData<PlayerResponse[]>(queryKey, (old) =>
        old?.map((p) => (p.id === body.id ? { ...p, status: body.status } : p)),
      );

      return { previousPlayers };
    },
    onError: (_err, _body, context) => {
      // Roll back to the previous players list on any error (non-200 response)
      if (context?.previousPlayers) {
        queryClient.setQueryData(["playerList", server?.id], context.previousPlayers);
      }
    },
    // Always refetch after error or success to stay in sync
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["playerList", server?.id] });
    },
  });

  // -- Handlers --------------------------------------------------------------

  const handleToggle = (field: ToggleField) => {
    if (!server) return;
    updateMutation.mutate(buildToggleUpdate(server.ip, field, server[field]));
  };

  const handlePing = () => {
    if (!server || pingCountdown !== null) return;
    serverApi.pingServer({ server_id: server.id });
    setPingCountdown(12);
    const interval = setInterval(() => {
      setPingCountdown((prev) => {
        if (prev === null || prev <= 1) {
          clearInterval(interval);
          window.location.reload();
          return null;
        }
        return prev - 1;
      });
    }, 1000);
  };

  // -- Early returns ---------------------------------------------------------

  if (isInfoLoading)
    return <div className="text-white text-center mt-20">Loading...</div>;
  if (!server)
    return (
      <div className="text-white text-center mt-20">Server is not found</div>
    );

  // -- Derived data ----------------------------------------------------------

  const chartData = history
    ?.map((d) => ({
      time: d.timestamp,
      online: d.online,
      formattedTime: format(new Date(d.timestamp), "HH:mm"),
    }))
    .reverse();

  // -- Render ----------------------------------------------------------------

  return (
    <div className="p-6 max-w-7xl mx-auto text-white">
      <button
        onClick={() => navigate(-1)}
        className="mb-4 text-blue-400 hover:underline"
      >
        ← Back
      </button>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* ----------------------------------------------------------------- */}
        {/* Left column                                                        */}
        {/* ----------------------------------------------------------------- */}
        <div className="lg:col-span-1 space-y-6">
          {/* Server info card */}
          <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
            <div className="flex items-center gap-2 mb-2">
              <h1 className="text-2xl font-bold break-all">{server.ip}</h1>
              <CopyButton text={server.ip} />
            </div>
            <p className="text-gray-400 mb-4">{server.version_name}</p>

            <div className="space-y-3">
              <InfoRow label="Status">
                <span
                  className={
                    server.was_online ? "text-green-400" : "text-red-400"
                  }
                >
                  {server.was_online ? "Online" : "Offline"}
                </span>
              </InfoRow>
              <InfoRow label="Online">
                {server.online} / {server.max}
              </InfoRow>
              <InfoRow label="Licensed">
                {server.license ? "Yes" : "No"}
              </InfoRow>
            </div>

            {/* Management toggles */}
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

            {/* Ping */}
            <div className="mt-4 pt-4 border-t border-gray-700">
              <button
                onClick={handlePing}
                disabled={pingCountdown !== null}
                className="w-full py-2 px-4 rounded font-medium transition bg-blue-900 hover:bg-blue-800 text-blue-300 hover:text-white flex items-center justify-center gap-2 disabled:opacity-70 disabled:cursor-not-allowed"
              >
                <span>📡</span>
                <span>
                  {pingCountdown !== null
                    ? `Reloading in ${pingCountdown}s...`
                    : "Ping Server"}
                </span>
              </button>
            </div>

            {/* Delete */}
            <div className="mt-4 pt-4 border-t border-gray-700">
              {showDeleteConfirm ? (
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
                      onClick={() => deleteMutation.mutate(server.id)}
                      disabled={deleteMutation.isPending}
                      className="flex-1 py-2 px-4 rounded font-medium transition bg-red-600 hover:bg-red-700 text-white disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {deleteMutation.isPending ? "Deleting..." : "Confirm"}
                    </button>
                  </div>
                </div>
              ) : (
                <button
                  onClick={() => setShowDeleteConfirm(true)}
                  className="w-full py-2 px-4 rounded font-medium transition bg-red-900 hover:bg-red-800 text-red-300 hover:text-white flex items-center justify-center gap-2"
                >
                  <span>🗑</span>
                  <span>Delete Server</span>
                </button>
              )}
            </div>
          </div>

          {/* MOTD */}
          <HtmlCard title="MOTD" html={server.description_html} />

          {/* Disconnect reason */}
          {server.disconnect_reason_html && (
            <HtmlCard
              title="Disconnect reason"
              html={server.disconnect_reason_html}
            />
          )}
        </div>

        {/* ----------------------------------------------------------------- */}
        {/* Right column                                                       */}
        {/* ----------------------------------------------------------------- */}
        <div className="lg:col-span-2 space-y-6">
          {/* Online graph */}
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

          {/* Players */}
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
                            {PLAYER_STATUSES.map((status) => (
                              <StatusBlock
                                key={status}
                                label={status}
                                active={player.status === status}
                                activeColor={PLAYER_STATUS_COLOR[status]}
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

// ---------------------------------------------------------------------------
// Small local components
// ---------------------------------------------------------------------------

/** A bordered key/value row used inside the server info card. */
const InfoRow = ({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) => (
  <div className="flex justify-between border-b border-gray-700 pb-2">
    <span>{label}:</span>
    <span>{children}</span>
  </div>
);

/** A card that renders server HTML content (MOTD, disconnect reason, etc.). */
const HtmlCard = ({ title, html }: { title: string; html: string }) => (
  <div className="bg-gray-800 p-6 rounded-lg border border-gray-700 overflow-hidden">
    <h3 className="font-bold mb-4">{title}</h3>
    <div
      className="prose prose-invert prose-sm max-w-none bg-gray-900 p-2 rounded"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  </div>
);