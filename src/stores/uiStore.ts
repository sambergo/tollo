import { create } from 'zustand';
import type { Tab } from '../components/NavigationSidebar';
import type { Channel } from '../components/ChannelList';

export interface UIState {
  // Navigation state
  activeTab: Tab;
  focusedIndex: number;
  
  // Selection state
  selectedChannel: Channel | null;
}

export interface UIActions {
  // Navigation actions
  setActiveTab: (tab: Tab) => void;
  setFocusedIndex: (value: number | ((prev: number) => number)) => void;
  
  // Selection actions
  setSelectedChannel: (channel: Channel | null) => void;
  
  // Utility actions
  reset: () => void;
}

type UIStore = UIState & UIActions;

export const useUIStore = create<UIStore>((set) => ({
  // Initial state
  activeTab: 'channels',
  focusedIndex: 0,
  selectedChannel: null,

  // Actions
  setActiveTab: (tab: Tab) => set({ activeTab: tab }),
  
  setFocusedIndex: (value: number | ((prev: number) => number)) => {
    if (typeof value === 'function') {
      set(state => ({ focusedIndex: value(state.focusedIndex) }));
    } else {
      set({ focusedIndex: value });
    }
  },
  
  setSelectedChannel: (channel: Channel | null) => set({ selectedChannel: channel }),
  
  reset: () => set({
    activeTab: 'channels',
    focusedIndex: 0,
    selectedChannel: null,
  }),
}));