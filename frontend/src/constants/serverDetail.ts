import type { PlayerStatus } from "@/types";

export const PLAYER_STATUSES: PlayerStatus[] = ["None", "Regular", "Admin"];

export const PLAYER_STATUS_COLOR: Record<PlayerStatus, string> = {
  None: "gray",
  Regular: "blue",
  Admin: "amber",
};

/** Fields that are mutually exclusive when toggling server flags. */
export const EXCLUSIVE_TOGGLE_FIELDS = ["is_checked", "is_spoofable", "is_crashed"] as const;
export type ToggleField = (typeof EXCLUSIVE_TOGGLE_FIELDS)[number];