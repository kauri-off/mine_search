import { DEFAULT_FILTERS, STORAGE_KEY } from "@/constants/dashboardFilters";
import type { Filters } from "@/constants/dashboardFilters";

/**
 * Merge stored filters with the current DEFAULT_FILTERS shape.
 * - Keys in storage that no longer exist in defaults are silently dropped.
 * - Keys that exist in defaults but not in storage get their default value.
 * - `limit` is always taken from defaults (never persisted).
 */
function mergeWithDefaults(stored: Record<string, unknown>): Filters {
  const merged = { ...DEFAULT_FILTERS };

  for (const key of Object.keys(DEFAULT_FILTERS) as (keyof Filters)[]) {
    if (key === "limit") continue;
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

export function loadFilters(): Filters {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_FILTERS;
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    return mergeWithDefaults(parsed);
  } catch {
    return DEFAULT_FILTERS;
  }
}

export function saveFilters(filters: Filters): void {
  try {
    // Exclude `limit` from persistence
    const { limit: _limit, ...persistable } = filters;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(persistable));
  } catch {
    // Storage quota exceeded or blocked — silently ignore
  }
}

export function clearFilters(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch {
    // ignore
  }
}