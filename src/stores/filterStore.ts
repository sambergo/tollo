import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { SavedFilter } from '../hooks/useSavedFilters';

export interface FilterState {
  savedFilters: SavedFilter[];
  isLoading: boolean;
}

export interface FilterActions {
  setSavedFilters: (filters: SavedFilter[]) => void;
  setIsLoading: (loading: boolean) => void;
  fetchSavedFilters: (channelListId: number) => Promise<void>;
  saveFilter: (channelListId: number, slotNumber: number, searchQuery: string, selectedGroup: string | null, name: string) => Promise<boolean>;
  deleteFilter: (channelListId: number, slotNumber: number) => Promise<boolean>;
  getFilter: (slotNumber: number) => SavedFilter | undefined;
  refreshFilters: (channelListId: number | null) => Promise<void>;
  reset: () => void;
}

type FilterStore = FilterState & FilterActions;

export const useFilterStore = create<FilterStore>((set, get) => ({
  // Initial state
  savedFilters: [],
  isLoading: false,

  // Actions
  setSavedFilters: (filters: SavedFilter[]) => set({ savedFilters: filters }),
  
  setIsLoading: (loading: boolean) => set({ isLoading: loading }),
  
  fetchSavedFilters: async (channelListId: number) => {
    set({ isLoading: true });
    try {
      const filters = await invoke<SavedFilter[]>('get_saved_filters', { 
        channelListId 
      });
      set({ savedFilters: filters });
    } catch (error) {
      console.error('Failed to fetch saved filters:', error);
      set({ savedFilters: [] });
    } finally {
      set({ isLoading: false });
    }
  },
  
  saveFilter: async (
    channelListId: number,
    slotNumber: number, 
    searchQuery: string, 
    selectedGroup: string | null,
    name: string
  ): Promise<boolean> => {
    try {
      await invoke('save_filter', {
        channelListId,
        slotNumber,
        searchQuery,
        selectedGroup,
        name
      });

      // Reload saved filters
      const filters = await invoke<SavedFilter[]>('get_saved_filters', { 
        channelListId 
      });
      set({ savedFilters: filters });
      return true;
    } catch (error) {
      console.error('Failed to save filter:', error);
      return false;
    }
  },
  
  deleteFilter: async (channelListId: number, slotNumber: number): Promise<boolean> => {
    try {
      await invoke('delete_saved_filter', {
        channelListId,
        slotNumber
      });

      // Reload saved filters
      const filters = await invoke<SavedFilter[]>('get_saved_filters', { 
        channelListId 
      });
      set({ savedFilters: filters });
      return true;
    } catch (error) {
      console.error('Failed to delete filter:', error);
      return false;
    }
  },
  
  getFilter: (slotNumber: number): SavedFilter | undefined => {
    const { savedFilters } = get();
    return savedFilters.find(filter => filter.slot_number === slotNumber);
  },
  
  refreshFilters: async (channelListId: number | null): Promise<void> => {
    if (channelListId === null) {
      set({ savedFilters: [] });
      return;
    }

    set({ isLoading: true });
    try {
      const filters = await invoke<SavedFilter[]>('get_saved_filters', { 
        channelListId 
      });
      set({ savedFilters: filters });
    } catch (error) {
      console.error('Failed to refresh saved filters:', error);
      set({ savedFilters: [] });
    } finally {
      set({ isLoading: false });
    }
  },
  
  reset: () => set({
    savedFilters: [],
    isLoading: false,
  }),
}));