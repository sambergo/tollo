import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Channel } from "../components/ChannelList";

interface ChannelState {
  // Data
  channels: Channel[];
  favorites: Channel[];
  groups: string[];
  history: Channel[];
  selectedChannel: Channel | null;
  selectedChannelListId: number | null;

  // Loading states
  isLoadingChannelList: boolean;
  isMpvPlaying: boolean;

  // Actions
  setChannels: (channels: Channel[]) => void;
  setFavorites: (favorites: Channel[]) => void;
  setGroups: (groups: string[]) => void;
  setHistory: (history: Channel[]) => void;
  setSelectedChannel: (channel: Channel | null) => void;
  setSelectedChannelListId: (id: number | null) => void;
  setIsLoadingChannelList: (loading: boolean) => void;
  setIsMpvPlaying: (playing: boolean) => void;

  // API actions
  fetchChannels: (id?: number | null) => Promise<void>;
  fetchFavorites: () => Promise<void>;
  fetchGroups: (id?: number | null) => Promise<void>;
  fetchHistory: () => Promise<void>;
  toggleFavorite: (channel: Channel) => Promise<void>;
  playInMpv: (channel: Channel) => Promise<void>;
}

export const useChannelStore = create<ChannelState>((set, get) => ({
  // Initial state
  channels: [],
  favorites: [],
  groups: [],
  history: [],
  selectedChannel: null,
  selectedChannelListId: null,
  isLoadingChannelList: false,
  isMpvPlaying: false,

  // Basic setters
  setChannels: (channels) => set({ channels }),
  setFavorites: (favorites) => set({ favorites }),
  setGroups: (groups) => set({ groups }),
  setHistory: (history) => set({ history }),
  setSelectedChannel: (selectedChannel) => set({ selectedChannel }),
  setSelectedChannelListId: (selectedChannelListId) =>
    set({ selectedChannelListId }),
  setIsLoadingChannelList: (isLoadingChannelList) =>
    set({ isLoadingChannelList }),
  setIsMpvPlaying: (isMpvPlaying) => set({ isMpvPlaying }),

  // API actions
  fetchChannels: async (id = null) => {
    const fetchedChannels = await invoke<Channel[]>("get_channels", { id });
    set({ channels: fetchedChannels });
  },

  fetchFavorites: async () => {
    const fetchedFavorites = await invoke<Channel[]>("get_favorites");
    set({ favorites: fetchedFavorites });
  },

  fetchGroups: async (id = null) => {
    const fetchedGroups = await invoke<string[]>("get_groups", { id });
    set({ groups: fetchedGroups });
  },

  fetchHistory: async () => {
    const fetchedHistory = await invoke<Channel[]>("get_history");
    set({ history: fetchedHistory });
  },

  toggleFavorite: async (channel) => {
    const { favorites } = get();
    const isFavorite = favorites.some((fav) => fav.name === channel.name);

    if (isFavorite) {
      await invoke("remove_favorite", { name: channel.name });
    } else {
      await invoke("add_favorite", { channel });
    }

    // Refresh favorites
    get().fetchFavorites();
  },

  playInMpv: async (channel) => {
    set({ isMpvPlaying: true });
    await invoke("play_channel", { channel });
    // Refresh history
    get().fetchHistory();
  },
}));
