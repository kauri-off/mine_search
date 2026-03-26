import { memo } from "react";
import { Link } from "react-router-dom";
import { formatDistanceToNow } from "date-fns";
import DOMPurify from "dompurify";
import { cn } from "@/cn";
import type { ServerInfoResponse } from "@/types";
import { useTranslation } from "@/i18n";

function getPingBadgeClass(ping: bigint | null): string {
  if (ping === null) return "bg-[#1a1a24] text-slate-500";
  const ms = Number(ping);
  if (ms < 50) return "bg-green-900/40 text-green-300";
  if (ms < 100) return "bg-yellow-900/40 text-yellow-300";
  if (ms < 200) return "bg-orange-900/40 text-orange-300";
  return "bg-red-900/40 text-red-300";
}

function formatPing(ping: bigint | null, ms: string): string {
  if (ping === null) return "N/A";
  return `${Number(ping)} ${ms}`;
}

interface ServerCardProps {
  server: ServerInfoResponse;
  cardRef?: React.Ref<HTMLAnchorElement>;
}

export const SkeletonCard = () => (
  <div className="p-4 bg-[#111118] border border-[#2a2a3a] rounded-xl animate-pulse">
    <div className="flex justify-between items-start mb-3">
      <div className="flex items-center gap-2.5 min-w-0">
        <div className="w-8 h-8 rounded flex-shrink-0 bg-[#1a1a24]" />
        <div className="h-4 w-32 bg-[#1a1a24] rounded" />
      </div>
      <div className="w-2.5 h-2.5 rounded-full flex-shrink-0 mt-1 bg-[#1a1a24]" />
    </div>
    <div className="bg-[#0d0d14] rounded-lg p-2 mb-3 space-y-1.5">
      <div className="h-3 bg-[#1a1a24] rounded w-full" />
      <div className="h-3 bg-[#1a1a24] rounded w-3/4" />
    </div>
    <div className="flex justify-between items-center">
      <div className="flex gap-1.5">
        <div className="h-5 w-16 bg-[#1a1a24] rounded-md" />
        <div className="h-5 w-20 bg-[#1a1a24] rounded-md" />
      </div>
      <div className="h-3 w-16 bg-[#1a1a24] rounded" />
    </div>
  </div>
);

export const ServerCard = memo(({ server, cardRef }: ServerCardProps) => {
  const { t } = useTranslation();

  return (
    <Link
      ref={cardRef}
      to={`/server/${server.ip}`}
      className={cn(
        "block p-4 bg-[#111118] border border-[#2a2a3a] rounded-xl transition-all duration-150",
        "hover:border-indigo-500/60 hover:shadow-lg hover:shadow-indigo-950/30",
      )}
    >
      {/* Header: favicon + IP + ping + status dot */}
      <div className="flex justify-between items-start mb-3">
        <div className="flex items-center gap-2.5 min-w-0">
          {server.favicon ? (
            <img
              src={server.favicon}
              alt=""
              className="w-8 h-8 rounded flex-shrink-0"
              style={{ imageRendering: "pixelated" }}
            />
          ) : (
            <div className="w-8 h-8 rounded flex-shrink-0 bg-[#1a1a24] grid grid-cols-2 p-0.5 gap-0.5">
              {Array.from({ length: 4 }).map((_, i) => (
                <div key={i} className="rounded-sm bg-[#2a2a3a]" />
              ))}
            </div>
          )}
          <h3 className="font-mono text-sm font-semibold truncate text-slate-200">
            {server.ip}
          </h3>
        </div>
        <div className="flex items-center gap-2 flex-shrink-0 ml-2">
          <span className={cn("px-1.5 py-0.5 rounded text-xs font-mono", getPingBadgeClass(server.ping))}>
            {formatPing(server.ping, t.serverInfo.ms)}
          </span>
          <span
            className={cn(
              "w-2.5 h-2.5 rounded-full",
              server.was_online ? "bg-green-400" : "bg-slate-600",
            )}
          />
        </div>
      </div>

      {/* MOTD */}
      <div
        className="prose prose-invert prose-xs max-w-none bg-[#0d0d14] px-2.5 py-2 rounded-lg mb-3 max-h-10 overflow-hidden text-xs leading-relaxed"
        dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(server.description_html) }}
      />

      {/* Footer */}
      <div className="flex justify-between items-center gap-2">
        <div className="flex gap-1.5 flex-wrap">
          <span className="px-2 py-0.5 rounded-md text-xs bg-[#1a1a24] text-slate-400 border border-[#2a2a3a]">
            {server.version_name}
          </span>
          <span className="px-2 py-0.5 rounded-md text-xs bg-[#1a1a24] text-slate-400 border border-[#2a2a3a]">
            {server.online}/{server.max}
          </span>
          {server.is_forge && (
            <span className="px-2 py-0.5 rounded-md text-xs bg-purple-950/40 text-purple-300 border border-purple-800/30">
              Forge
            </span>
          )}
          {!server.license && (
            <span className="px-2 py-0.5 rounded-md text-xs bg-orange-950/40 text-orange-300 border border-orange-800/30">
              Cracked
            </span>
          )}
        </div>
        <span className="text-xs text-slate-600 whitespace-nowrap flex-shrink-0">
          {formatDistanceToNow(new Date(server.updated), {
            addSuffix: true,
            locale: t.dateFnsLocale,
          })}
        </span>
      </div>
    </Link>
  );
});
