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
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
        {Array.from({ length: 9 }).map((_, i) => (
          <SkeletonCard key={i} />
        ))}
      </div>
    );
  }

  return (
    <>
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
        {servers.map((server, index) => (
          <ServerCard
            key={server.id}
            server={server}
            cardRef={index === servers.length - 1 ? lastServerRef : undefined}
          />
        ))}

        {isEmpty && (
          <div className="col-span-full py-20 text-center text-slate-600">
            {t.serverGrid.empty}
          </div>
        )}
      </div>

      {isFetchingNextPage && (
        <div className="flex justify-center py-6">
          <Spinner className="w-6 h-6 text-indigo-500" />
        </div>
      )}

      {!hasNextPage && !isEmpty && servers.length > 0 && (
        <div className="mt-10 py-4 text-center border-t border-[#1a1a24] text-slate-600 text-sm">
          {t.serverGrid.end}
        </div>
      )}
    </>
  );
};
