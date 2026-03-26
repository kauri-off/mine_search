import { useState } from "react";
import { Save, X } from "lucide-react";
import type { ServerInfoResponse, OverwriteServerRequest } from "@/types";
import { useTranslation } from "@/i18n";

interface ServerEditFormProps {
  server: ServerInfoResponse;
  isPending: boolean;
  error: string | null;
  onSave: (data: OverwriteServerRequest) => void;
  onCancel: () => void;
}

const inputClass =
  "w-full bg-[#0d0d14] border border-[#2a2a3a] rounded-lg px-3 py-1.5 text-sm text-slate-200 focus:outline-none focus:ring-1 focus:ring-indigo-500 focus:border-indigo-500 transition-colors";

const labelClass = "text-xs text-slate-500 block mb-1";

function Toggle({
  value,
  onChange,
  label,
}: {
  value: boolean;
  onChange: (v: boolean) => void;
  label: string;
}) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-xs text-slate-500">{label}</span>
      <button
        type="button"
        onClick={() => onChange(!value)}
        className={`relative w-9 h-5 rounded-full transition-colors ${value ? "bg-indigo-600" : "bg-[#2a2a3a]"}`}
      >
        <span
          className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-transform ${value ? "translate-x-4" : "translate-x-0.5"}`}
        />
      </button>
    </div>
  );
}

export const ServerEditForm = ({
  server,
  isPending,
  error,
  onSave,
  onCancel,
}: ServerEditFormProps) => {
  const { t } = useTranslation();

  const [versionName, setVersionName] = useState(server.version_name);
  const [protocol, setProtocol] = useState(String(server.protocol));
  const [isOnlineMode, setIsOnlineMode] = useState(server.license);
  const [isForge, setIsForge] = useState(server.is_forge);
  const [isOnline, setIsOnline] = useState(server.was_online);
  const [ping, setPing] = useState(
    server.ping !== null ? String(Number(server.ping)) : "",
  );
  const [favicon, setFavicon] = useState(server.favicon ?? "");

  const handleSave = () => {
    const pingNum = ping ? parseInt(ping, 10) : null;
    const body: OverwriteServerRequest = {
      server_id: server.id,
      port: null,
      version_name: versionName || null,
      protocol: protocol ? parseInt(protocol, 10) : null,
      is_online_mode: isOnlineMode,
      is_forge: isForge,
      is_online: isOnline,
      // ts-rs maps i64 → bigint but JSON serializes as number; cast is safe
      ping: (pingNum as unknown) as bigint,
      favicon: favicon || null,
      is_checked: null,
      is_spoofable: null,
      is_crashed: null,
    };
    onSave(body);
  };

  return (
    <div className="space-y-3">
      <div>
        <label className={labelClass}>{t.serverInfo.versionName}</label>
        <input
          type="text"
          value={versionName}
          onChange={(e) => setVersionName(e.target.value)}
          className={inputClass}
        />
      </div>

      <div>
        <label className={labelClass}>{t.serverInfo.protocol}</label>
        <input
          type="number"
          value={protocol}
          onChange={(e) => setProtocol(e.target.value)}
          className={inputClass}
        />
      </div>

      <div>
        <label className={labelClass}>{t.serverInfo.ping}</label>
        <input
          type="number"
          value={ping}
          onChange={(e) => setPing(e.target.value)}
          placeholder="(empty = N/A)"
          className={inputClass}
        />
      </div>

      <div>
        <label className={labelClass}>{t.serverInfo.favicon}</label>
        <input
          type="text"
          value={favicon}
          onChange={(e) => setFavicon(e.target.value)}
          placeholder="data:image/png;base64,..."
          className={inputClass}
        />
      </div>

      <div className="pt-1 space-y-2.5">
        <Toggle
          label={t.serverInfo.licensed}
          value={isOnlineMode}
          onChange={setIsOnlineMode}
        />
        <Toggle
          label={t.serverInfo.forgeModded}
          value={isForge}
          onChange={setIsForge}
        />
        <Toggle
          label={t.serverInfo.isOnline}
          value={isOnline}
          onChange={setIsOnline}
        />
      </div>

      {error && <p className="text-xs text-red-400">{error}</p>}

      <div className="flex gap-2 pt-1">
        <button
          onClick={onCancel}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-[#1a1a24] border border-[#2a2a3a] text-slate-400 hover:text-slate-200 transition-colors"
        >
          <X className="w-3 h-3" />
          {t.serverInfo.cancelEdit}
        </button>
        <button
          onClick={handleSave}
          disabled={isPending}
          className="flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-indigo-600 hover:bg-indigo-500 text-white disabled:opacity-50 transition-colors"
        >
          <Save className="w-3 h-3" />
          {isPending ? "..." : t.serverInfo.saveChanges}
        </button>
      </div>
    </div>
  );
};
