import type { ServerListRequest } from "@/types";

export type Filters = Omit<ServerListRequest, "offset_id">;

/** The server-property filter fields, shared by the dashboard and the worker
 *  Search/Update config panels (no pagination `offset_id`/`limit`). */
export type ServerFilterValue = Omit<ServerListRequest, "offset_id" | "limit">;

export type FilterFieldKey = keyof ServerFilterValue;

/** Tri-state boolean filter keys (everything except the text/dropdown fields). */
export type BoolFilterKey = Exclude<FilterFieldKey, "query" | "join_status">;

/** An empty (no-constraint) filter — every field null. */
export const EMPTY_FILTER: ServerFilterValue = {
  online: null,
  licensed: null,
  checked: null,
  crashed: null,
  requires_mods: null,
  has_players: null,
  has_none_players: null,
  join_status: null,
  query: null,
};

/** Full dashboard filter set for the worker's Update panel (which existing
 *  servers get re-probed). Ordered for display; `query` first. */
export const UPDATE_FILTER_FIELDS: FilterFieldKey[] = [
  "query",
  "online",
  "licensed",
  "checked",
  "crashed",
  "requires_mods",
  "has_players",
  "has_none_players",
  "join_status",
];

/** Trimmed acceptance filter for the worker's Search panel — only fields a fresh
 *  discovery actually measures (online-mode, mods, players). */
export const SEARCH_FILTER_FIELDS: FilterFieldKey[] = [
  "licensed",
  "requires_mods",
  "has_players",
];

export const STORAGE_KEY = "dashboard_filters";

export const DEFAULT_FILTERS: Filters = {
  limit: 50,
  licensed: null,
  checked: null,
  join_status: null,
  crashed: null,
  has_players: null,
  online: null,
  requires_mods: null,
  has_none_players: null,
  query: null,
};

/** Returns true when every filter is at its default value. */
export function areFiltersDefault(filters: Filters): boolean {
  const keys = Object.keys(DEFAULT_FILTERS).filter(
    (k) => k !== "limit" && k !== "query",
  ) as (keyof Omit<Filters, "limit" | "query">)[];
  const restDefault = keys.every((k) => filters[k] === null);
  const queryDefault = !filters.query;
  return restDefault && queryDefault;
}
