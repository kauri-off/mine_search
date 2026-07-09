import type { PlayerStatus, JoinStatus } from "@/types";

export const PLAYER_STATUSES: PlayerStatus[] = ["None", "Regular", "Admin"];

export const PLAYER_STATUS_COLOR: Record<PlayerStatus, string> = {
  None: "gray",
  Regular: "blue",
  Admin: "amber",
};

/** Independent boolean server flags (checked / crashed) toggled one at a time. */
export const SERVER_FLAG_FIELDS = ["is_checked", "is_crashed"] as const;
export type ServerFlagField = (typeof SERVER_FLAG_FIELDS)[number];

/** The join_status enum values, in display order. */
export const JOIN_STATUSES: JoinStatus[] = [
  "Undetermined",
  "Spoofable",
  "Whitelist",
  "Password",
  "Modded",
];

export const JOIN_STATUS_COLOR: Record<JoinStatus, string> = {
  Undetermined: "gray",
  Spoofable: "green",
  Whitelist: "amber",
  Password: "blue",
  Modded: "purple",
};