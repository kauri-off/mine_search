import type { ServerListRequest } from "@/types";

export type Filters = Omit<ServerListRequest, "offset_id">;

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
