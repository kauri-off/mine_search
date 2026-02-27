import { useState, useCallback } from "react";
import { useInfiniteQuery, useMutation, useQuery } from "@tanstack/react-query";
import type { InfiniteData } from "@tanstack/react-query";
import type { ServerInfoResponse } from "@/types";
import { serverApi } from "@/api/client";
import { useIntersectionRef } from "@/hooks/useIntersectionRef";
import {
  areFiltersDefault,
  DEFAULT_FILTERS,
  type Filters,
} from "@/constants/dashboardFilters";
import { clearFilters, loadFilters, saveFilters } from "@/utils/filterStorage";
import { FilterBar } from "@/components/dashboard/FilterBar";
import { AddIpForm } from "@/components/dashboard/AddIpForm";
import { ServerGrid } from "@/components/dashboard/ServerGrid";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";
import { useTranslation } from "@/i18n";

export const Dashboard = () => {
  const { t } = useTranslation();
  const [ip, setIp] = useState("");
  const [filters, setFilters] = useState<Filters>(loadFilters);

  // -- Queries ---------------------------------------------------------------

  const { data: stats } = useQuery({
    queryKey: ["stats"],
    queryFn: serverApi.fetchStats,
  });

  const { data, fetchNextPage, hasNextPage, isFetchingNextPage, isLoading } =
    useInfiniteQuery<
      ServerInfoResponse[],
      Error,
      InfiniteData<ServerInfoResponse[]>,
      [string, Filters],
      number | null
    >({
      queryKey: ["servers", filters],
      queryFn: ({ pageParam = null }) =>
        serverApi.fetchServerList({ ...filters, offset_id: pageParam }),
      getNextPageParam: (lastPage) => {
        if (!lastPage || lastPage.length < filters.limit) return undefined;
        return lastPage.at(-1)!.id;
      },
      initialPageParam: null,
    });

  // -- Mutations -------------------------------------------------------------

  const addIpMutation = useMutation({
    mutationFn: (ip: string) => serverApi.addServerIp({ ip }),
    onSuccess: () => setIp(""),
    onError: (err) => console.error(err),
  });

  // -- Infinite scroll -------------------------------------------------------

  const onEndReached = useCallback(() => {
    if (hasNextPage) fetchNextPage();
  }, [hasNextPage, fetchNextPage]);

  const lastServerRef = useIntersectionRef(
    onEndReached,
    !isLoading && !isFetchingNextPage,
  );

  // -- Helpers ---------------------------------------------------------------

  const setFilter = (field: keyof Filters, value: boolean | null) => {
    setFilters((prev) => {
      const next = { ...prev, [field]: value };
      saveFilters(next);
      return next;
    });
  };

  const resetFilters = () => {
    clearFilters();
    setFilters(DEFAULT_FILTERS);
  };

  // -- Derived state ---------------------------------------------------------

  const allServers = data?.pages.flat() ?? [];
  const isEmpty = data?.pages[0]?.length === 0;
  const filtersActive = !areFiltersDefault(filters);

  // -- Render ----------------------------------------------------------------

  return (
    <div className="p-6 max-w-7xl mx-auto text-white">
      <header className="mb-8 flex justify-between items-center">
        <h1 className="text-3xl font-bold">{t.dashboard.title}</h1>
        <div className="flex items-center gap-3">
          {stats && (
            <div className="flex gap-4 text-sm bg-gray-800 p-3 rounded-lg">
              <span>
                {t.dashboard.all}:{" "}
                <b className="text-blue-400">{stats.total_servers}</b>
              </span>
              <span>
                {t.dashboard.cracked}:{" "}
                <b className="text-orange-400">{stats.cracked_servers}</b>
              </span>
            </div>
          )}
          <LanguageSwitcher />
        </div>
      </header>

      <FilterBar
        filters={filters}
        filtersActive={filtersActive}
        onFilterChange={setFilter}
        onReset={resetFilters}
      />

      <AddIpForm
        ip={ip}
        isPending={addIpMutation.isPending}
        onChange={setIp}
        onSubmit={() => addIpMutation.mutate(ip)}
      />

      <ServerGrid
        servers={allServers}
        isEmpty={isEmpty}
        isLoading={isLoading}
        isFetchingNextPage={isFetchingNextPage}
        hasNextPage={!!hasNextPage}
        lastServerRef={lastServerRef}
      />
    </div>
  );
};
