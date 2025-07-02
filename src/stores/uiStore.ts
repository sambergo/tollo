import { create } from 'zustand';
import type { Tab } from '../components/NavigationSidebar';

export enum GroupDisplayMode {
  EnabledGroups = 'enabled',
  AllGroups = 'all'
}

interface UIState {
  // Navigation state
  activeTab: Tab;
  focusedIndex: number;
  
  // Loading states
  isLoadingChannelList: boolean;
  skipSearchEffect: boolean;
  
  // Search state
  searchQuery: string;
  debouncedSearchQuery: string;
  isSearching: boolean;
  
  // Actions
  setActiveTab: (tab: Tab) => void;
  setFocusedIndex: (index: number) => void;
  setIsLoadingChannelList: (loading: boolean) => void;
  setSkipSearchEffect: (skip: boolean) => void;
  setSearchQuery: (query: string) => void;
  setDebouncedSearchQuery: (query: string) => void;
  setIsSearching: (searching: boolean) => void;
  resetUIState: () => void;
}

const useUIStore = create<UIState>((set) => ({
  // Initial state
  activeTab: 'channels',
  focusedIndex: 0,
  isLoadingChannelList: false,
  skipSearchEffect: false,
  searchQuery: '',
  debouncedSearchQuery: '',
  isSearching: false,
  
  // Actions
  setActiveTab: (tab) => set({ activeTab: tab }),
  setFocusedIndex: (index) => set({ focusedIndex: index }),
  setIsLoadingChannelList: (loading) => set({ isLoadingChannelList: loading }),
  setSkipSearchEffect: (skip) => set({ skipSearchEffect: skip }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  setDebouncedSearchQuery: (query) => set({ debouncedSearchQuery: query }),
  setIsSearching: (searching) => set({ isSearching: searching }),
  
  resetUIState: () => set({
    focusedIndex: 0,
    searchQuery: '',
    debouncedSearchQuery: '',
    isSearching: false,
  }),
}));

export default useUIStore;