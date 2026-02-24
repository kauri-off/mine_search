import type { PlayerResponse, PlayerStatus } from "@/types";
import { CopyButton, StatusBlock } from "@/components";
import { PLAYER_STATUSES, PLAYER_STATUS_COLOR } from "@/constants/serverDetail";

interface PlayersTableProps {
  players: PlayerResponse[] | undefined;
  onUpdateStatus: (playerId: number, status: PlayerStatus) => void;
}

export const PlayersTable = ({
  players,
  onUpdateStatus,
}: PlayersTableProps) => (
  <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
    <h3 className="font-bold mb-4">Players (All)</h3>

    {players && players.length > 0 ? (
      <div className="overflow-x-auto">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b border-gray-700 text-gray-400 text-left">
              <th className="pb-2 font-medium">Name</th>
              <th className="pb-2 font-medium text-right">Status</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-700/50">
            {players.map((player) => (
              <tr key={player.id} className="group">
                <td className="py-2.5 pr-4">
                  <div className="flex items-center gap-2">
                    <span className="text-white">{player.name}</span>
                    <span className="opacity-0 group-hover:opacity-100 transition-opacity">
                      <CopyButton text={player.name} />
                    </span>
                  </div>
                </td>
                <td className="py-2.5">
                  <div className="flex items-center justify-end gap-1.5">
                    {PLAYER_STATUSES.map((status) => (
                      <StatusBlock
                        key={status}
                        label={status}
                        active={player.status === status}
                        activeColor={PLAYER_STATUS_COLOR[status]}
                        onClick={() => {
                          if (player.status !== status) {
                            onUpdateStatus(player.id, status);
                          }
                        }}
                      />
                    ))}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    ) : (
      <span className="text-gray-500">Empty</span>
    )}
  </div>
);
