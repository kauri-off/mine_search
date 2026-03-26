import { Pencil, Radio, Trash2, Link2, Wifi } from "lucide-react";
import { formatDistanceToNow } from "date-fns";
import { cn } from "@/cn";
import type { ServerInfoResponse, OverwriteServerRequest } from "@/types";
import { CopyButton, ToggleButton } from "@/components";
import type { ToggleField } from "@/constants/serverDetail";
import { useTranslation } from "@/i18n";
import { ServerEditForm } from "./ServerEditForm";

function getPingColor(ping: bigint | null): string {
  if (ping === null) return "text-slate-500";
  const ms = Number(ping);
  if (ms < 50) return "text-green-400";
  if (ms < 100) return "text-yellow-400";
  if (ms < 200) return "text-orange-400";
  return "text-red-400";
}

function formatPing(ping: bigint | null, ms: string): string {
  return ping === null ? "N/A" : `${Number(ping)} ${ms}`;
}

function InfoRow({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex justify-between items-center gap-2 py-1.5 border-b border-[#1a1a24] last:border-0">
      <span className="text-xs text-slate-500 flex-shrink-0">{label}</span>
      <span className="text-sm text-right">{children}</span>
    </div>
  );
}

interface ServerInfoCardProps {
  server: ServerInfoResponse;
  pingCountdown: number | null;
  showPingSplit: boolean;
  showDeleteConfirm: boolean;
  isDeletePending: boolean;
  isEditing: boolean;
  isEditPending: boolean;
  editError: string | null;
  onToggle: (field: ToggleField) => void;
  onPingRequest: () => void;
  onPing: (withConnection: boolean) => void;
  onDeleteRequest: () => void;
  onDeleteCancel: () => void;
  onDeleteConfirm: () => void;
  onEditStart: () => void;
  onEditSave: (data: OverwriteServerRequest) => void;
  onEditCancel: () => void;
}

export const ServerInfoCard = ({
  server,
  pingCountdown,
  showPingSplit,
  showDeleteConfirm,
  isDeletePending,
  isEditing,
  isEditPending,
  editError,
  onToggle,
  onPingRequest,
  onPing,
  onDeleteRequest,
  onDeleteCancel,
  onDeleteConfirm,
  onEditStart,
  onEditSave,
  onEditCancel,
}: ServerInfoCardProps) => {
  const { t } = useTranslation();

  return (
    <div className="bg-[#111118] border border-[#2a2a3a] rounded-xl p-5 space-y-4">
      {/* Header */}
      <div className="flex items-start gap-3">
        {server.favicon ? (
          <img
            src={server.favicon}
            alt=""
            className="w-14 h-14 rounded-lg flex-shrink-0"
            style={{ imageRendering: "pixelated" }}
          />
        ) : (
          <div className="w-14 h-14 rounded-lg flex-shrink-0 bg-[#1a1a24] grid grid-cols-4 p-1 gap-0.5">
            {Array.from({ length: 16 }).map((_, i) => (
              <div key={i} className="rounded-sm bg-[#2a2a3a]" />
            ))}
          </div>
        )}
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2 mb-0.5">
            <h1 className="font-mono text-base font-bold text-slate-100 break-all leading-tight">
              {server.ip}
            </h1>
            <CopyButton text={server.ip} />
          </div>
          <p className="text-xs text-slate-500 truncate">{server.version_name}</p>
          <span
            className={cn(
              "inline-block mt-1 px-2 py-0.5 rounded-md text-xs font-medium",
              server.was_online
                ? "bg-green-900/40 text-green-300"
                : "bg-[#1a1a24] text-slate-500",
            )}
          >
            {server.was_online ? t.serverInfo.statusOnline : t.serverInfo.statusOffline}
          </span>
        </div>
        {!isEditing && (
          <button
            onClick={onEditStart}
            className="flex-shrink-0 p-1.5 rounded-lg text-slate-500 hover:text-indigo-400 hover:bg-indigo-950/30 transition-colors"
            title={t.serverInfo.editServer}
          >
            <Pencil className="w-4 h-4" />
          </button>
        )}
      </div>

      {/* Info rows or edit form */}
      {isEditing ? (
        <ServerEditForm
          server={server}
          isPending={isEditPending}
          error={editError}
          onSave={onEditSave}
          onCancel={onEditCancel}
        />
      ) : (
        <div>
          <InfoRow label={t.serverInfo.onlineCount}>
            <span className="text-slate-300">
              {server.online} / {server.max}
            </span>
          </InfoRow>
          <InfoRow label={t.serverInfo.licensed}>
            <span className={server.license ? "text-blue-400" : "text-orange-400"}>
              {server.license ? t.serverInfo.yes : t.serverInfo.no}
            </span>
          </InfoRow>
          <InfoRow label={t.serverInfo.forgeModded}>
            <span className={server.is_forge ? "text-purple-400" : "text-slate-500"}>
              {server.is_forge ? t.serverInfo.yes : t.serverInfo.no}
            </span>
          </InfoRow>
          <InfoRow label={t.serverInfo.lastSeen}>
            <span
              className="text-slate-400 text-xs"
              title={new Date(server.updated).toLocaleString()}
            >
              {formatDistanceToNow(new Date(server.updated), {
                addSuffix: true,
                locale: t.dateFnsLocale,
              })}
            </span>
          </InfoRow>
          <InfoRow label={t.serverInfo.ping}>
            <span className={getPingColor(server.ping)}>
              {formatPing(server.ping, t.serverInfo.ms)}
            </span>
          </InfoRow>

          {/* Management toggles */}
          <div className="pt-3 space-y-2">
            <p className="text-xs text-slate-500 font-medium uppercase tracking-wider mb-2">
              {t.serverInfo.management}
            </p>
            <ToggleButton
              label={t.serverInfo.checked}
              active={!!server.is_checked}
              onClick={() => onToggle("is_checked")}
            />
            <ToggleButton
              label={t.serverInfo.spoofable}
              active={!!server.is_spoofable}
              onClick={() => onToggle("is_spoofable")}
            />
            <ToggleButton
              label={t.serverInfo.crashed}
              active={!!server.is_crashed}
              onClick={() => onToggle("is_crashed")}
              color="red"
            />
          </div>
        </div>
      )}

      {/* Ping section */}
      {!isEditing && (
        <div className="border-t border-[#2a2a3a] pt-3">
          {pingCountdown !== null ? (
            <button
              disabled
              className="w-full py-2 px-3 rounded-lg text-sm bg-indigo-950/40 text-indigo-400 border border-indigo-900/30 flex items-center justify-center gap-2 opacity-70 cursor-not-allowed"
            >
              <Radio className="w-4 h-4 animate-pulse" />
              {t.serverInfo.reloadingIn(pingCountdown)}
            </button>
          ) : showPingSplit ? (
            <div className="space-y-2">
              <p className="text-xs text-slate-500 text-center">{t.serverInfo.choosePingType}</p>
              <button
                onClick={() => onPing(true)}
                className="w-full py-2 px-3 rounded-lg text-sm font-medium bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 border border-indigo-600/30 flex items-center justify-center gap-2 transition-colors"
              >
                <Link2 className="w-4 h-4" />
                {t.serverInfo.withConnection}
              </button>
              <button
                onClick={() => onPing(false)}
                className="w-full py-2 px-3 rounded-lg text-sm font-medium bg-[#1a1a24] hover:bg-[#2a2a3a] text-slate-400 border border-[#2a2a3a] flex items-center justify-center gap-2 transition-colors"
              >
                <Wifi className="w-4 h-4" />
                {t.serverInfo.withoutConnection}
              </button>
            </div>
          ) : (
            <button
              onClick={onPingRequest}
              className="w-full py-2 px-3 rounded-lg text-sm font-medium bg-[#1a1a24] hover:bg-indigo-600/20 text-slate-400 hover:text-indigo-300 border border-[#2a2a3a] hover:border-indigo-600/30 flex items-center justify-center gap-2 transition-colors"
            >
              <Radio className="w-4 h-4" />
              {t.serverInfo.pingServer}
            </button>
          )}
        </div>
      )}

      {/* Delete section */}
      {!isEditing && (
        <div className="border-t border-[#2a2a3a] pt-3">
          {showDeleteConfirm ? (
            <div className="space-y-2">
              <p className="text-xs text-red-400 text-center font-medium">
                {t.serverInfo.deleteConfirm(server.ip)}
              </p>
              <p className="text-xs text-slate-600 text-center">
                {t.serverInfo.deleteWarning}
              </p>
              <div className="flex gap-2">
                <button
                  onClick={onDeleteCancel}
                  disabled={isDeletePending}
                  className="flex-1 py-1.5 rounded-lg text-xs font-medium bg-[#1a1a24] border border-[#2a2a3a] text-slate-400 hover:text-slate-200 disabled:opacity-50 transition-colors"
                >
                  {t.serverInfo.cancel}
                </button>
                <button
                  onClick={onDeleteConfirm}
                  disabled={isDeletePending}
                  className="flex-1 py-1.5 rounded-lg text-xs font-medium bg-red-600 hover:bg-red-500 text-white disabled:opacity-50 transition-colors"
                >
                  {isDeletePending ? t.serverInfo.deleting : t.serverInfo.confirm}
                </button>
              </div>
            </div>
          ) : (
            <button
              onClick={onDeleteRequest}
              className="w-full py-2 px-3 rounded-lg text-sm font-medium bg-[#1a1a24] hover:bg-red-950/40 text-slate-500 hover:text-red-400 border border-[#2a2a3a] hover:border-red-800/30 flex items-center justify-center gap-2 transition-colors"
            >
              <Trash2 className="w-4 h-4" />
              {t.serverInfo.deleteServer}
            </button>
          )}
        </div>
      )}
    </div>
  );
};
