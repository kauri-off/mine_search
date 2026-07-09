import { useState, useCallback } from "react";
import { Dialog, DialogPanel } from "@headlessui/react";
import { useInfiniteQuery, useMutation } from "@tanstack/react-query";
import type { InfiniteData } from "@tanstack/react-query";
import type { ServerInfoResponse, JoinStatus } from "@/types";
import { serverApi } from "@/api/client";
import {
  areFiltersDefault,
  DEFAULT_FILTERS,
  type Filters,
} from "@/constants/dashboardFilters";
import { clearFilters, loadFilters, saveFilters } from "@/utils/filterStorage";
import { FilterSidebar } from "@/components/dashboard/FilterSidebar";
import { AddTargetForm } from "@/components/dashboard/AddTargetForm";
import { BulkImportModal } from "@/components/dashboard/BulkImportModal";
import { ServerGrid } from "@/components/dashboard/ServerGrid";
import { useTranslation } from "@/i18n";
import { Plus, Upload, SlidersHorizontal, X } from "lucide-react";

type BoolFilterKey = keyof Omit<Filters, "limit" | "query" | "join_status">;

export const Dashboard = () => {
  const { t } = useTranslation();
  const [filters, setFilters] = useState<Filters>(loadFilters);
  const [showAddTarget, setShowAddTarget] = useState(false);
  const [showBulkImport, setShowBulkImport] = useState(false);
  const [showMobileFilters, setShowMobileFilters] = useState(false);
  const [queryDebounced, setQueryDebounced] = useState(filters.query ?? "");
  const [queryInputTimer, setQueryInputTimer] = useState<ReturnType<
    typeof setTimeout
  > | null>(null);

  // -- Queries ---------------------------------------------------------------

  const {
    data,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    isLoading,
    isError,
    refetch,
  } = useInfiniteQuery<
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

  const addTargetMutation = useMutation({
    mutationFn: ({ addr, workerId }: { addr: string; workerId: string }) =>
      serverApi.addTarget({ addr, quick: true }, workerId),
    onSuccess: () => setShowAddTarget(false),
  });

  // -- Infinite scroll -------------------------------------------------------

  const onEndReached = useCallback(() => {
    if (hasNextPage) fetchNextPage();
  }, [hasNextPage, fetchNextPage]);

  // -- Helpers ---------------------------------------------------------------

  const setBoolFilter = (field: BoolFilterKey, value: boolean | null) => {
    setFilters((prev) => {
      const next = { ...prev, [field]: value };
      saveFilters(next);
      return next;
    });
  };

  const setJoinStatusFilter = (value: JoinStatus | null) => {
    setFilters((prev) => {
      const next = { ...prev, join_status: value };
      saveFilters(next);
      return next;
    });
  };

  const handleQueryChange = (value: string) => {
    setQueryDebounced(value);
    if (queryInputTimer) clearTimeout(queryInputTimer);
    const timer = setTimeout(() => {
      setFilters((prev) => {
        const next = { ...prev, query: value || null };
        saveFilters(next);
        return next;
      });
    }, 400);
    setQueryInputTimer(timer);
  };

  const resetFilters = () => {
    clearFilters();
    setQueryDebounced("");
    setFilters(DEFAULT_FILTERS);
  };

  // -- Derived state ---------------------------------------------------------

  const allServers = data?.pages.flat() ?? [];
  const isEmpty = data?.pages[0]?.length === 0;
  const filtersActive = !areFiltersDefault(filters);
  const addTargetError = addTargetMutation.isError ? t.addIp.error : null;

  // -- Render ----------------------------------------------------------------

  const filterSidebarProps = {
    filters: { ...filters, query: queryDebounced || null },
    filtersActive,
    onBoolChange: setBoolFilter,
    onJoinStatusChange: setJoinStatusFilter,
    onQueryChange: handleQueryChange,
    onReset: resetFilters,
  };

  return (
    <div className="flex flex-1 overflow-hidden">
      {/* Desktop filter sidebar */}
      <div className="hidden md:flex">
        <FilterSidebar {...filterSidebarProps} />
      </div>

      {/* Mobile filter drawer */}
      <Dialog
        open={showMobileFilters}
        onClose={() => setShowMobileFilters(false)}
        className="relative z-50 md:hidden"
      >
        <div
          className="fixed inset-0 bg-black/60 transition-opacity duration-200 data-[closed]:opacity-0"
          aria-hidden="true"
        />
        <div className="fixed inset-y-0 left-0 flex">
          <DialogPanel
            transition
            className="flex transition duration-200 ease-in-out data-[closed]:-translate-x-full"
          >
            <FilterSidebar
              {...filterSidebarProps}
              onClose={() => setShowMobileFilters(false)}
            />
          </DialogPanel>
        </div>
      </Dialog>

      {/* Main content */}
      <div className="flex-1 flex flex-col overflow-hidden min-w-0">
        <div className="px-3 pt-3 sm:px-6 sm:pt-5 flex-shrink-0">
          {/* Top bar */}
          <div className="flex items-center justify-between mb-5">
            <div className="flex items-center gap-2">
              {/* Mobile filters button */}
              <button
                onClick={() => setShowMobileFilters(true)}
                className={`md:hidden flex items-center gap-1.5 px-3 py-2 rounded-lg text-sm font-medium border transition-colors ${filtersActive ? "bg-indigo-950/30 border-indigo-600/50 text-indigo-300" : "bg-surface border-border text-slate-400 hover:text-slate-200 hover:border-border-hover"}`}
              >
                <SlidersHorizontal className="w-3.5 h-3.5" />
                {t.filters.label}
                {filtersActive && (
                  <span
                    className="ml-1"
                    onClick={(e) => {
                      e.stopPropagation();
                      resetFilters();
                    }}
                  >
                    <X className="w-3 h-3" />
                  </span>
                )}
              </button>
              {allServers.length > 0 && (
                <p className="text-sm text-slate-500">
                  {t.dashboard.loaded}:{" "}
                  <span className="text-slate-300 font-medium">
                    {allServers.length}
                  </span>
                </p>
              )}
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setShowBulkImport(true)}
                className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium bg-surface border border-border text-slate-400 hover:text-slate-200 hover:border-border-hover transition-colors"
              >
                <Upload className="w-3.5 h-3.5" />
                <span className="hidden sm:inline">
                  {t.dashboard.bulkImport}
                </span>
              </button>
              <button
                onClick={() => setShowAddTarget(true)}
                className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium bg-indigo-600 hover:bg-indigo-500 text-white transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                <span className="hidden sm:inline">
                  {t.dashboard.addTarget}
                </span>
              </button>
            </div>
          </div>
        </div>

        <ServerGrid
          servers={allServers}
          isEmpty={isEmpty}
          isLoading={isLoading}
          isError={isError}
          onRetry={refetch}
          isFetchingNextPage={isFetchingNextPage}
          hasNextPage={!!hasNextPage}
          onEndReached={onEndReached}
        />
      </div>

      {/* Modals */}
      <AddTargetForm
        isOpen={showAddTarget}
        onClose={() => {
          setShowAddTarget(false);
          addTargetMutation.reset();
        }}
        isPending={addTargetMutation.isPending}
        error={addTargetError}
        onSubmit={(addr, workerId) => addTargetMutation.mutate({ addr, workerId })}
      />
      <BulkImportModal
        isOpen={showBulkImport}
        onClose={() => setShowBulkImport(false)}
      />
    </div>
  );
};
