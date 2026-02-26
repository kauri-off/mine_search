import type { ServerInfoResponse } from "@/types";
import { CopyButton, ToggleButton } from "@/components";
import { InfoRow } from "./InfoRow";
import type { ToggleField } from "@/constants/serverDetail";
import { formatDistanceToNow } from "date-fns";
import { enUS } from "date-fns/locale";

interface ServerInfoCardProps {
  server: ServerInfoResponse;
  pingCountdown: number | null;
  showPingSplit: boolean;
  showDeleteConfirm: boolean;
  isDeletePending: boolean;
  onToggle: (field: ToggleField) => void;
  onPingRequest: () => void;
  onPing: (withConnection: boolean) => void;
  onDeleteRequest: () => void;
  onDeleteCancel: () => void;
  onDeleteConfirm: () => void;
}

export const ServerInfoCard = ({
  server,
  pingCountdown,
  showPingSplit,
  showDeleteConfirm,
  isDeletePending,
  onToggle,
  onPingRequest,
  onPing,
  onDeleteRequest,
  onDeleteCancel,
  onDeleteConfirm,
}: ServerInfoCardProps) => (
  <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
    {/* Header: favicon + IP */}
    <div className="flex items-center gap-3 mb-2">
      {server.favicon ? (
        <img
          src={server.favicon}
          alt="Server icon"
          className="w-16 h-16 rounded flex-shrink-0"
          style={{ imageRendering: "pixelated" }}
          title="Server favicon"
        />
      ) : (
        <div className="w-16 h-16 rounded flex-shrink-0 bg-gray-700 flex items-center justify-center text-gray-500 text-2xl">
          ?
        </div>
      )}
      <div className="min-w-0">
        <div className="flex items-center gap-2">
          <h1 className="text-2xl font-bold break-all">{server.ip}</h1>
          <CopyButton text={server.ip} />
        </div>
        <p className="text-gray-400">{server.version_name}</p>
      </div>
    </div>

    {/* Info rows */}
    <div className="space-y-3">
      <InfoRow label="Status">
        <span className={server.was_online ? "text-green-400" : "text-red-400"}>
          {server.was_online ? "Online" : "Offline"}
        </span>
      </InfoRow>
      <InfoRow label="Online">
        <span className="text-gray-300">
          {server.online} / {server.max}
        </span>
      </InfoRow>
      <InfoRow label="Licensed">
        <span className="text-gray-300">{server.license ? "Yes" : "No"}</span>
      </InfoRow>
      <InfoRow label="Forge / Modded">
        {server.is_forge ? (
          <span className="text-purple-400">Yes</span>
        ) : (
          <span className="text-gray-300">No</span>
        )}
      </InfoRow>
      <InfoRow label="Last Seen">
        <span
          className="text-gray-300"
          title={new Date(server.updated).toLocaleString()}
        >
          {formatDistanceToNow(new Date(server.updated), {
            addSuffix: true,
            locale: enUS,
          })}
        </span>
      </InfoRow>
    </div>

    {/* Management toggles */}
    <div className="mt-6 space-y-2">
      <h3 className="font-semibold mb-2 text-gray-300">Management:</h3>
      <ToggleButton
        label="Checked"
        active={!!server.checked}
        onClick={() => onToggle("checked")}
      />
      <ToggleButton
        label="Spoofable"
        active={!!server.spoofable}
        onClick={() => onToggle("spoofable")}
      />
      <ToggleButton
        label="Crashed"
        active={!!server.crashed}
        onClick={() => onToggle("crashed")}
        color="red"
      />
    </div>

    {/* Ping */}
    <div className="mt-4 pt-4 border-t border-gray-700">
      {pingCountdown !== null ? (
        <button
          disabled
          className="w-full py-2 px-4 rounded font-medium bg-blue-900 text-blue-300 opacity-70 cursor-not-allowed flex items-center justify-center gap-2"
        >
          <span>📡</span>
          <span>Reloading in {pingCountdown}s...</span>
        </button>
      ) : showPingSplit ? (
        <div className="flex flex-col gap-2">
          <p className="text-xs text-gray-400 text-center">Choose ping type:</p>
          <button
            onClick={() => onPing(true)}
            className="w-full py-2 px-4 rounded font-medium transition bg-blue-900 hover:bg-blue-800 text-blue-300 hover:text-white flex items-center justify-center gap-2"
          >
            <span>🔗</span>
            <span>With Connection</span>
          </button>
          <button
            onClick={() => onPing(false)}
            className="w-full py-2 px-4 rounded font-medium transition bg-blue-950 hover:bg-blue-900 text-blue-400 hover:text-white flex items-center justify-center gap-2"
          >
            <span>📡</span>
            <span>Without Connection</span>
          </button>
        </div>
      ) : (
        <button
          onClick={onPingRequest}
          className="w-full py-2 px-4 rounded font-medium transition bg-blue-900 hover:bg-blue-800 text-blue-300 hover:text-white flex items-center justify-center gap-2"
        >
          <span>📡</span>
          <span>Ping Server</span>
        </button>
      )}
    </div>

    {/* Delete */}
    <div className="mt-4 pt-4 border-t border-gray-700">
      {showDeleteConfirm ? (
        <div className="space-y-2">
          <p className="text-sm text-red-400 text-center font-medium">
            Are you sure you want to delete{" "}
            <span className="font-bold text-white">{server.ip}</span>?
          </p>
          <p className="text-xs text-gray-500 text-center">
            This action cannot be undone.
          </p>
          <div className="flex gap-2">
            <button
              onClick={onDeleteCancel}
              disabled={isDeletePending}
              className="flex-1 py-2 px-4 rounded font-medium transition bg-gray-700 hover:bg-gray-600 text-gray-300 disabled:opacity-50"
            >
              Cancel
            </button>
            <button
              onClick={onDeleteConfirm}
              disabled={isDeletePending}
              className="flex-1 py-2 px-4 rounded font-medium transition bg-red-600 hover:bg-red-700 text-white disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isDeletePending ? "Deleting..." : "Confirm"}
            </button>
          </div>
        </div>
      ) : (
        <button
          onClick={onDeleteRequest}
          className="w-full py-2 px-4 rounded font-medium transition bg-red-900 hover:bg-red-800 text-red-300 hover:text-white flex items-center justify-center gap-2"
        >
          <span>🗑</span>
          <span>Delete Server</span>
        </button>
      )}
    </div>
  </div>
);
