import { useState } from "react";
import { useParams, useNavigate, Link } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft } from "lucide-react";
import type {
  PlayerStatus,
  PlayerResponse,
  ServerInfoResponse,
  UpdatePlayerRequest,
  UpdateServerRequest,
  OverwriteServerRequest,
} from "@/types";
import { serverApi } from "@/api/client";
import type { ToggleField } from "@/constants/serverDetail";
import { buildChartData, buildToggleUpdate } from "@/utils/serverDetailHelpers";
import { useTranslation } from "@/i18n";
import { ServerInfoCard } from "@/components/serverDetail/ServerInfoCard";
import { HtmlCard } from "@/components/serverDetail/HtmlCard";
import { OnlineGraph } from "@/components/serverDetail/OnlineGraph";
import { PlayersTable } from "@/components/serverDetail/PlayersTable";
import { Spinner } from "@/components";

export const ServerDetail = () => {
  const { t } = useTranslation();
  const { ip } = useParams<{ ip: string }>();
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [pingCountdown, setPingCountdown] = useState<number | null>(null);
  const [showPingSplit, setShowPingSplit] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editError, setEditError] = useState<string | null>(null);

  if (!ip) return null;

  // -- Queries ---------------------------------------------------------------

  const { data: server, isLoading: isInfoLoading } = useQuery({
    queryKey: ["server", ip],
    queryFn: () => serverApi.fetchServerInfo(ip),
    staleTime: 10 * 60 * 1000,
  });

  const { data: history, isLoading: isHistoryLoading } = useQuery({
    queryKey: ["serverData", server?.id],
    queryFn: () =>
      serverApi.fetchServerSnapshots({ server_id: server!.id, limit: 100 }),
    enabled: !!server?.id,
    staleTime: 10 * 60 * 1000,
  });

  const { data: players } = useQuery({
    queryKey: ["playerList", server?.id],
    queryFn: () => serverApi.fetchPlayerList({ server_id: server!.id }),
    enabled: !!server?.id,
    staleTime: 10 * 60 * 1000,
  });

  // -- Mutations -------------------------------------------------------------

  const updateMutation = useMutation({
    mutationFn: (body: UpdateServerRequest) => serverApi.updateServer(body),
    onMutate: async (body: UpdateServerRequest) => {
      await queryClient.cancelQueries({ queryKey: ["server", ip] });
      const previousServer = queryClient.getQueryData<ServerInfoResponse>(["server", ip]);
      queryClient.setQueryData<ServerInfoResponse>(["server", ip], (old) => {
        if (!old) return old;
        return {
          ...old,
          ...(body.is_checked !== null && { is_checked: body.is_checked }),
          ...(body.is_spoofable !== null && { is_spoofable: body.is_spoofable }),
          ...(body.is_crashed !== null && { is_crashed: body.is_crashed }),
        };
      });
      return { previousServer };
    },
    onError: (_err, _body, context) => {
      if (context?.previousServer) {
        queryClient.setQueryData(["server", ip], context.previousServer);
      }
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
      await queryClient.cancelQueries({ queryKey });
      const previousPlayers = queryClient.getQueryData<PlayerResponse[]>(queryKey);
      queryClient.setQueryData<PlayerResponse[]>(queryKey, (old) =>
        old?.map((p) => (p.id === body.id ? { ...p, status: body.status } : p)),
      );
      return { previousPlayers };
    },
    onError: (_err, _body, context) => {
      if (context?.previousPlayers) {
        queryClient.setQueryData(["playerList", server?.id], context.previousPlayers);
      }
    },
  });

  const deletePlayerMutation = useMutation({
    mutationFn: (id: number) => serverApi.deletePlayer({ id }),
    onMutate: async (id: number) => {
      const queryKey = ["playerList", server?.id];
      await queryClient.cancelQueries({ queryKey });
      const previousPlayers = queryClient.getQueryData<PlayerResponse[]>(queryKey);
      queryClient.setQueryData<PlayerResponse[]>(queryKey, (old) =>
        old?.filter((p) => p.id !== id),
      );
      return { previousPlayers };
    },
    onError: (_err, _id, context) => {
      if (context?.previousPlayers) {
        queryClient.setQueryData(["playerList", server?.id], context.previousPlayers);
      }
    },
  });

  const overwriteMutation = useMutation({
    mutationFn: (body: OverwriteServerRequest) => serverApi.overwriteServer(body),
    onSuccess: () => {
      setIsEditing(false);
      setEditError(null);
      queryClient.invalidateQueries({ queryKey: ["server", ip] });
    },
    onError: () => {
      setEditError(t.serverInfo.editError);
    },
  });

  // -- Handlers --------------------------------------------------------------

  const handleToggle = (field: ToggleField) => {
    if (!server) return;
    updateMutation.mutate(buildToggleUpdate(server.ip, field, server[field]));
  };

  const handlePingRequest = () => {
    if (!server || pingCountdown !== null) return;
    setShowPingSplit(true);
  };

  const handlePing = (withConnection: boolean) => {
    if (!server || pingCountdown !== null) return;
    const serverId = server.id;
    setShowPingSplit(false);
    serverApi.pingServer({ server_id: serverId, with_connection: withConnection });
    setPingCountdown(12);
    const interval = setInterval(() => {
      setPingCountdown((prev) => {
        if (prev === null || prev <= 1) {
          clearInterval(interval);
          queryClient.invalidateQueries({ queryKey: ["server", ip] });
          queryClient.invalidateQueries({ queryKey: ["serverData", serverId] });
          queryClient.invalidateQueries({ queryKey: ["playerList", serverId] });
          return null;
        }
        return prev - 1;
      });
    }, 1000);
  };

  // -- Derived data ----------------------------------------------------------

  const chartData = history ? buildChartData(history) : undefined;

  // -- Render ----------------------------------------------------------------

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="px-3 py-3 sm:px-6 sm:py-5 max-w-7xl mx-auto">
        <Link
          to="/"
          className="inline-flex items-center gap-1.5 text-sm text-slate-500 hover:text-slate-300 transition-colors mb-5"
        >
          <ArrowLeft className="w-4 h-4" />
          {t.serverDetail.back}
        </Link>

        {isInfoLoading ? (
          <div className="flex items-center justify-center mt-20">
            <Spinner className="w-8 h-8 text-indigo-500" />
          </div>
        ) : !server ? (
          <div className="text-slate-500 text-center mt-20">
            {t.serverDetail.notFound}
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-5">
            {/* Left column */}
            <div className="lg:col-span-1 space-y-4">
              <ServerInfoCard
                server={server}
                pingCountdown={pingCountdown}
                showPingSplit={showPingSplit}
                showDeleteConfirm={showDeleteConfirm}
                isDeletePending={deleteMutation.isPending}
                isEditing={isEditing}
                isEditPending={overwriteMutation.isPending}
                editError={editError}
                onToggle={handleToggle}
                onPingRequest={handlePingRequest}
                onPing={handlePing}
                onDeleteRequest={() => setShowDeleteConfirm(true)}
                onDeleteCancel={() => setShowDeleteConfirm(false)}
                onDeleteConfirm={() => deleteMutation.mutate(server.id)}
                onEditStart={() => {
                  setIsEditing(true);
                  setEditError(null);
                }}
                onEditSave={(data) => overwriteMutation.mutate(data)}
                onEditCancel={() => {
                  setIsEditing(false);
                  setEditError(null);
                }}
              />

              <HtmlCard title="MOTD" html={server.description_html} />

              {server.disconnect_reason_html && (
                <HtmlCard
                  title={t.serverDetail.disconnectReason}
                  html={server.disconnect_reason_html}
                />
              )}
            </div>

            {/* Right column */}
            <div className="lg:col-span-2 space-y-4">
              <OnlineGraph data={chartData} isLoading={isHistoryLoading} />
              <PlayersTable
                players={players}
                onUpdateStatus={(id, status: PlayerStatus) =>
                  updatePlayerMutation.mutate({ id, status })
                }
                onDeletePlayer={(id) => deletePlayerMutation.mutate(id)}
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
