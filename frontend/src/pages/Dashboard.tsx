import { useState, useCallback } from "react";
import { useInfiniteQuery, useMutation, useQuery } from "@tanstack/react-query";
import type { InfiniteData } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { formatDistanceToNow } from "date-fns";
import { enUS } from "date-fns/locale";
import type { ServerInfoResponse, ServerListRequest } from "@/types";
import { cn } from "@/cn";
import { serverApi } from "@/api/client";
import { useIntersectionRef } from "@/hooks/useIntersectionRef";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

type Filters = Omit<ServerListRequest, "offset_id">;

// Tri-state cycle: null → true → false → null
const cycleTriState = (current: boolean | null): boolean | null => {
  if (current === null) return true;
  if (current === true) return false;
  return null;
};

const TRI_STATE_LABEL: Record<string, string> = {
  null: "All",
  true: "Yes",
  false: "No",
};

// ---------------------------------------------------------------------------
// Persistent filters — localStorage with schema-migration safety
//
// Strategy: when reading from storage, we compare the stored keys against
// DEFAULT_FILTERS keys. Unknown keys (removed from codebase) are dropped;
// missing keys (added to codebase) fall back to their default values.
// The `limit` field is never persisted — it always uses the default.
// ---------------------------------------------------------------------------

const STORAGE_KEY = "dashboard_filters";

const DEFAULT_FILTERS: Filters = {
  limit: 50,
  licensed: null,
  checked: null,
  spoofable: null,
  crashed: null,
  has_players: null,
  online: null,
};

/**
 * Merge stored filters with the current DEFAULT_FILTERS shape.
 * - Keys in storage that no longer exist in defaults are silently dropped.
 * - Keys that exist in defaults but not in storage get their default value.
 * - `limit` is always taken from defaults (never persisted).
 */
function mergeWithDefaults(stored: Record<string, unknown>): Filters {
  const merged = { ...DEFAULT_FILTERS };

  for (const key of Object.keys(DEFAULT_FILTERS) as (keyof Filters)[]) {
    if (key === "limit") continue; // never persist limit
    if (key in stored) {
      const val = stored[key];
      // Only accept boolean | null — guards against corrupt data
      if (val === null || val === true || val === false) {
        (merged as Record<string, unknown>)[key] = val;
      }
    }
  }

  return merged;
}

function loadFilters(): Filters {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_FILTERS;
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    return mergeWithDefaults(parsed);
  } catch {
    return DEFAULT_FILTERS;
  }
}

function saveFilters(filters: Filters): void {
  try {
    // Exclude `limit` from persistence
    const { limit: _limit, ...persistable } = filters;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(persistable));
  } catch {
    // Storage quota exceeded or blocked — silently ignore
  }
}

function clearFilters(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch {
    // ignore
  }
}

// ---------------------------------------------------------------------------
// FilterButton
// ---------------------------------------------------------------------------

interface FilterButtonProps {
  label: string;
  value: boolean | null;
  onToggle: (next: boolean | null) => void;
}

const FilterButton = ({ label, value, onToggle }: FilterButtonProps) => (
  <button
    onClick={() => onToggle(cycleTriState(value))}
    className={cn(
      "px-3 py-1 rounded text-sm font-medium transition",
      value === true && "bg-green-600 text-white",
      value === false && "bg-red-600 text-white",
      value === null && "bg-gray-700 text-gray-300",
    )}
  >
    {label}: {TRI_STATE_LABEL[String(value)]}
  </button>
);

// ---------------------------------------------------------------------------
// ServerCard
// ---------------------------------------------------------------------------

interface ServerCardProps {
  server: ServerInfoResponse;
  cardRef?: React.Ref<HTMLAnchorElement>;
}

const ServerCard = ({ server, cardRef }: ServerCardProps) => (
  <Link
    ref={cardRef}
    to={`/server/${server.ip}`}
    className="block p-4 bg-gray-800 hover:bg-gray-750 border border-gray-700 rounded-lg transition hover:shadow-lg hover:border-blue-500"
  >
    <div className="flex justify-between items-start mb-2">
      <h3 className="font-bold text-lg truncate">{server.ip}</h3>
      <span
        className={cn(
          "w-3 h-3 rounded-full",
          server.was_online ? "bg-green-500" : "bg-red-500",
        )}
      />
    </div>

    <div
      className="prose prose-invert prose-sm max-w-none bg-gray-900 p-2 rounded mb-2"
      dangerouslySetInnerHTML={{ __html: server.description_html }}
    />

    <div className="flex justify-between items-center text-sm">
      <div className="flex gap-2">
        <span className="bg-gray-700 px-2 py-0.5 rounded text-white">
          {server.version_name}
        </span>
        <span className="bg-gray-700 px-2 py-0.5 rounded text-white">
          Online: {server.online}/{server.max}
        </span>
      </div>
      <span className="text-xs text-gray-500">
        {formatDistanceToNow(new Date(server.updated), {
          addSuffix: true,
          locale: enUS,
        })}
      </span>
    </div>
  </Link>
);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Returns true when every tristate filter is null (i.e. nothing is active) */
function areFiltersDefault(filters: Filters): boolean {
  const triStateKeys = Object.keys(DEFAULT_FILTERS).filter(
    (k) => k !== "limit",
  ) as (keyof Omit<Filters, "limit">)[];
  return triStateKeys.every((k) => filters[k] === null);
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

export const Dashboard = () => {
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

  const handleAddIp = () => addIpMutation.mutate(ip);

  // -- Render ----------------------------------------------------------------

  const allServers = data?.pages.flat() ?? [];
  const isEmpty = data?.pages[0]?.length === 0;
  const filtersActive = !areFiltersDefault(filters);

  return (
    <div className="p-6 max-w-7xl mx-auto text-white">
      {/* Header */}
      <header className="mb-8 flex justify-between items-center">
        <h1 className="text-3xl font-bold">Dashboard</h1>
        {stats && (
          <div className="flex gap-4 text-sm bg-gray-800 p-3 rounded-lg">
            <span>
              All: <b className="text-blue-400">{stats.total_servers}</b>
            </span>
            <span>
              Cracked:{" "}
              <b className="text-orange-400">{stats.cracked_servers}</b>
            </span>
          </div>
        )}
      </header>

      {/* Filters */}
      <div className="mb-6 p-4 bg-gray-800 rounded-lg flex flex-wrap gap-4 items-center">
        <span className="text-gray-400">Filters:</span>

        {(
          [
            { label: "Licensed", field: "licensed" },
            { label: "Checked", field: "checked" },
            { label: "Spoofable", field: "spoofable" },
            { label: "Crashed", field: "crashed" },
            { label: "Has Players", field: "has_players" },
            { label: "Online", field: "online" },
          ] as const
        ).map(({ label, field }) => (
          <FilterButton
            key={field}
            label={label}
            value={filters[field]}
            onToggle={(next) => setFilter(field, next)}
          />
        ))}

        {/* Reset button — only visible when at least one filter is active */}
        {filtersActive && (
          <button
            onClick={resetFilters}
            className="ml-auto px-3 py-1 rounded text-sm font-medium transition bg-gray-600 hover:bg-gray-500 text-gray-200 flex items-center gap-1.5"
            title="Reset all filters"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              className="w-3.5 h-3.5"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth={2.5}
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
              <path d="M3 3v5h5" />
            </svg>
            Reset filters
          </button>
        )}
      </div>

      {/* Add IP */}
      <div className="mb-6 p-4 bg-gray-800 rounded-lg flex gap-3 items-center">
        <span className="text-gray-400">Add IP:</span>
        <input
          type="text"
          value={ip}
          onChange={(e) => setIp(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleAddIp()}
          placeholder="e.g. 192.168.1.1"
          className="flex-1 bg-gray-700 text-white placeholder-gray-500 rounded-md px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          onClick={handleAddIp}
          disabled={addIpMutation.isPending || !ip.trim()}
          className="bg-blue-600 hover:bg-blue-500 disabled:opacity-50 disabled:cursor-not-allowed text-white text-sm font-medium px-4 py-2 rounded-md transition-colors"
        >
          {addIpMutation.isPending ? "Adding..." : "Add"}
        </button>
      </div>

      {/* Server grid */}
      {isLoading ? (
        <div className="text-center py-20">Loading...</div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {allServers.map((server, index) => (
            <ServerCard
              key={server.id}
              server={server}
              cardRef={
                index === allServers.length - 1 ? lastServerRef : undefined
              }
            />
          ))}

          {isEmpty && (
            <div className="col-span-full text-center text-gray-500">
              Server list is empty
            </div>
          )}
        </div>
      )}

      {isFetchingNextPage && (
        <div className="text-center py-4 text-gray-400">Loading...</div>
      )}

      {!hasNextPage && !isLoading && !isEmpty && (
        <div className="mt-8 p-4 text-center border-t border-gray-800 text-gray-500 italic">
          🎉 This is the end
        </div>
      )}
    </div>
  );
};
