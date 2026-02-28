import { memo } from "react";
import { Link } from "react-router-dom";
import { formatDistanceToNow } from "date-fns";
import DOMPurify from "dompurify";
import { cn } from "@/cn";
import type { ServerInfoResponse } from "@/types";
import { useTranslation } from "@/i18n";

function getPingBadgeClass(ping: bigint | null): string {
  if (ping === null) return "bg-gray-700 text-gray-400";
  const ms = Number(ping);
  if (ms < 50) return "bg-green-600/30 text-green-300";
  if (ms < 100) return "bg-yellow-600/30 text-yellow-300";
  if (ms < 200) return "bg-orange-600/30 text-orange-300";
  return "bg-red-600/30 text-red-300";
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
  <div className="block p-4 bg-gray-800 border border-gray-700 rounded-lg animate-pulse">
    <div className="flex justify-between items-start mb-2">
      <div className="flex items-center gap-2 min-w-0">
        <div className="w-8 h-8 rounded flex-shrink-0 bg-gray-700" />
        <div className="h-5 w-32 bg-gray-700 rounded" />
      </div>
      <div className="w-3 h-3 rounded-full flex-shrink-0 mt-1 bg-gray-700" />
    </div>
    <div className="bg-gray-900 p-2 rounded mb-2 space-y-1.5">
      <div className="h-3 bg-gray-700 rounded w-full" />
      <div className="h-3 bg-gray-700 rounded w-3/4" />
    </div>
    <div className="flex justify-between items-center">
      <div className="flex gap-2">
        <div className="h-5 w-16 bg-gray-700 rounded" />
        <div className="h-5 w-24 bg-gray-700 rounded" />
      </div>
      <div className="h-3 w-20 bg-gray-700 rounded" />
    </div>
  </div>
);

export const ServerCard = memo(({ server, cardRef }: ServerCardProps) => {
  const { t } = useTranslation();

  return (
    <Link
      ref={cardRef}
      to={`/server/${server.ip}`}
      className="block p-4 bg-gray-800 hover:bg-gray-750 border border-gray-700 rounded-lg transition hover:shadow-lg hover:border-blue-500"
    >
      {/* Header: favicon + IP + ping badge + online dot */}
      <div className="flex justify-between items-start mb-2">
        <div className="flex items-center gap-2 min-w-0">
          {server.favicon ? (
            <img
              src={server.favicon}
              alt="Server icon"
              className="w-8 h-8 rounded flex-shrink-0 image-rendering-pixelated"
              style={{ imageRendering: "pixelated" }}
            />
          ) : (
            // Placeholder so cards without a favicon still align nicely
            <div className="w-8 h-8 rounded flex-shrink-0 bg-gray-700 flex items-center justify-center text-gray-500 text-xs">
              ?
            </div>
          )}
          <h3 className="font-bold text-lg truncate">{server.ip}</h3>
        </div>
        <div className="flex items-center gap-2 flex-shrink-0 mt-1">
          <span
            className={cn(
              "px-2 py-0.5 rounded text-xs whitespace-nowrap",
              getPingBadgeClass(server.ping),
            )}
          >
            {formatPing(server.ping, t.serverInfo.ms)}
          </span>
          <span
            className={cn(
              "w-3 h-3 rounded-full",
              server.was_online ? "bg-green-500" : "bg-red-500",
            )}
          />
        </div>
      </div>

      <div
        className="prose prose-invert prose-sm max-w-none bg-gray-900 p-2 rounded mb-2"
        dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(server.description_html) }}
      />

      <div className="flex justify-between items-center text-sm">
        <div className="flex flex-wrap gap-2">
          <span className="bg-gray-700 px-2 py-0.5 rounded text-white whitespace-nowrap">
            {server.version_name}
          </span>
          <span className="bg-gray-700 px-2 py-0.5 rounded text-white whitespace-nowrap">
            {t.serverInfo.onlineCount}: {server.online}/{server.max}
          </span>
        </div>
        <span className="text-xs text-gray-500">
          {formatDistanceToNow(new Date(server.updated), {
            addSuffix: true,
            locale: t.dateFnsLocale,
          })}
        </span>
      </div>
    </Link>
  );
});
