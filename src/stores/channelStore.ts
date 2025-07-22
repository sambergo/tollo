import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Channel } from "../components/ChannelList";

export interface ChannelLoadingStatus {
  progress: number; // 0.0 to 1.0
  message: string;
  channel_count?: number;
  is_complete: boolean;
}

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
  isExternalPlayerPlaying: boolean;

  // NEW: Progress tracking
  loadingProgress: ChannelLoadingStatus | null;
  isAsyncLoading: boolean;

  // Actions
  setChannels: (channels: Channel[]) => void;
  setFavorites: (favorites: Channel[]) => void;
  setGroups: (groups: string[]) => void;
  setHistory: (history: Channel[]) => void;
  setSelectedChannel: (channel: Channel | null) => void;
  setSelectedChannelListId: (id: number | null) => void;
  setIsLoadingChannelList: (loading: boolean) => void;
  setIsExternalPlayerPlaying: (playing: boolean) => void;

  // NEW: Progress actions
  setLoadingProgress: (progress: ChannelLoadingStatus | null) => void;
  setIsAsyncLoading: (loading: boolean) => void;

  // API actions
  fetchChannels: (id?: number | null) => Promise<void>;
  fetchFavorites: () => Promise<void>;
  fetchGroups: (id?: number | null) => Promise<void>;
  fetchHistory: () => Promise<void>;
  toggleFavorite: (channel: Channel) => Promise<void>;
  playInExternalPlayer: (channel: Channel) => Promise<void>;

  // NEW: Async API actions
  fetchChannelsAsync: (id?: number | null) => Promise<void>;
  fetchFavoritesAsync: () => Promise<void>;
  fetchGroupsAsync: (id?: number | null) => Promise<void>;
  fetchHistoryAsync: () => Promise<void>;
  toggleFavoriteAsync: (channel: Channel) => Promise<void>;
  playInExternalPlayerAsync: (channel: Channel) => Promise<void>;
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
  isExternalPlayerPlaying: false,
  loadingProgress: null,
  isAsyncLoading: false,

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
  setIsExternalPlayerPlaying: (isExternalPlayerPlaying) =>
    set({ isExternalPlayerPlaying }),
  setLoadingProgress: (loadingProgress) => set({ loadingProgress }),
  setIsAsyncLoading: (isAsyncLoading) => set({ isAsyncLoading }),

  // Original API actions (kept for backwards compatibility)
  fetchChannels: async (id = null) => {
    set({ isLoadingChannelList: true });
    try {
      const fetchedChannels = await invoke<Channel[]>("get_channels", { id });
      set({ channels: fetchedChannels });
    } catch (error) {
      console.error("Failed to fetch channels:", error);
    } finally {
      set({ isLoadingChannelList: false });
    }
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

  playInExternalPlayer: async (channel) => {
    set({ isExternalPlayerPlaying: true });
    try {
      await invoke("play_channel", { channel });
      // Refresh history only on successful playback
      get().fetchHistory();
      // Reset loading state after successful playback verification
      set({ isExternalPlayerPlaying: false });
    } catch (error) {
      console.error("Failed to play channel:", error);
      // Reset loading state on error
      set({ isExternalPlayerPlaying: false });
      // Notify user about the failure
      alert(
        `Failed to play channel "${channel.name}". The external player couldn't play this channel.`,
      );
    }
  },

  // NEW: Async API actions with progress tracking
  fetchChannelsAsync: async (id = null) => {
    set({ isAsyncLoading: true, loadingProgress: null });

    try {
      const fetchedChannels = await invoke<Channel[]>("get_channels_async", {
        id,
      });
      set({ channels: fetchedChannels });
    } catch (error) {
      console.error("Failed to fetch channels async:", error);
      set({
        loadingProgress: {
          progress: 0,
          message: `Error: ${error}`,
          is_complete: true,
        },
      });
    } finally {
      set({ isAsyncLoading: false });
    }
  },

  fetchFavoritesAsync: async () => {
    try {
      const fetchedFavorites = await invoke<Channel[]>("get_favorites_async");
      set({ favorites: fetchedFavorites });
    } catch (error) {
      console.error("Failed to fetch favorites async:", error);
    }
  },

  fetchGroupsAsync: async (id = null) => {
    try {
      const fetchedGroups = await invoke<string[]>("get_groups_async", { id });
      set({ groups: fetchedGroups });
    } catch (error) {
      console.error("Failed to fetch groups async:", error);
    }
  },

  fetchHistoryAsync: async () => {
    try {
      const fetchedHistory = await invoke<Channel[]>("get_history_async");
      set({ history: fetchedHistory });
    } catch (error) {
      console.error("Failed to fetch history async:", error);
    }
  },

  toggleFavoriteAsync: async (channel) => {
    const { favorites } = get();
    const isFavorite = favorites.some((fav) => fav.name === channel.name);

    try {
      if (isFavorite) {
        await invoke("remove_favorite_async", { name: channel.name });
      } else {
        await invoke("add_favorite_async", { channel });
      }

      // Refresh favorites
      await get().fetchFavoritesAsync();
    } catch (error) {
      console.error("Failed to toggle favorite async:", error);
    }
  },

  playInExternalPlayerAsync: async (channel) => {
    set({ isExternalPlayerPlaying: true });
    try {
      await invoke("play_channel", { channel });
      // Refresh history only on successful playback
      await get().fetchHistoryAsync();
      // Reset loading state after successful playback verification
      set({ isExternalPlayerPlaying: false });
    } catch (error) {
      console.error("Failed to play channel:", error);
      // Reset loading state on error
      set({ isExternalPlayerPlaying: false });
      // Notify user about the failure
      alert(
        `Failed to play channel "${channel.name}". The external player couldn't play this channel - it may be offline or geo-blocked.`,
      );
    }
  },
}));

// Setup event listeners for progress updates
// This runs once when the module is loaded
listen<ChannelLoadingStatus>("channel_loading", (event) => {
  const status = event.payload;
  console.log("Channel loading progress:", status);

  useChannelStore.getState().setLoadingProgress(status);

  // Clear progress when complete
  if (status.is_complete) {
    setTimeout(() => {
      useChannelStore.getState().setLoadingProgress(null);
    }, 2000); // Keep the "complete" message for 2 seconds
  }
});

// Listen for favorite operation updates
listen<string>("favorite_operation", (event) => {
  console.log("Favorite operation:", event.payload);
  // Could add UI feedback here if needed
});

// Listen for history/favorites loading updates
listen<string>("favorites_loading", (event) => {
  console.log("Favorites loading:", event.payload);
});

listen<string>("history_loading", (event) => {
  console.log("History loading:", event.payload);
});
