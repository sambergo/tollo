import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { ChannelList } from '../types/settings';

interface SettingsState {
  // Channel lists
  channelLists: ChannelList[];
  channelListName: string | null;
  
  // Player settings
  playerCommand: string;
  enablePreview: boolean;
  
  // Cache settings
  cacheDuration: number; // in hours
  
  // Actions
  setChannelLists: (lists: ChannelList[]) => void;
  setChannelListName: (name: string | null) => void;
  
  // Channel list operations
  fetchChannelLists: () => Promise<void>;
  getChannelListName: (selectedChannelListId: number | null) => Promise<string>;
  
  // Player settings actions
  setPlayerCommand: (command: string) => void;
  savePlayerCommand: () => Promise<void>;
  fetchPlayerCommand: () => Promise<void>;
  setEnablePreview: (enabled: boolean) => void;
  saveEnablePreview: () => Promise<void>;
  fetchEnablePreview: () => Promise<void>;
  
  // Cache settings actions
  setCacheDuration: (duration: number) => void;
  saveCacheDuration: () => Promise<void>;
  fetchCacheDuration: () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  // Initial state
  channelLists: [],
  channelListName: null,
  playerCommand: "",
  enablePreview: true,
  cacheDuration: 24,
  
  // Basic setters
  setChannelLists: (channelLists) => set({ channelLists }),
  setChannelListName: (channelListName) => set({ channelListName }),
  setPlayerCommand: (playerCommand) => set({ playerCommand }),
  setEnablePreview: (enablePreview) => set({ enablePreview }),
  setCacheDuration: (cacheDuration) => set({ cacheDuration }),
  
  // Channel list operations
  fetchChannelLists: async () => {
    try {
      const lists = await invoke<ChannelList[]>("get_channel_lists");
      set({ channelLists: lists });
    } catch (error) {
      console.error("Failed to fetch channel lists:", error);
      set({ channelLists: [] });
    }
  },
  
  getChannelListName: async (selectedChannelListId) => {
    if (selectedChannelListId === null) {
      set({ channelListName: null });
      return "";
    }
    
    const { channelLists } = get();
    
    // First check if we already have the lists loaded
    if (channelLists.length > 0) {
      const found = channelLists.find((l) => l.id === selectedChannelListId);
      const name = found ? found.name : "";
      set({ channelListName: name });
      return name;
    }
   
    // If not loaded, fetch them
    try {
      const lists = await invoke<ChannelList[]>("get_channel_lists");
      set({ channelLists: lists });
      const found = lists.find((l) => l.id === selectedChannelListId);
      const name = found ? found.name : "";
      set({ channelListName: name });
      return name;
    } catch (error) {
      console.error("Failed to fetch channel lists:", error);
      set({ channelListName: null });
      return "";
    }
  },
  
  // Player settings actions
  savePlayerCommand: async () => {
    const { playerCommand } = get();
    await invoke("set_player_command", { command: playerCommand });
  },
  
  fetchPlayerCommand: async () => {
    const fetchedCommand = await invoke<string>("get_player_command");
    set({ playerCommand: fetchedCommand });
  },
  
  saveEnablePreview: async () => {
    const { enablePreview } = get();
    await invoke("set_enable_preview", { enabled: enablePreview });
  },
  
  fetchEnablePreview: async () => {
    const enabled = await invoke<boolean>("get_enable_preview");
    set({ enablePreview: enabled });
  },
  
  // Cache settings actions
  saveCacheDuration: async () => {
    const { cacheDuration } = get();
    await invoke("set_cache_duration", { hours: cacheDuration });
  },
  
  fetchCacheDuration: async () => {
    const duration = await invoke<number>("get_cache_duration");
    set({ cacheDuration: duration });
  },
})); 