import { useState } from "react";
import { Trash2 } from "lucide-react";
import { formatDistanceToNow } from "date-fns";
import type { PlayerResponse, PlayerStatus } from "@/types";
import { CopyButton, StatusBlock } from "@/components";
import { PLAYER_STATUSES, PLAYER_STATUS_COLOR } from "@/constants/serverDetail";
import { useTranslation } from "@/i18n";

interface PlayersTableProps {
  players: PlayerResponse[] | undefined;
  onUpdateStatus: (playerId: number, status: PlayerStatus) => void;
  onDeletePlayer: (playerId: number) => void;
}

export const PlayersTable = ({
  players,
  onUpdateStatus,
  onDeletePlayer,
}: PlayersTableProps) => {
  const { t } = useTranslation();
  const [confirmDeleteId, setConfirmDeleteId] = useState<number | null>(null);

  return (
    <div className="bg-[#111118] border border-[#2a2a3a] rounded-xl p-5">
      <h3 className="text-sm font-semibold text-slate-300 mb-4">{t.playersTable.title}</h3>

      {players && players.length > 0 ? (
        <div className="overflow-x-auto -mx-1">
          <table className="w-full text-sm min-w-full">
            <thead>
              <tr className="border-b border-[#2a2a3a] text-slate-500 text-left">
                <th className="pb-2.5 pl-1 text-xs font-medium">{t.playersTable.name}</th>
                <th className="pb-2.5 text-xs font-medium text-right">{t.playersTable.status}</th>
                <th className="pb-2.5 pr-1 w-8" />
              </tr>
            </thead>
            <tbody>
              {players.map((player) => (
                <tr
                  key={player.id}
                  className="group border-b border-[#1a1a24] last:border-0 hover:bg-white/[0.02] transition-colors"
                >
                  <td className="py-2.5 pl-1 pr-3">
                    <div className="flex items-center gap-2">
                      <div>
                        <span className="text-slate-200">{player.name}</span>
                        <p className="text-xs text-slate-600 mt-0.5">
                          {formatDistanceToNow(new Date(player.last_seen_at), {
                            addSuffix: true,
                            locale: t.dateFnsLocale,
                          })}
                        </p>
                      </div>
                      <span>
                        <CopyButton text={player.name} />
                      </span>
                    </div>
                  </td>
                  <td className="py-2.5 px-2">
                    <div className="flex items-center justify-end gap-1">
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
                  <td className="py-2.5 pr-1">
                    {confirmDeleteId === player.id ? (
                      <div className="flex items-center gap-1">
                        <button
                          onClick={() => {
                            onDeletePlayer(player.id);
                            setConfirmDeleteId(null);
                          }}
                          className="px-1.5 py-0.5 rounded text-xs bg-red-900/40 text-red-300 hover:bg-red-600 hover:text-white transition-colors"
                        >
                          {t.playersTable.deleteYes}
                        </button>
                        <button
                          onClick={() => setConfirmDeleteId(null)}
                          className="px-1.5 py-0.5 rounded text-xs bg-[#1a1a24] text-slate-400 hover:text-slate-200 transition-colors"
                        >
                          {t.playersTable.deleteNo}
                        </button>
                      </div>
                    ) : (
                      <button
                        onClick={() => setConfirmDeleteId(player.id)}
                        className="p-1 rounded text-slate-600 hover:text-red-400 hover:bg-red-950/30 transition-all"
                        title={t.playersTable.deletePlayer}
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p className="text-sm text-slate-600">{t.playersTable.empty}</p>
      )}
    </div>
  );
};
