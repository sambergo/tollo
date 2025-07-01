import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Channel } from '../components/ChannelList';

export interface HistoryState {
  history: Channel[];
}

export interface HistoryActions {
  setHistory: (history: Channel[]) => void;
  fetchHistory: () => Promise<void>;
  addToHistory: (channel: Channel) => Promise<void>;
  clearHistory: () => Promise<void>;
  reset: () => void;
}

type HistoryStore = HistoryState & HistoryActions;

export const useHistoryStore = create<HistoryStore>((set, get) => ({
  // Initial state
  history: [],

  // Actions
  setHistory: (history: Channel[]) => set({ history }),
  
  fetchHistory: async () => {
    try {
      const fetchedHistory = await invoke<Channel[]>('get_history');
      set({ history: fetchedHistory });
    } catch (error) {
      console.error('Failed to fetch history:', error);
      set({ history: [] });
    }
  },
  
  addToHistory: async (channel: Channel) => {
    try {
      // Note: The backend probably handles adding to history automatically when playing
      // But we include this method for completeness
      await invoke('add_to_history', { channel });
      const { fetchHistory } = get();
      await fetchHistory();
    } catch (error) {
      console.error('Failed to add to history:', error);
    }
  },
  
  clearHistory: async () => {
    try {
      await invoke('clear_history');
      set({ history: [] });
    } catch (error) {
      console.error('Failed to clear history:', error);
    }
  },
  
  reset: () => set({
    history: [],
  }),
}));