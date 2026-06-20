import { createClient, type Interceptor, ConnectError, Code } from "@connectrpc/connect";
import { createGrpcWebTransport } from "@connectrpc/connect-web";
import { Api } from "@/gen/api_pb";
import { PlayerStatus as PbPlayerStatus } from "@/gen/api_pb";
import type { WorkerInfo } from "@/gen/api_pb";
import { Control } from "@/gen/worker_pb";
import type {
  AuthBody,
  StatsResponse,
  ServerSnapshotsRequest,
  ServerListRequest,
  ServerInfoResponse,
  ServerSnapshotsResponse,
  AddAddrRequest,
  ServerDeleteRequest,
  UpdateServerRequest,
  PlayerResponse,
  PlayerListRequest,
  UpdatePlayerRequest,
  PingServerRequest,
  DeletePlayerRequest,
  OverwriteServerRequest,
  PlayerSearchRequest,
  PlayerSearchResponse,
  PlayerStatus,
} from "@/types";

const TOKEN_KEY = "ms_token";

export const getToken = () => localStorage.getItem(TOKEN_KEY);
export const setToken = (t: string) => localStorage.setItem(TOKEN_KEY, t);
export const clearToken = () => localStorage.removeItem(TOKEN_KEY);

// Inject the session token and redirect to /login on Unauthenticated.
const authInterceptor: Interceptor = (next) => async (req) => {
  const token = getToken();
  if (token) req.header.set("Authorization", `Bearer ${token}`);
  try {
    return await next(req);
  } catch (e) {
    if (e instanceof ConnectError && e.code === Code.Unauthenticated) {
      clearToken();
      if (!window.location.pathname.startsWith("/login")) {
        window.location.href = "/login";
      }
    }
    throw e;
  }
};

const transport = createGrpcWebTransport({
  baseUrl: window.location.origin,
  interceptors: [authInterceptor],
});

export const client = createClient(Api, transport);

// ---------------------------------------------------------------------------
// Conversions between protobuf wire types and the view-model shapes the
// UI components consume (snake_case, string enums, numbers instead of bigint).
// ---------------------------------------------------------------------------

const u = <T>(v: T | undefined): T | null => (v === undefined ? null : v);

const STATUS_TO_STR: Record<number, PlayerStatus> = {
  [PbPlayerStatus.NONE]: "None",
  [PbPlayerStatus.REGULAR]: "Regular",
  [PbPlayerStatus.ADMIN]: "Admin",
};
const STATUS_TO_NUM: Record<PlayerStatus, PbPlayerStatus> = {
  None: PbPlayerStatus.NONE,
  Regular: PbPlayerStatus.REGULAR,
  Admin: PbPlayerStatus.ADMIN,
};

type PbServerInfo = Awaited<ReturnType<typeof client.getServerInfo>>;
const toServerInfo = (s: PbServerInfo): ServerInfoResponse => ({
  id: s.id,
  ip: s.ip,
  online: s.online,
  max: s.max,
  version_name: s.versionName,
  protocol: s.protocol,
  license: s.license,
  disconnect_reason_html: u(s.disconnectReasonHtml),
  updated: s.updated,
  description_html: s.descriptionHtml,
  was_online: s.wasOnline,
  is_checked: s.isChecked,
  is_spoofable: u(s.isSpoofable),
  is_crashed: s.isCrashed,
  requires_mods: s.requiresMods,
  favicon: u(s.favicon),
  ping: s.ping === undefined ? null : s.ping,
});

export const authApi = {
  login: async (body: AuthBody): Promise<void> => {
    const res = await client.login({ password: body.password });
    setToken(res.token);
  },
};

export const systemApi = {
  // Triggers watchtower to pull new images and recreate the stack's containers.
  triggerUpdate: async (): Promise<void> => {
    await client.triggerUpdate({});
  },
};

export const serverApi = {
  fetchMe: async (): Promise<null> => {
    await client.me({});
    return null;
  },

  fetchStats: async (): Promise<StatsResponse> => {
    const s = await client.getStats({});
    return {
      total_servers: Number(s.totalServers),
      cracked_servers: Number(s.crackedServers),
      online_servers: Number(s.onlineServers),
      crashed_servers: Number(s.crashedServers),
      mod_required_servers: Number(s.modRequiredServers),
      spoofable_servers: Number(s.spoofableServers),
      total_players: Number(s.totalPlayers),
      admin_players: Number(s.adminPlayers),
      avg_ping: s.avgPing === undefined ? null : s.avgPing,
      version_distribution: s.versionDistribution.map((v) => ({
        version: v.version,
        count: Number(v.count),
      })),
      db_size_mb: s.dbSizeMb,
      favicon_size_mb: s.faviconSizeMb,
    };
  },

  fetchServerList: async (body: ServerListRequest): Promise<ServerInfoResponse[]> => {
    const res = await client.listServers({
      limit: BigInt(body.limit),
      offsetId: u(body.offset_id) ?? undefined,
      licensed: u(body.licensed) ?? undefined,
      checked: u(body.checked) ?? undefined,
      spoofable: u(body.spoofable) ?? undefined,
      crashed: u(body.crashed) ?? undefined,
      hasPlayers: u(body.has_players) ?? undefined,
      online: u(body.online) ?? undefined,
      requiresMods: u(body.requires_mods) ?? undefined,
      hasNonePlayers: u(body.has_none_players) ?? undefined,
      query: u(body.query) ?? undefined,
    });
    return res.servers.map(toServerInfo);
  },

  fetchServerInfo: async (ip: string): Promise<ServerInfoResponse> => {
    return toServerInfo(await client.getServerInfo({ ip }));
  },

  // Live subscription: yields the current ServerInfo immediately, then again
  // each time the server's row changes (manual ping or background re-probe).
  streamServerInfo: async function* (
    ip: string,
    signal: AbortSignal,
  ): AsyncGenerator<ServerInfoResponse> {
    for await (const s of client.streamServerInfo({ ip }, { signal })) {
      yield toServerInfo(s);
    }
  },

  fetchServerSnapshots: async (
    body: ServerSnapshotsRequest,
  ): Promise<ServerSnapshotsResponse[]> => {
    const res = await client.getServerSnapshots({
      serverId: body.server_id,
      limit: BigInt(body.limit),
    });
    return res.snapshots.map((s) => ({
      server_id: s.serverId,
      players_online: s.playersOnline,
      players_max: s.playersMax,
      recorded_at: s.recordedAt,
    }));
  },

  updateServer: (body: UpdateServerRequest) =>
    client.updateServer({
      serverIp: body.server_ip,
      isChecked: u(body.is_checked) ?? undefined,
      isSpoofable: u(body.is_spoofable) ?? undefined,
      isCrashed: u(body.is_crashed) ?? undefined,
    }),

  addTarget: (body: AddAddrRequest, workerId: string) =>
    client.addTarget({ addr: body.addr, quick: body.quick, workerId }),

  deleteServer: (body: ServerDeleteRequest) => client.deleteServer({ id: body.id }),

  fetchPlayerList: async (body: PlayerListRequest): Promise<PlayerResponse[]> => {
    const res = await client.listPlayers({ serverId: body.server_id });
    return res.players.map((p) => ({
      id: p.id,
      server_id: p.serverId,
      name: p.name,
      status: STATUS_TO_STR[p.status] ?? "None",
      last_seen_at: p.lastSeenAt,
    }));
  },

  updatePlayer: (body: UpdatePlayerRequest) =>
    client.updatePlayer({ id: body.id, status: STATUS_TO_NUM[body.status] }),

  deletePlayer: (body: DeletePlayerRequest) => client.deletePlayer({ id: body.id }),

  overwriteServer: (body: OverwriteServerRequest) =>
    client.overwriteServer({
      serverId: body.server_id,
      port: u(body.port) ?? undefined,
      versionName: u(body.version_name) ?? undefined,
      protocol: u(body.protocol) ?? undefined,
      isOnlineMode: u(body.is_online_mode) ?? undefined,
      requiresMods: u(body.requires_mods) ?? undefined,
      isOnline: u(body.is_online) ?? undefined,
      ping: body.ping === null || body.ping === undefined ? undefined : BigInt(body.ping),
      favicon: u(body.favicon) ?? undefined,
      isChecked: u(body.is_checked) ?? undefined,
      isSpoofable: u(body.is_spoofable) ?? undefined,
      isCrashed: u(body.is_crashed) ?? undefined,
    }),

  addTargetList: (body: AddAddrRequest[], workerId: string) =>
    client.addTargetList({
      targets: body.map((t) => ({ addr: t.addr, quick: t.quick })),
      workerId,
    }),

  pingServer: (body: PingServerRequest) =>
    client.pingServer({
      serverId: body.server_id,
      withConnection: body.with_connection,
      workerId: body.worker_id,
    }),

  searchPlayers: async (body: PlayerSearchRequest): Promise<PlayerSearchResponse[]> => {
    const res = await client.searchPlayers({
      limit: BigInt(body.limit),
      offsetId: u(body.offset_id) ?? undefined,
      nameContains: u(body.name_contains) ?? undefined,
      status: body.status ? STATUS_TO_NUM[body.status] : undefined,
      licensed: u(body.licensed) ?? undefined,
    });
    return res.players.map((p) => ({
      id: p.id,
      server_id: p.serverId,
      server_ip: p.serverIp,
      name: p.name,
      status: STATUS_TO_STR[p.status] ?? "None",
      last_seen_at: p.lastSeenAt,
    }));
  },
};

// ---------------------------------------------------------------------------
// Worker management (new). Returns proto WorkerInfo directly; the page converts
// bigint metrics with Number() at render time.
// ---------------------------------------------------------------------------

export interface WorkerConfigInput {
  threads: number;
  search_module: boolean;
  update_module: boolean;
  update_with_connection: boolean;
  only_update_spoofable: boolean;
  only_update_cracked: boolean;
  update_interval_secs: number;
  update_concurrency: number;
}

export const workerApi = {
  listWorkers: async (): Promise<WorkerInfo[]> => {
    const res = await client.listWorkers({});
    return res.workers;
  },

  updateWorkerConfig: (workerId: string, config: WorkerConfigInput) =>
    client.updateWorkerConfig({
      workerId,
      config: {
        threads: config.threads,
        searchModule: config.search_module,
        updateModule: config.update_module,
        updateWithConnection: config.update_with_connection,
        onlyUpdateSpoofable: config.only_update_spoofable,
        onlyUpdateCracked: config.only_update_cracked,
        updateIntervalSecs: config.update_interval_secs,
        updateConcurrency: config.update_concurrency,
      },
    }),

  // Renames a worker. An empty/whitespace name clears the override, falling back
  // to the worker id for display.
  setWorkerName: (workerId: string, name: string) =>
    client.setWorkerName({ workerId, name: name.trim() || undefined }),

  pauseSearch: (id: string) => client.controlWorker({ workerId: id, control: Control.PAUSE_SEARCH }),
  resumeSearch: (id: string) => client.controlWorker({ workerId: id, control: Control.RESUME_SEARCH }),
  abortUpdate: (id: string) => client.controlWorker({ workerId: id, control: Control.ABORT_UPDATE }),
  triggerUpdate: (id: string) => client.controlWorker({ workerId: id, control: Control.TRIGGER_UPDATE }),
};
