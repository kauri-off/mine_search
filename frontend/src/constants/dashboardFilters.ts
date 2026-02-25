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
};

// Tri-state cycle: null → true → false → null
export const cycleTriState = (current: boolean | null): boolean | null => {
  if (current === null) return true;
  if (current === true) return false;
  return null;
};

export const TRI_STATE_LABEL: Record<string, string> = {
  null: "All",
  true: "Yes",
  false: "No",
};

/** Returns true when every tristate filter is null (i.e. nothing is active) */
export function areFiltersDefault(filters: Filters): boolean {
  const triStateKeys = Object.keys(DEFAULT_FILTERS).filter(
    (k) => k !== "limit",
  ) as (keyof Omit<Filters, "limit">)[];
  return triStateKeys.every((k) => filters[k] === null);
}