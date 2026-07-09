import { useEffect, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { ServerInfoResponse } from "@/types";
import { useTranslation } from "@/i18n";
import { Spinner } from "@/components";
import { ServerCard, SkeletonCard } from "./ServerCard";

interface ServerGridProps {
  servers: ServerInfoResponse[];
  isEmpty: boolean;
  isLoading: boolean;
  isError: boolean;
  onRetry: () => void;
  isFetchingNextPage: boolean;
  hasNextPage: boolean;
  onEndReached: () => void;
}

/** Column count matching the old `md:grid-cols-2 xl:grid-cols-3` breakpoints,
 *  tracked via matchMedia so the row virtualizer knows how many cards per row. */
function useColumns(): number {
  const [cols, setCols] = useState(1);
  useEffect(() => {
    const xl = window.matchMedia("(min-width: 1280px)");
    const md = window.matchMedia("(min-width: 768px)");
    const update = () => setCols(xl.matches ? 3 : md.matches ? 2 : 1);
    update();
    xl.addEventListener("change", update);
    md.addEventListener("change", update);
    return () => {
      xl.removeEventListener("change", update);
      md.removeEventListener("change", update);
    };
  }, []);
  return cols;
}

export const ServerGrid = ({
  servers,
  isEmpty,
  isLoading,
  isError,
  onRetry,
  isFetchingNextPage,
  hasNextPage,
  onEndReached,
}: ServerGridProps) => {
  const { t } = useTranslation();
  const scrollRef = useRef<HTMLDivElement>(null);
  const cols = useColumns();

  const rowCount = Math.ceil(servers.length / cols);
  const virtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 160, // card height + row gap; refined by measureElement
    overscan: 6,
  });

  // Infinite scroll: fetch the next page once the last virtualized row is
  // within the overscan window (replaces the old IntersectionObserver sentinel,
  // which required every card to stay mounted).
  const virtualRows = virtualizer.getVirtualItems();
  const lastIndex = virtualRows.at(-1)?.index ?? -1;
  useEffect(() => {
    if (
      lastIndex >= rowCount - 1 &&
      hasNextPage &&
      !isFetchingNextPage &&
      !isLoading
    ) {
      onEndReached();
    }
  }, [lastIndex, rowCount, hasNextPage, isFetchingNextPage, isLoading, onEndReached]);

  if (isLoading) {
    return (
      <div className="flex-1 overflow-y-auto px-3 sm:px-6 pb-3">
        <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-3">
          {Array.from({ length: 9 }).map((_, i) => (
            <SkeletonCard key={i} />
          ))}
        </div>
      </div>
    );
  }

  if (isError) {
    return (
      <div className="flex-1 overflow-y-auto px-3 sm:px-6 pb-3">
        <div className="flex flex-col items-center gap-3 py-20 text-center">
          <p className="text-sm text-red-400">{t.serverGrid.error}</p>
          <button
            onClick={onRetry}
            className="px-4 py-2 rounded-lg text-sm font-medium bg-surface border border-border text-slate-300 hover:border-border-hover transition-colors"
          >
            {t.serverGrid.retry}
          </button>
        </div>
      </div>
    );
  }

  if (isEmpty) {
    return (
      <div className="flex-1 overflow-y-auto px-3 sm:px-6 pb-3">
        <div className="py-20 text-center text-slate-600">
          {t.serverGrid.empty}
        </div>
      </div>
    );
  }

  return (
    <div ref={scrollRef} className="flex-1 overflow-y-auto px-3 sm:px-6 pb-3">
      <div
        style={{ height: virtualizer.getTotalSize(), position: "relative", width: "100%" }}
      >
        {virtualRows.map((vr) => {
          const start = vr.index * cols;
          const rowServers = servers.slice(start, start + cols);
          return (
            <div
              key={vr.key}
              data-index={vr.index}
              ref={virtualizer.measureElement}
              className="grid gap-3 pb-3"
              style={{
                gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
                position: "absolute",
                top: 0,
                left: 0,
                width: "100%",
                transform: `translateY(${vr.start}px)`,
              }}
            >
              {rowServers.map((server) => (
                <ServerCard key={server.id} server={server} />
              ))}
            </div>
          );
        })}
      </div>

      {isFetchingNextPage && (
        <div className="flex justify-center py-6">
          <Spinner className="w-6 h-6 text-indigo-500" />
        </div>
      )}

      {!hasNextPage && servers.length > 0 && (
        <div className="mt-10 py-4 text-center border-t border-surface text-slate-600 text-sm">
          {t.serverGrid.end}
        </div>
      )}
    </div>
  );
};
