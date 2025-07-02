import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

export interface SavedFilter {
  slot_number: number;
  search_query: string;
  selected_group: string | null;
  name: string;
}

interface FilterState {
  // Saved filters state
  savedFilters: SavedFilter[];
  isLoading: boolean;

  // Actions
  setSavedFilters: (filters: SavedFilter[]) => void;
  setIsLoading: (loading: boolean) => void;

  // Filter operations
  loadSavedFilters: (channelListId: number | null) => Promise<void>;
  saveFilter: (
    channelListId: number,
    slotNumber: number,
    searchQuery: string,
    selectedGroup: string | null,
    name: string,
  ) => Promise<boolean>;
  deleteFilter: (channelListId: number, slotNumber: number) => Promise<boolean>;
  getFilter: (slotNumber: number) => SavedFilter | undefined;
  refreshFilters: (channelListId: number | null) => Promise<void>;
}

export const useFilterStore = create<FilterState>((set, get) => ({
  // Initial state
  savedFilters: [],
  isLoading: false,

  // Basic setters
  setSavedFilters: (savedFilters) => set({ savedFilters }),
  setIsLoading: (isLoading) => set({ isLoading }),

  // Filter operations
  loadSavedFilters: async (channelListId) => {
    if (channelListId === null) {
      set({ savedFilters: [] });
      return;
    }

    set({ isLoading: true });
    try {
      const filters = await invoke<SavedFilter[]>("get_saved_filters", {
        channelListId,
      });
      set({ savedFilters: filters });
    } catch (error) {
      console.error("Failed to load saved filters:", error);
      set({ savedFilters: [] });
    } finally {
      set({ isLoading: false });
    }
  },

  saveFilter: async (
    channelListId,
    slotNumber,
    searchQuery,
    selectedGroup,
    name,
  ) => {
    try {
      await invoke("save_filter", {
        channelListId,
        slotNumber,
        searchQuery,
        selectedGroup,
        name,
      });

      // Reload saved filters
      await get().loadSavedFilters(channelListId);
      return true;
    } catch (error) {
      console.error("Failed to save filter:", error);
      return false;
    }
  },

  deleteFilter: async (channelListId, slotNumber) => {
    try {
      await invoke("delete_saved_filter", {
        channelListId,
        slotNumber,
      });

      // Reload saved filters
      await get().loadSavedFilters(channelListId);
      return true;
    } catch (error) {
      console.error("Failed to delete filter:", error);
      return false;
    }
  },

  getFilter: (slotNumber) => {
    const { savedFilters } = get();
    return savedFilters.find((filter) => filter.slot_number === slotNumber);
  },

  refreshFilters: async (channelListId) => {
    await get().loadSavedFilters(channelListId);
  },
}));
