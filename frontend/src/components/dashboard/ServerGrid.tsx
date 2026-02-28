import type { ServerInfoResponse } from "@/types";
import { useTranslation } from "@/i18n";
import { Spinner } from "@/components";
import { ServerCard, SkeletonCard } from "./ServerCard";

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
  const { t } = useTranslation();

  if (isLoading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {Array.from({ length: 6 }).map((_, i) => (
          <SkeletonCard key={i} />
        ))}
      </div>
    );
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
            {t.serverGrid.empty}
          </div>
        )}
      </div>

      {isFetchingNextPage && (
        <div className="flex justify-center py-4">
          <Spinner />
        </div>
      )}

      {!hasNextPage && !isEmpty && (
        <div className="mt-8 p-4 text-center border-t border-gray-800 text-gray-500 italic">
          {t.serverGrid.end}
        </div>
      )}
    </>
  );
};
