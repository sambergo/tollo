import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export interface SavedFilter {
  id: number;
  channel_list_id: number;
  search_query: string;
  selected_group: string | null;
  name: string;
  slot_number: number;
}

interface FiltersState {
  // State
  savedFilters: SavedFilter[];
  
  // Actions
  setSavedFilters: (filters: SavedFilter[]) => void;
  
  // Async actions
  fetchFilters: (channelListId: number | null) => Promise<void>;
  saveFilter: (
    channelListId: number,
    slotNumber: number,
    searchQuery: string,
    selectedGroup: string | null,
    name: string
  ) => Promise<boolean>;
  deleteFilter: (id: number) => Promise<boolean>;
}

const useFiltersStore = create<FiltersState>((set) => ({
  // Initial state
  savedFilters: [],
  
  // Actions
  setSavedFilters: (filters) => set({ savedFilters: filters }),
  
  // Async actions
  fetchFilters: async (channelListId) => {
    if (channelListId === null) {
      set({ savedFilters: [] });
      return;
    }
    
    try {
      const filters = await invoke<SavedFilter[]>('get_saved_filters', { channelListId });
      set({ savedFilters: filters });
    } catch (error) {
      console.error('Failed to fetch saved filters:', error);
      set({ savedFilters: [] });
    }
  },
  
  saveFilter: async (channelListId, slotNumber, searchQuery, selectedGroup, name) => {
    try {
      await invoke('save_filter', {
        channelListId,
        slotNumber,
        searchQuery,
        selectedGroup,
        name
      });
      
      // Refresh filters after saving
      const filters = await invoke<SavedFilter[]>('get_saved_filters', { channelListId });
      set({ savedFilters: filters });
      
      return true;
    } catch (error) {
      console.error('Failed to save filter:', error);
      return false;
    }
  },
  
  deleteFilter: async (id) => {
    try {
      await invoke('delete_saved_filter', { id });
      return true;
    } catch (error) {
      console.error('Failed to delete filter:', error);
      return false;
    }
  },
}));

export default useFiltersStore;