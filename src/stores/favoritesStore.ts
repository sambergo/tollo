import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Channel } from '../components/ChannelList';

export interface FavoritesState {
  favorites: Channel[];
}

export interface FavoritesActions {
  setFavorites: (favorites: Channel[]) => void;
  fetchFavorites: () => Promise<void>;
  addFavorite: (channel: Channel) => Promise<void>;
  removeFavorite: (channelName: string) => Promise<void>;
  toggleFavorite: (channel: Channel) => Promise<void>;
  isFavorite: (channel: Channel) => boolean;
  reset: () => void;
}

type FavoritesStore = FavoritesState & FavoritesActions;

export const useFavoritesStore = create<FavoritesStore>((set, get) => ({
  // Initial state
  favorites: [],

  // Actions
  setFavorites: (favorites: Channel[]) => set({ favorites }),
  
  fetchFavorites: async () => {
    try {
      const fetchedFavorites = await invoke<Channel[]>('get_favorites');
      set({ favorites: fetchedFavorites });
    } catch (error) {
      console.error('Failed to fetch favorites:', error);
      set({ favorites: [] });
    }
  },
  
  addFavorite: async (channel: Channel) => {
    try {
      await invoke('add_favorite', { channel });
      const { fetchFavorites } = get();
      await fetchFavorites();
    } catch (error) {
      console.error('Failed to add favorite:', error);
    }
  },
  
  removeFavorite: async (channelName: string) => {
    try {
      await invoke('remove_favorite', { name: channelName });
      const { fetchFavorites } = get();
      await fetchFavorites();
    } catch (error) {
      console.error('Failed to remove favorite:', error);
    }
  },
  
  toggleFavorite: async (channel: Channel) => {
    const { isFavorite, addFavorite, removeFavorite } = get();
    
    if (isFavorite(channel)) {
      await removeFavorite(channel.name);
    } else {
      await addFavorite(channel);
    }
  },
  
  isFavorite: (channel: Channel) => {
    const { favorites } = get();
    return favorites.some((fav) => fav.name === channel.name);
  },
  
  reset: () => set({
    favorites: [],
  }),
}));