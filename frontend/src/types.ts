// View-model shapes consumed by the UI components. The API speaks gRPC (see
// src/gen, generated from ../proto via `npx buf generate`); the adapter in
// src/api/client.ts maps the protobuf messages to/from these shapes —
// snake_case fields, string enums, and numbers instead of bigint where the UI
// doesn't need 64-bit precision.
//
// These are hand-maintained. Keep
// them in sync with proto/proto/api.proto and the conversions in client.ts.

// ----- Auth -----

export type AuthBody = { password: string };

// ----- Players -----

export type PlayerStatus = "None" | "Regular" | "Admin";

export type PlayerListRequest = { server_id: number };

export type PlayerResponse = {
  id: number;
  server_id: number;
  name: string;
  status: PlayerStatus;
  last_seen_at: string;
};

export type UpdatePlayerRequest = { id: number; status: PlayerStatus };

export type DeletePlayerRequest = { id: number };

export type PlayerSearchRequest = {
  limit: number;
  offset_id: number | null;
  name_contains: string | null;
  status: PlayerStatus | null;
  licensed: boolean | null;
};

export type PlayerSearchResponse = {
  id: number;
  server_id: number;
  server_ip: string;
  name: string;
  status: PlayerStatus;
  last_seen_at: string;
};

// ----- Servers -----

// Manual "how do I get in" classification, set by an operator. Independent of
// is_checked / is_crashed and of the auto-detected requires_mods.
export type JoinStatus =
  | "Undetermined"
  | "Spoofable"
  | "Whitelist"
  | "Password"
  | "Modded"
  | "Broken";

export type AddAddrRequest = { addr: string; quick: boolean };

export type ServerDeleteRequest = { id: number };

export type ServerInfoRequest = { ip: string };

export type ServerInfoResponse = {
  id: number;
  ip: string;
  online: number;
  max: number;
  version_name: string;
  protocol: number;
  license: boolean;
  disconnect_reason_html: string | null;
  updated: string;
  description_html: string;
  was_online: boolean;
  is_checked: boolean;
  join_status: JoinStatus;
  is_crashed: boolean;
  requires_mods: boolean;
  favicon: string | null;
  ping: bigint | null;
};

export type ServerListRequest = {
  limit: number;
  offset_id: number | null;
  licensed: boolean | null;
  checked: boolean | null;
  join_status: JoinStatus | null;
  crashed: boolean | null;
  has_players: boolean | null;
  online: boolean | null;
  requires_mods: boolean | null;
  has_none_players: boolean | null;
  // Free-text search matched against IP, version name, and plain-text MOTD.
  query: string | null;
};

export type UpdateServerRequest = {
  server_ip: string;
  is_checked: boolean | null;
  join_status: JoinStatus | null;
  is_crashed: boolean | null;
};

export type OverwriteServerRequest = {
  server_id: number;
  port: number | null;
  version_name: string | null;
  protocol: number | null;
  is_online_mode: boolean | null;
  requires_mods: boolean | null;
  is_online: boolean | null;
  ping: bigint | null;
  favicon: string | null;
  is_checked: boolean | null;
  join_status: JoinStatus | null;
  is_crashed: boolean | null;
};

export type PingServerRequest = {
  server_id: number;
  with_connection: boolean;
  worker_id: string;
};

// ----- Snapshots -----

export type ServerSnapshotsRequest = { server_id: number; limit: number };

export type ServerSnapshotsResponse = {
  server_id: number;
  players_online: number;
  players_max: number;
  recorded_at: string;
};

// ----- Stats -----

export type VersionStat = { version: string; count: number };

export type StatsResponse = {
  total_servers: number;
  cracked_servers: number;
  online_servers: number;
  crashed_servers: number;
  mod_required_servers: number;
  spoofable_servers: number;
  total_players: number;
  admin_players: number;
  avg_ping: number | null;
  version_distribution: Array<VersionStat>;
  db_size_mb: number;
  favicon_size_mb: number;
};
