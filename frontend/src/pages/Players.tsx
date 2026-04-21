import { useState, useCallback } from "react";
import { useInfiniteQuery } from "@tanstack/react-query";
import type { InfiniteData } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { formatDistanceToNow } from "date-fns";
import { Search, ExternalLink } from "lucide-react";
import type { PlayerSearchResponse, PlayerStatus } from "@/types";
import { serverApi } from "@/api/client";
import { useIntersectionRef } from "@/hooks/useIntersectionRef";
import { useTranslation } from "@/i18n";
import { Spinner, StatusBlock } from "@/components";
import { PLAYER_STATUSES, PLAYER_STATUS_COLOR } from "@/constants/serverDetail";
import { cn } from "@/cn";

const LIMIT = 50;

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

  const { data, fetchNextPage, hasNextPage, isFetchingNextPage, isLoading } =
    useInfiniteQuery<
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

  const lastRowRef = useIntersectionRef(
    onEndReached,
    !isLoading && !isFetchingNextPage,
  );

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
        : "bg-[#1a1a24] border-[#2a2a3a] text-slate-400 hover:text-slate-200 hover:border-[#3a3a4a]",
    );

  const allPlayers = data?.pages.flat() ?? [];
  const isEmpty = !isLoading && allPlayers.length === 0;

  return (
    <div className="flex-1 overflow-y-auto">
      <div className="px-3 py-3 sm:px-6 sm:py-5 max-w-screen-xl mx-auto">
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
              className="w-full pl-9 pr-3 py-2 rounded-lg bg-[#1a1a24] border border-[#2a2a3a] text-sm text-slate-200 placeholder-slate-600 focus:outline-none focus:border-indigo-600/60 focus:bg-[#1e1e2c] transition-colors"
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
                  {s}
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

        {isLoading ? (
          <div className="flex justify-center py-16">
            <Spinner className="w-8 h-8 text-indigo-500" />
          </div>
        ) : isEmpty ? (
          <p className="text-sm text-slate-600 py-10 text-center">
            {t.players.empty}
          </p>
        ) : (
          <div className="bg-[#111118] border border-[#2a2a3a] rounded-xl overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full text-sm min-w-full">
                <thead>
                  <tr className="border-b border-[#2a2a3a] text-slate-500 text-left">
                    <th className="pb-2.5 pt-3 pl-5 text-xs font-medium">
                      {t.players.name}
                    </th>
                    <th className="pb-2.5 pt-3 px-3 text-xs font-medium">
                      {t.players.server}
                    </th>
                    <th className="pb-2.5 pt-3 pr-5 text-xs font-medium text-right">
                      {t.players.lastSeen}
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {allPlayers.map((player, idx) => {
                    const isLast = idx === allPlayers.length - 1;
                    return (
                      <tr
                        key={player.id}
                        ref={isLast ? lastRowRef : undefined}
                        className="border-b border-[#1a1a24] last:border-0 hover:bg-white/[0.02] transition-colors"
                      >
                        <td className="py-2.5 pl-5 pr-3">
                          <div className="flex items-center gap-2">
                            <StatusBlock
                              label={player.status}
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
                        </td>
                        <td className="py-2.5 px-3">
                          <Link
                            to={`/server/${player.server_ip}`}
                            className="text-indigo-400 hover:text-indigo-300 hover:underline text-xs transition-colors"
                          >
                            {player.server_ip}
                          </Link>
                        </td>
                        <td className="py-2.5 pr-5 text-right">
                          <span className="text-xs text-slate-500">
                            {formatDistanceToNow(
                              new Date(player.last_seen_at),
                              {
                                addSuffix: true,
                                locale: t.dateFnsLocale,
                              },
                            )}
                          </span>
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
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
        )}
      </div>
    </div>
  );
};
