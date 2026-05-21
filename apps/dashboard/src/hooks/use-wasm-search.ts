import { useState, useEffect } from "react";
import { batchFuzzySearchWasm } from "@/lib/wasm-utils";

/**
 * A hook that performs fuzzy search on a list of items using Rust/WASM.
 * @param items The list of items to search
 * @param query The search query
 * @param selector A function to get the searchable string from an item
 * @param threshold Minimum score (0-1) to include an item
 */
export function useFuzzySearch<T>(
  items: T[] | undefined,
  query: string,
  selector: (item: T) => string,
  threshold: number = 0.5,
) {
  const [results, setFilteredResults] = useState<T[]>([]);

  useEffect(() => {
    if (!items) {
      setFilteredResults([]);
      return;
    }

    if (!query.trim()) {
      setFilteredResults(items);
      return;
    }

    async function performSearch() {
      if (!items) return;

      const targetStrings = items.map(selector);
      const batchResults = await batchFuzzySearchWasm(query, targetStrings, threshold);

      const filtered = batchResults.map((res) => items[res.index]);

      setFilteredResults(filtered);
    }

    performSearch();
  }, [items, query, selector, threshold]);

  return results;
}
