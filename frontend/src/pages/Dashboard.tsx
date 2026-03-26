import { useState, useCallback } from "react";
import { useInfiniteQuery, useMutation } from "@tanstack/react-query";
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
import { FilterSidebar } from "@/components/dashboard/FilterSidebar";
import { AddTargetForm } from "@/components/dashboard/AddTargetForm";
import { BulkImportModal } from "@/components/dashboard/BulkImportModal";
import { ServerGrid } from "@/components/dashboard/ServerGrid";
import { useTranslation } from "@/i18n";
import { Plus, Upload, SlidersHorizontal, X } from "lucide-react";

type BoolFilterKey = keyof Omit<Filters, "limit" | "ip_contains">;

export const Dashboard = () => {
  const { t } = useTranslation();
  const [filters, setFilters] = useState<Filters>(loadFilters);
  const [showAddTarget, setShowAddTarget] = useState(false);
  const [showBulkImport, setShowBulkImport] = useState(false);
  const [showMobileFilters, setShowMobileFilters] = useState(false);
  const [ipDebounced, setIpDebounced] = useState(filters.ip_contains ?? "");
  const [ipInputTimer, setIpInputTimer] = useState<ReturnType<typeof setTimeout> | null>(null);

  // -- Queries ---------------------------------------------------------------

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

  const addTargetMutation = useMutation({
    mutationFn: (addr: string) => serverApi.addTarget({ addr, quick: true }),
    onSuccess: () => setShowAddTarget(false),
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

  const setBoolFilter = (field: BoolFilterKey, value: boolean | null) => {
    setFilters((prev) => {
      const next = { ...prev, [field]: value };
      saveFilters(next);
      return next;
    });
  };

  const handleIpChange = (value: string) => {
    setIpDebounced(value);
    if (ipInputTimer) clearTimeout(ipInputTimer);
    const timer = setTimeout(() => {
      setFilters((prev) => {
        const next = { ...prev, ip_contains: value || null };
        saveFilters(next);
        return next;
      });
    }, 400);
    setIpInputTimer(timer);
  };

  const resetFilters = () => {
    clearFilters();
    setIpDebounced("");
    setFilters(DEFAULT_FILTERS);
  };

  // -- Derived state ---------------------------------------------------------

  const allServers = data?.pages.flat() ?? [];
  const isEmpty = data?.pages[0]?.length === 0;
  const filtersActive = !areFiltersDefault(filters);
  const addTargetError = addTargetMutation.isError ? t.addIp.error : null;

  // -- Render ----------------------------------------------------------------

  const filterSidebarProps = {
    filters: { ...filters, ip_contains: ipDebounced || null },
    filtersActive,
    onBoolChange: setBoolFilter,
    onIpChange: handleIpChange,
    onReset: resetFilters,
  };

  return (
    <div className="flex flex-1 overflow-hidden">
      {/* Desktop filter sidebar */}
      <div className="hidden md:flex">
        <FilterSidebar {...filterSidebarProps} />
      </div>

      {/* Mobile filter drawer */}
      {showMobileFilters && (
        <>
          <div
            className="fixed inset-0 z-40 bg-black/60 md:hidden"
            onClick={() => setShowMobileFilters(false)}
          />
          <div className="fixed inset-y-0 left-0 z-50 md:hidden">
            <FilterSidebar
              {...filterSidebarProps}
              onClose={() => setShowMobileFilters(false)}
            />
          </div>
        </>
      )}

      {/* Main content */}
      <div className="flex-1 overflow-y-auto">
        <div className="px-3 py-3 sm:px-6 sm:py-5">
          {/* Top bar */}
          <div className="flex items-center justify-between mb-5">
            <div className="flex items-center gap-2">
              {/* Mobile filters button */}
              <button
                onClick={() => setShowMobileFilters(true)}
                className={`md:hidden flex items-center gap-1.5 px-3 py-2 rounded-lg text-sm font-medium border transition-colors ${filtersActive ? "bg-indigo-950/30 border-indigo-600/50 text-indigo-300" : "bg-[#1a1a24] border-[#2a2a3a] text-slate-400 hover:text-slate-200 hover:border-[#3a3a4a]"}`}
              >
                <SlidersHorizontal className="w-3.5 h-3.5" />
                {t.filters.label}
                {filtersActive && (
                  <span
                    className="ml-1"
                    onClick={(e) => { e.stopPropagation(); resetFilters(); }}
                  >
                    <X className="w-3 h-3" />
                  </span>
                )}
              </button>
              {allServers.length > 0 && (
                <p className="text-sm text-slate-500">
                  {t.dashboard.loaded}:{" "}
                  <span className="text-slate-300 font-medium">{allServers.length}</span>
                </p>
              )}
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => setShowBulkImport(true)}
                className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium bg-[#1a1a24] border border-[#2a2a3a] text-slate-400 hover:text-slate-200 hover:border-[#3a3a4a] transition-colors"
              >
                <Upload className="w-3.5 h-3.5" />
                <span className="hidden sm:inline">{t.dashboard.bulkImport}</span>
              </button>
              <button
                onClick={() => setShowAddTarget(true)}
                className="flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium bg-indigo-600 hover:bg-indigo-500 text-white transition-colors"
              >
                <Plus className="w-3.5 h-3.5" />
                <span className="hidden sm:inline">{t.dashboard.addTarget}</span>
              </button>
            </div>
          </div>

          <ServerGrid
            servers={allServers}
            isEmpty={isEmpty}
            isLoading={isLoading}
            isFetchingNextPage={isFetchingNextPage}
            hasNextPage={!!hasNextPage}
            lastServerRef={lastServerRef}
          />
        </div>
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
        onSubmit={(addr) => addTargetMutation.mutate(addr)}
      />
      <BulkImportModal
        isOpen={showBulkImport}
        onClose={() => setShowBulkImport(false)}
      />
    </div>
  );
};
