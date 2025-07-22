import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Channel } from "../components/ChannelList";

interface SearchState {
  // Search state
  searchQuery: string;
  debouncedSearchQuery: string;
  isSearching: boolean;

  // Actions
  setSearchQuery: (query: string) => void;
  setDebouncedSearchQuery: (query: string) => void;
  setIsSearching: (searching: boolean) => void;
  clearSearch: () => void;

  // Search operations
  searchChannels: (
    query: string,
    channelListId: number | null,
  ) => Promise<Channel[]>;

  // Debounce timer management
  debounceTimer: number | null;
  setDebounceTimer: (timer: number | null) => void;
}

export const useSearchStore = create<SearchState>((set, get) => ({
  // Initial state
  searchQuery: "",
  debouncedSearchQuery: "",
  isSearching: false,
  debounceTimer: null,

  // Basic setters
  setSearchQuery: (searchQuery) => {
    set({ searchQuery });

    // Clear existing timer
    const { debounceTimer } = get();
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    // Set new timer for debouncing
    const newTimer = setTimeout(() => {
      set({ debouncedSearchQuery: searchQuery });
    }, 400);

    set({ debounceTimer: newTimer });
  },

  setDebouncedSearchQuery: (debouncedSearchQuery) =>
    set({ debouncedSearchQuery }),
  setIsSearching: (isSearching) => set({ isSearching }),
  setDebounceTimer: (debounceTimer) => set({ debounceTimer }),

  clearSearch: () => {
    const { debounceTimer } = get();
    if (debounceTimer) {
      clearTimeout(debounceTimer);
    }

    set({
      searchQuery: "",
      debouncedSearchQuery: "",
      isSearching: false,
      debounceTimer: null,
    });
  },

  // Search operations
  searchChannels: async (query, channelListId) => {
    if (query === "" || query.length < 3) {
      const fetchedChannels = await invoke<Channel[]>("get_channels", {
        id: channelListId,
      });
      return fetchedChannels;
    } else {
      set({ isSearching: true });
      try {
        const searchedChannels = await invoke<Channel[]>("search_channels", {
          query,
          id: channelListId,
        });
        return searchedChannels;
      } catch (error) {
        // Handle search cancellation gracefully
        if (
          error &&
          typeof error === "string" &&
          error.includes("Search cancelled")
        ) {
          console.log("Search was cancelled - ignoring result");
          // Return empty array for cancelled searches instead of fallback
          // This prevents showing stale results
          return [];
        }

        console.error("Search failed:", error);

        // For other errors, fall back to returning all channels
        const fetchedChannels = await invoke<Channel[]>("get_channels", {
          id: channelListId,
        });
        return fetchedChannels;
      } finally {
        set({ isSearching: false });
      }
    }
  },
}));
