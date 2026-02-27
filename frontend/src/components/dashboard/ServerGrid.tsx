import type { ServerInfoResponse } from "@/types";
import { useTranslation } from "@/i18n";
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
  const { t } = useTranslation();

  if (isLoading) {
    return (
      <div className="text-white text-center mt-20">
        <div className="animate-pulse">{t.serverGrid.loading}</div>
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
        <div className="text-center py-4 text-gray-400">
          {t.serverGrid.loading}
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
