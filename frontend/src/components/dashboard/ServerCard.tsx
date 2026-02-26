import { Link } from "react-router-dom";
import { formatDistanceToNow } from "date-fns";
import { enUS } from "date-fns/locale";
import { cn } from "@/cn";
import type { ServerInfoResponse } from "@/types";

interface ServerCardProps {
  server: ServerInfoResponse;
  cardRef?: React.Ref<HTMLAnchorElement>;
}

export const ServerCard = ({ server, cardRef }: ServerCardProps) => (
  <Link
    ref={cardRef}
    to={`/server/${server.ip}`}
    className="block p-4 bg-gray-800 hover:bg-gray-750 border border-gray-700 rounded-lg transition hover:shadow-lg hover:border-blue-500"
  >
    {/* Header: favicon + IP + online dot */}
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
      <span
        className={cn(
          "w-3 h-3 rounded-full flex-shrink-0 mt-1",
          server.was_online ? "bg-green-500" : "bg-red-500",
        )}
      />
    </div>

    <div
      className="prose prose-invert prose-sm max-w-none bg-gray-900 p-2 rounded mb-2"
      dangerouslySetInnerHTML={{ __html: server.description_html }}
    />

    <div className="flex justify-between items-center text-sm">
      <div className="flex gap-2">
        <span className="bg-gray-700 px-2 py-0.5 rounded text-white">
          {server.version_name}
        </span>
        <span className="bg-gray-700 px-2 py-0.5 rounded text-white">
          Online: {server.online}/{server.max}
        </span>
      </div>
      <span className="text-xs text-gray-500">
        {formatDistanceToNow(new Date(server.updated), {
          addSuffix: true,
          locale: enUS,
        })}
      </span>
    </div>
  </Link>
);
