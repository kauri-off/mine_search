// View-model shapes consumed by the UI components. The API now speaks gRPC
// (see src/gen, generated from ../proto via `npx buf generate`); the adapter in
// src/api/client.ts maps the protobuf messages to these shapes. These files are
// hand-maintained.

export type { AddAddrRequest } from "./AddAddrRequest";
export type { AuthBody } from "./AuthBody";
export type { PlayerListRequest } from "./PlayerListRequest";
export type { PlayerResponse } from "./PlayerResponse";
export type { PlayerStatus } from "./PlayerStatus";
export type { ServerSnapshotsRequest } from "./ServerSnapshotsRequest";
export type { ServerSnapshotsResponse } from "./ServerSnapshotsResponse";
export type { ServerDeleteRequest } from "./ServerDeleteRequest";
export type { ServerInfoRequest } from "./ServerInfoRequest";
export type { ServerInfoResponse } from "./ServerInfoResponse";
export type { ServerListRequest } from "./ServerListRequest";
export type { StatsResponse } from "./StatsResponse";
export type { VersionStat } from "./VersionStat";
export type { UpdatePlayerRequest } from "./UpdatePlayerRequest";
export type { UpdateServerRequest } from "./UpdateServerRequest";
export type { PingServerRequest } from "./PingServerRequest";
export type { DeletePlayerRequest } from "./DeletePlayerRequest";
export type { OverwriteServerRequest } from "./OverwriteServerRequest";
export type { PlayerSearchRequest } from "./PlayerSearchRequest";
export type { PlayerSearchResponse } from "./PlayerSearchResponse";
