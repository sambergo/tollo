import { useEffect } from "react";
import { useSearchStore } from "../stores";

export function useChannelSearch(selectedChannelListId: number | null) {
  const {
    searchQuery,
    debouncedSearchQuery,
    isSearching,
    setDebouncedSearchQuery,
    searchChannels,
  } = useSearchStore();

  // Clear debounced query when channel list changes
  useEffect(() => {
    setDebouncedSearchQuery("");
  }, [selectedChannelListId, setDebouncedSearchQuery]);

  return {
    searchQuery,
    debouncedSearchQuery,
    isSearching,
    searchChannels: (query: string) =>
      searchChannels(query, selectedChannelListId),
  };
}
