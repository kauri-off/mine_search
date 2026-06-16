import { useState, useEffect, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
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
import { serverApi, workerApi } from "@/api/client";
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
  const [isPinging, setIsPinging] = useState(false);
  const [showWorkerSelect, setShowWorkerSelect] = useState(false);
  const [showPingSplit, setShowPingSplit] = useState(false);
  const [selectedWorkerId, setSelectedWorkerId] = useState<string | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [editError, setEditError] = useState<string | null>(null);
  const pingTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // -- Queries ---------------------------------------------------------------

  const { data: server, isLoading: isInfoLoading } = useQuery({
    queryKey: ["server", ip],
    queryFn: () => serverApi.fetchServerInfo(ip!),
    enabled: !!ip,
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

  const { data: workers } = useQuery({
    queryKey: ["workers"],
    queryFn: workerApi.listWorkers,
    refetchInterval: 3000,
  });
  const onlineWorkers = (workers ?? []).filter((w) => w.online);

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

  // -- Live updates ----------------------------------------------------------
  // Subscribe to server-info pushes while mounted: the stream emits the current
  // state, then re-emits on every change (manual ping or background re-probe),
  // so the page reflects new data in real time without a fixed wait.

  useEffect(() => {
    if (!ip) return;
    const controller = new AbortController();
    let cancelled = false;

    (async () => {
      while (!cancelled) {
        try {
          for await (const info of serverApi.streamServerInfo(ip, controller.signal)) {
            queryClient.setQueryData<ServerInfoResponse>(["server", ip], info);
            queryClient.invalidateQueries({ queryKey: ["serverData", info.id] });
            queryClient.invalidateQueries({ queryKey: ["playerList", info.id] });
            if (pingTimeoutRef.current) {
              clearTimeout(pingTimeoutRef.current);
              pingTimeoutRef.current = null;
            }
            setIsPinging(false);
          }
        } catch {
          // Aborts and transient stream errors both land here.
        }
        if (cancelled) break;
        // Brief backoff before reconnecting a dropped stream.
        await new Promise((r) => setTimeout(r, 2000));
      }
    })();

    return () => {
      cancelled = true;
      controller.abort();
      if (pingTimeoutRef.current) {
        clearTimeout(pingTimeoutRef.current);
        pingTimeoutRef.current = null;
      }
    };
  }, [ip, queryClient]);

  // -- Handlers --------------------------------------------------------------

  const handleToggle = (field: ToggleField) => {
    if (!server) return;
    updateMutation.mutate(buildToggleUpdate(server.ip, field, server[field]));
  };

  const handlePingRequest = () => {
    if (!server || isPinging) return;
    setShowWorkerSelect(true);
  };

  const handleWorkerSelect = (workerId: string) => {
    setSelectedWorkerId(workerId);
    setShowWorkerSelect(false);
    setShowPingSplit(true);
  };

  const handlePing = (withConnection: boolean) => {
    if (!server || isPinging || !selectedWorkerId) return;
    const serverId = server.id;
    setShowPingSplit(false);
    serverApi.pingServer({
      server_id: serverId,
      with_connection: withConnection,
      worker_id: selectedWorkerId,
    });
    setIsPinging(true);
    if (pingTimeoutRef.current) clearTimeout(pingTimeoutRef.current);
    // Fallback so the spinner can't get stuck if the probe yields no change.
    pingTimeoutRef.current = setTimeout(() => {
      setIsPinging(false);
      pingTimeoutRef.current = null;
    }, 15000);
  };

  // -- Derived data ----------------------------------------------------------

  const chartData = history ? buildChartData(history) : undefined;

  // -- Render ----------------------------------------------------------------

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="px-3 py-3 sm:px-6 sm:py-5 max-w-7xl mx-auto">
        <button
          onClick={() => navigate(-1)}
          className="inline-flex items-center gap-1.5 text-sm text-slate-500 hover:text-slate-300 transition-colors mb-5"
        >
          <ArrowLeft className="w-4 h-4" />
          {t.serverDetail.back}
        </button>

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
                isPinging={isPinging}
                showWorkerSelect={showWorkerSelect}
                showPingSplit={showPingSplit}
                workers={onlineWorkers}
                showDeleteConfirm={showDeleteConfirm}
                isDeletePending={deleteMutation.isPending}
                isEditing={isEditing}
                isEditPending={overwriteMutation.isPending}
                editError={editError}
                onToggle={handleToggle}
                onPingRequest={handlePingRequest}
                onWorkerSelect={handleWorkerSelect}
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
