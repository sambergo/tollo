import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Channel } from '../components/ChannelList';

export interface ChannelState {
  // Data
  channels: Channel[];
  selectedChannelListId: number | null;
  channelListName: string;
  
  // Search state
  searchQuery: string;
  debouncedSearchQuery: string;
  isSearching: boolean;
  
  // Loading state
  isLoadingChannelList: boolean;
  skipSearchEffect: boolean;
}

export interface ChannelActions {
  // Channel list actions
  setSelectedChannelListId: (id: number | null) => void;
  setChannelListName: (name: string) => void;
  fetchChannels: (id?: number | null) => Promise<void>;
  setChannels: (channels: Channel[]) => void;
  
  // Search actions
  setSearchQuery: (query: string) => void;
  setDebouncedSearchQuery: (query: string) => void;
  setIsSearching: (searching: boolean) => void;
  searchChannels: (query: string, channelListId: number | null) => Promise<Channel[]>;
  performSearch: (query: string, channelListId: number | null) => Promise<void>;
  
  // Loading actions
  setIsLoadingChannelList: (loading: boolean) => void;
  setSkipSearchEffect: (skip: boolean) => void;
  
  // Utility actions
  clearChannels: () => void;
  reset: () => void;
}

type ChannelStore = ChannelState & ChannelActions;

export const useChannelStore = create<ChannelStore>((set, get) => ({
  // Initial state
  channels: [],
  selectedChannelListId: null,
  channelListName: '',
  searchQuery: '',
  debouncedSearchQuery: '',
  isSearching: false,
  isLoadingChannelList: false,
  skipSearchEffect: false,

  // Actions
  setSelectedChannelListId: (id) => set({ selectedChannelListId: id }),
  
  setChannelListName: (name) => set({ channelListName: name }),
  
  fetchChannels: async (id = null) => {
    try {
      const fetchedChannels = await invoke<Channel[]>('get_channels', { id });
      set({ channels: fetchedChannels });
    } catch (error) {
      console.error('Failed to fetch channels:', error);
      set({ channels: [] });
    }
  },
  
  setChannels: (channels) => set({ channels }),
  
  setSearchQuery: (query) => set({ searchQuery: query }),
  
  setDebouncedSearchQuery: (query) => set({ debouncedSearchQuery: query }),
  
  setIsSearching: (searching) => set({ isSearching: searching }),
  
  searchChannels: async (query: string, channelListId: number | null) => {
    if (query === '' || query.length < 3) {
      try {
        const fetchedChannels = await invoke<Channel[]>('get_channels', { id: channelListId });
        return fetchedChannels;
      } catch (error) {
        console.error('Failed to fetch channels:', error);
        return [];
      }
    } else {
      set({ isSearching: true });
      try {
        const searchedChannels = await invoke<Channel[]>('search_channels', { 
          query, 
          id: channelListId 
        });
        return searchedChannels;
      } catch (error) {
        console.error('Search failed:', error);
        try {
          const fetchedChannels = await invoke<Channel[]>('get_channels', { id: channelListId });
          return fetchedChannels;
        } catch (fallbackError) {
          console.error('Fallback fetch failed:', fallbackError);
          return [];
        }
      } finally {
        set({ isSearching: false });
      }
    }
  },
  
  performSearch: async (query: string, channelListId: number | null) => {
    const searchedChannels = await get().searchChannels(query, channelListId);
    set({ channels: searchedChannels });
  },
  
  setIsLoadingChannelList: (loading) => set({ isLoadingChannelList: loading }),
  
  setSkipSearchEffect: (skip) => set({ skipSearchEffect: skip }),
  
  clearChannels: () => set({ channels: [] }),
  
  reset: () => set({
    channels: [],
    selectedChannelListId: null,
    channelListName: '',
    searchQuery: '',
    debouncedSearchQuery: '',
    isSearching: false,
    isLoadingChannelList: false,
    skipSearchEffect: false,
  }),
}));