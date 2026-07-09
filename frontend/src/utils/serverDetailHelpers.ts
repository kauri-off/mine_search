import type { JoinStatus, UpdateServerRequest } from "@/types";
import type { ServerFlagField } from "@/constants/serverDetail";

/**
 * Builds an UpdateServerRequest that flips a single boolean flag (checked /
 * crashed). The other fields are left null, and the backend skips null columns
 * (Diesel `AsChangeset` treats `None` as "don't touch"), so each flag is
 * independent — toggling one no longer clears the others.
 */
export function buildFlagUpdate(
  serverIp: string,
  field: ServerFlagField,
  currentValue: boolean,
): UpdateServerRequest {
  return {
    server_ip: serverIp,
    is_checked: field === "is_checked" ? !currentValue : null,
    join_status: null,
    is_crashed: field === "is_crashed" ? !currentValue : null,
  };
}

/** Builds an UpdateServerRequest that sets only the join_status enum. */
export function buildJoinStatusUpdate(
  serverIp: string,
  status: JoinStatus,
): UpdateServerRequest {
  return {
    server_ip: serverIp,
    is_checked: null,
    join_status: status,
    is_crashed: null,
  };
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