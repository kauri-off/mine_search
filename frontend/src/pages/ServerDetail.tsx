import { useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import type {
  PlayerStatus,
  PlayerResponse,
  ServerInfoResponse,
  UpdatePlayerRequest,
  UpdateServerRequest,
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

  if (!ip) return null;

  // -- Queries ---------------------------------------------------------------

  const { data: server, isLoading: isInfoLoading } = useQuery({
    queryKey: ["server", ip],
    queryFn: () => serverApi.fetchServerInfo(ip),
  });

  const { data: history, isLoading: isHistoryLoading } = useQuery({
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
      await queryClient.cancelQueries({ queryKey: ["server", ip] });

      const previousServer = queryClient.getQueryData<ServerInfoResponse>([
        "server",
        ip,
      ]);

      queryClient.setQueryData<ServerInfoResponse>(["server", ip], (old) => {
        if (!old) return old;
        return {
          ...old,
          ...(body.checked !== null && { checked: body.checked }),
          ...(body.spoofable !== null && { spoofable: body.spoofable }),
          ...(body.crashed !== null && { crashed: body.crashed }),
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

      const previousPlayers =
        queryClient.getQueryData<PlayerResponse[]>(queryKey);

      queryClient.setQueryData<PlayerResponse[]>(queryKey, (old) =>
        old?.map((p) => (p.id === body.id ? { ...p, status: body.status } : p)),
      );

      return { previousPlayers };
    },
    onError: (_err, _body, context) => {
      if (context?.previousPlayers) {
        queryClient.setQueryData(
          ["playerList", server?.id],
          context.previousPlayers,
        );
      }
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
    serverApi.pingServer({
      server_id: serverId,
      with_connection: withConnection,
    });
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
    <div className="p-6 max-w-7xl mx-auto text-white">
      <button
        onClick={() => navigate(-1)}
        aria-label={t.serverDetail.back}
        className="mb-4 text-blue-400 hover:underline flex items-center gap-1"
      >
        <span aria-hidden="true">←</span> {t.serverDetail.back}
      </button>

      {isInfoLoading ? (
        <div className="flex items-center justify-center mt-20">
          <Spinner className="w-10 h-10" />
        </div>
      ) : !server ? (
        <div className="text-white text-center mt-20">{t.serverDetail.notFound}</div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Left column */}
          <div className="lg:col-span-1 space-y-6">
            <ServerInfoCard
              server={server}
              pingCountdown={pingCountdown}
              showPingSplit={showPingSplit}
              showDeleteConfirm={showDeleteConfirm}
              isDeletePending={deleteMutation.isPending}
              onToggle={handleToggle}
              onPingRequest={handlePingRequest}
              onPing={handlePing}
              onDeleteRequest={() => setShowDeleteConfirm(true)}
              onDeleteCancel={() => setShowDeleteConfirm(false)}
              onDeleteConfirm={() => deleteMutation.mutate(server.id)}
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
          <div className="lg:col-span-2 space-y-6">
            <OnlineGraph data={chartData} isLoading={isHistoryLoading} />

            <PlayersTable
              players={players}
              onUpdateStatus={(id, status: PlayerStatus) =>
                updatePlayerMutation.mutate({ id, status })
              }
            />
          </div>
        </div>
      )}
    </div>
  );
};
