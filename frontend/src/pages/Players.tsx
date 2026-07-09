import { useState, useCallback, useEffect, useRef } from "react";
import { useInfiniteQuery } from "@tanstack/react-query";
import type { InfiniteData } from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Link } from "react-router-dom";
import { formatDistanceToNow } from "date-fns";
import { Search, ExternalLink } from "lucide-react";
import type { PlayerSearchResponse, PlayerStatus } from "@/types";
import { serverApi } from "@/api/client";
import { useTranslation } from "@/i18n";
import { Spinner, StatusBlock } from "@/components";
import { PLAYER_STATUSES, PLAYER_STATUS_COLOR } from "@/constants/serverDetail";
import { cn } from "@/cn";

const LIMIT = 50;

/** Shared column layout for the header and the virtualized rows so they align. */
const ROW_GRID = "grid-cols-[minmax(0,2fr)_minmax(0,1fr)_auto]";

type LicenseFilter = boolean | null;

export const Players = () => {
  const { t } = useTranslation();

  const [nameInput, setNameInput] = useState("");
  const [nameDebounced, setNameDebounced] = useState("");
  const [debounceTimer, setDebounceTimer] = useState<ReturnType<
    typeof setTimeout
  > | null>(null);
  const [statusFilter, setStatusFilter] = useState<PlayerStatus | null>(null);
  const [licenseFilter, setLicenseFilter] = useState<LicenseFilter>(null);

  const {
    data,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    isLoading,
    isError,
    refetch,
  } = useInfiniteQuery<
    PlayerSearchResponse[],
    Error,
    InfiniteData<PlayerSearchResponse[]>,
    [string, string, PlayerStatus | null, LicenseFilter],
    number | null
  >({
      queryKey: ["players", nameDebounced, statusFilter, licenseFilter],
      queryFn: ({ pageParam = null }) =>
        serverApi.searchPlayers({
          limit: LIMIT,
          offset_id: pageParam,
          name_contains: nameDebounced || null,
          status: statusFilter,
          licensed: licenseFilter,
        }),
      getNextPageParam: (lastPage) => {
        if (!lastPage || lastPage.length < LIMIT) return undefined;
        return lastPage.at(-1)!.id;
      },
      initialPageParam: null,
    });

  const onEndReached = useCallback(() => {
    if (hasNextPage) fetchNextPage();
  }, [hasNextPage, fetchNextPage]);

  const handleNameChange = (value: string) => {
    setNameInput(value);
    if (debounceTimer) clearTimeout(debounceTimer);
    const timer = setTimeout(() => setNameDebounced(value), 400);
    setDebounceTimer(timer);
  };

  const filterBtnClass = (active: boolean) =>
    cn(
      "px-3 py-1.5 rounded-lg text-sm font-medium border transition-colors",
      active
        ? "bg-indigo-600/20 border-indigo-600/30 text-indigo-300"
        : "bg-surface border-border text-slate-400 hover:text-slate-200 hover:border-border-hover",
    );

  const allPlayers = data?.pages.flat() ?? [];
  const isEmpty = !isLoading && !isError && allPlayers.length === 0;

  // Virtualize the rows so an unbounded infinite-scroll list keeps only the
  // visible window mounted instead of every row.
  const scrollRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: allPlayers.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 45,
    overscan: 12,
  });
  const virtualRows = virtualizer.getVirtualItems();
  const lastIndex = virtualRows.at(-1)?.index ?? -1;
  useEffect(() => {
    if (
      lastIndex >= allPlayers.length - 1 &&
      hasNextPage &&
      !isFetchingNextPage &&
      !isLoading
    ) {
      onEndReached();
    }
  }, [lastIndex, allPlayers.length, hasNextPage, isFetchingNextPage, isLoading, onEndReached]);

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="px-3 pt-3 sm:px-6 sm:pt-5 max-w-screen-xl mx-auto w-full flex-shrink-0">
        <h1 className="text-xl font-bold text-slate-100 mb-5">
          {t.players.title}
        </h1>

        <div className="flex flex-col gap-3 mb-5">
          {/* Search input */}
          <div className="relative max-w-sm">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500 pointer-events-none" />
            <input
              type="text"
              value={nameInput}
              onChange={(e) => handleNameChange(e.target.value)}
              placeholder={t.players.searchPlaceholder}
              className="w-full pl-9 pr-3 py-2 rounded-lg bg-surface border border-border text-sm text-slate-200 placeholder-slate-600 focus:outline-none focus:border-indigo-600/60 focus:bg-[#1e1e2c] transition-colors"
            />
          </div>

          <div className="flex flex-wrap gap-4">
            {/* Status filter */}
            <div className="flex items-center gap-1.5 flex-wrap">
              <button
                onClick={() => setStatusFilter(null)}
                className={filterBtnClass(statusFilter === null)}
              >
                {t.players.statusAll}
              </button>
              {PLAYER_STATUSES.map((s) => (
                <button
                  key={s}
                  onClick={() => setStatusFilter(statusFilter === s ? null : s)}
                  className={filterBtnClass(statusFilter === s)}
                >
                  {t.playerStatus.values[s]}
                </button>
              ))}
            </div>

            {/* License filter */}
            <div className="flex items-center gap-1.5 flex-wrap">
              <button
                onClick={() => setLicenseFilter(null)}
                className={filterBtnClass(licenseFilter === null)}
              >
                {t.players.licenseAll}
              </button>
              <button
                onClick={() =>
                  setLicenseFilter(licenseFilter === true ? null : true)
                }
                className={filterBtnClass(licenseFilter === true)}
              >
                {t.players.licensed}
              </button>
              <button
                onClick={() =>
                  setLicenseFilter(licenseFilter === false ? null : false)
                }
                className={filterBtnClass(licenseFilter === false)}
              >
                {t.players.cracked}
              </button>
            </div>
          </div>
        </div>

        {allPlayers.length > 0 && (
          <p className="text-sm text-slate-500 mb-3">
            {t.players.loaded}:{" "}
            <span className="text-slate-300 font-medium">
              {allPlayers.length}
            </span>
          </p>
        )}
      </div>

      <div className="flex-1 min-h-0 px-3 sm:px-6 pb-3 max-w-screen-xl mx-auto w-full flex flex-col">
        {isLoading ? (
          <div className="flex justify-center py-16">
            <Spinner className="w-8 h-8 text-indigo-500" />
          </div>
        ) : isError ? (
          <div className="flex flex-col items-center gap-3 py-16 text-center">
            <p className="text-sm text-red-400">{t.players.error}</p>
            <button
              onClick={() => refetch()}
              className="px-4 py-2 rounded-lg text-sm font-medium bg-surface border border-border text-slate-300 hover:border-border-hover transition-colors"
            >
              {t.players.retry}
            </button>
          </div>
        ) : isEmpty ? (
          <p className="text-sm text-slate-600 py-10 text-center">
            {t.players.empty}
          </p>
        ) : (
          <div className="bg-panel border border-border rounded-xl overflow-hidden flex flex-col flex-1 min-h-0">
            {/* Header */}
            <div
              role="row"
              className={cn(
                "grid border-b border-border text-slate-500 text-xs font-medium flex-shrink-0",
                ROW_GRID,
              )}
            >
              <div className="pb-2.5 pt-3 pl-5">{t.players.name}</div>
              <div className="pb-2.5 pt-3 px-3">{t.players.server}</div>
              <div className="pb-2.5 pt-3 pr-5 text-right">
                {t.players.lastSeen}
              </div>
            </div>

            {/* Virtualized rows */}
            <div ref={scrollRef} className="overflow-y-auto flex-1">
              <div
                style={{
                  height: virtualizer.getTotalSize(),
                  position: "relative",
                  width: "100%",
                }}
              >
                {virtualRows.map((vr) => {
                  const player = allPlayers[vr.index];
                  return (
                    <div
                      key={player.id}
                      data-index={vr.index}
                      ref={virtualizer.measureElement}
                      role="row"
                      className={cn(
                        "grid items-center border-b border-surface hover:bg-white/[0.02] transition-colors",
                        ROW_GRID,
                      )}
                      style={{
                        position: "absolute",
                        top: 0,
                        left: 0,
                        width: "100%",
                        transform: `translateY(${vr.start}px)`,
                      }}
                    >
                      <div className="py-2.5 pl-5 pr-3">
                        <div className="flex items-center gap-2">
                          <StatusBlock
                            label={t.playerStatus.values[player.status]}
                            active={true}
                            activeColor={PLAYER_STATUS_COLOR[player.status]}
                          />
                          <span className="text-slate-200 font-medium">
                            {player.name}
                          </span>
                          <a
                            href={`https://namemc.com/profile/${player.name}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-slate-600 hover:text-indigo-400 transition-colors"
                            title="View on NameMC"
                          >
                            <ExternalLink className="w-3 h-3" />
                          </a>
                        </div>
                      </div>
                      <div className="py-2.5 px-3">
                        <Link
                          to={`/server/${player.server_ip}`}
                          className="text-indigo-400 hover:text-indigo-300 hover:underline text-xs transition-colors"
                        >
                          {player.server_ip}
                        </Link>
                      </div>
                      <div className="py-2.5 pr-5 text-right">
                        <span className="text-xs text-slate-500">
                          {formatDistanceToNow(new Date(player.last_seen_at), {
                            addSuffix: true,
                            locale: t.dateFnsLocale,
                          })}
                        </span>
                      </div>
                    </div>
                  );
                })}
              </div>

              {isFetchingNextPage && (
                <div className="flex justify-center py-4">
                  <Spinner className="w-5 h-5 text-indigo-500" />
                </div>
              )}
              {!hasNextPage && allPlayers.length > 0 && (
                <p className="text-xs text-slate-600 text-center py-3">
                  {t.players.end}
                </p>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
