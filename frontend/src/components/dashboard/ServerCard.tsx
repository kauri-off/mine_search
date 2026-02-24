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
    <div className="flex justify-between items-start mb-2">
      <h3 className="font-bold text-lg truncate">{server.ip}</h3>
      <span
        className={cn(
          "w-3 h-3 rounded-full",
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
