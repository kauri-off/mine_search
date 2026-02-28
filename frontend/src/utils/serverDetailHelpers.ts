import type { UpdateServerRequest } from "@/types";
import {
  EXCLUSIVE_TOGGLE_FIELDS,
  type ToggleField,
} from "@/constants/serverDetail";

/**
 * Builds an UpdateServerRequest that flips `field` and resets all other
 * exclusive fields to null (the API treats them as mutually exclusive).
 */
export function buildToggleUpdate(
  serverIp: string,
  field: ToggleField,
  currentValue: boolean | null,
): UpdateServerRequest {
  const resets = Object.fromEntries(
    EXCLUSIVE_TOGGLE_FIELDS.filter((f) => f !== field).map((f) => [f, null]),
  ) as Record<ToggleField, null>;

  return { server_ip: serverIp, ...resets, [field]: !currentValue };
}

export function buildChartData(
  history: Array<{ recorded_at: string; players_online: number }>,
) {
  return history
    .map((d) => ({
      time: d.recorded_at,
      online: d.players_online,
      formattedTime: new Date(d.recorded_at).toLocaleTimeString("en-GB", {
        hour: "2-digit",
        minute: "2-digit",
      }),
    }))
    .reverse();
}