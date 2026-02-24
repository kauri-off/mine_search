import type { ServerInfoResponse } from "@/types";
import { ServerCard } from "./ServerCard";

interface ServerGridProps {
  servers: ServerInfoResponse[];
  isEmpty: boolean;
  isLoading: boolean;
  isFetchingNextPage: boolean;
  hasNextPage: boolean;
  lastServerRef: React.Ref<HTMLAnchorElement>;
}

export const ServerGrid = ({
  servers,
  isEmpty,
  isLoading,
  isFetchingNextPage,
  hasNextPage,
  lastServerRef,
}: ServerGridProps) => {
  if (isLoading) {
    return <div className="text-center py-20">Loading...</div>;
  }

  return (
    <>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {servers.map((server, index) => (
          <ServerCard
            key={server.id}
            server={server}
            cardRef={index === servers.length - 1 ? lastServerRef : undefined}
          />
        ))}

        {isEmpty && (
          <div className="col-span-full text-center text-gray-500">
            Server list is empty
          </div>
        )}
      </div>

      {isFetchingNextPage && (
        <div className="text-center py-4 text-gray-400">Loading...</div>
      )}

      {!hasNextPage && !isEmpty && (
        <div className="mt-8 p-4 text-center border-t border-gray-800 text-gray-500 italic">
          🎉 This is the end
        </div>
      )}
    </>
  );
};
