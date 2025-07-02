import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Channel } from '../components/ChannelList';

interface UserDataState {
  // State
  favorites: Channel[];
  history: Channel[];
  
  // Actions
  setFavorites: (favorites: Channel[]) => void;
  setHistory: (history: Channel[]) => void;
  
  // Async actions
  fetchFavorites: () => Promise<void>;
  fetchHistory: () => Promise<void>;
  toggleFavorite: (channel: Channel) => Promise<void>;
  addToHistory: (channel: Channel) => Promise<void>;
}

const useUserDataStore = create<UserDataState>((set, get) => ({
  // Initial state
  favorites: [],
  history: [],
  
  // Actions
  setFavorites: (favorites) => set({ favorites }),
  setHistory: (history) => set({ history }),
  
  // Async actions
  fetchFavorites: async () => {
    const fetchedFavorites = await invoke<Channel[]>('get_favorites');
    set({ favorites: fetchedFavorites });
  },
  
  fetchHistory: async () => {
    const fetchedHistory = await invoke<Channel[]>('get_history');
    set({ history: fetchedHistory });
  },
  
  toggleFavorite: async (channel) => {
    const { favorites } = get();
    const isFavorite = favorites.some((fav) => fav.name === channel.name);
    
    if (isFavorite) {
      await invoke('remove_favorite', { name: channel.name });
    } else {
      await invoke('add_favorite', { channel });
    }
    
    // Refresh favorites after toggling
    await get().fetchFavorites();
  },
  
  addToHistory: async (channel) => {
    await invoke('play_channel', { channel });
    // Refresh history after adding
    await get().fetchHistory();
  },
}));

export default useUserDataStore;