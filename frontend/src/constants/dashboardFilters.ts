import type { ServerListRequest } from "@/types";

export type Filters = Omit<ServerListRequest, "offset_id">;

export const STORAGE_KEY = "dashboard_filters";

export const DEFAULT_FILTERS: Filters = {
  limit: 50,
  licensed: null,
  checked: null,
  spoofable: null,
  crashed: null,
  has_players: null,
  online: null,
  is_forge: null,
  has_none_players: null,
  ip_contains: null,
};

/** Returns true when every filter is at its default value. */
export function areFiltersDefault(filters: Filters): boolean {
  const boolKeys = Object.keys(DEFAULT_FILTERS).filter(
    (k) => k !== "limit" && k !== "ip_contains",
  ) as (keyof Omit<Filters, "limit" | "ip_contains">)[];
  const boolsDefault = boolKeys.every((k) => filters[k] === null);
  const ipDefault = !filters.ip_contains;
  return boolsDefault && ipDefault;
}
