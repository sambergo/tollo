import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Channel } from '../components/ChannelList';

interface ChannelList {
  id: number;
  name: string;
  source: string;
  is_default: boolean;
  last_fetched: number | null;
}

interface ChannelState {
  // State
  channels: Channel[];
  selectedChannel: Channel | null;
  selectedChannelListId: number | null;
  channelLists: ChannelList[];
  filteredChannels: Channel[];
  
  // Actions
  setChannels: (channels: Channel[]) => void;
  setSelectedChannel: (channel: Channel | null) => void;
  setSelectedChannelListId: (id: number | null) => void;
  setChannelLists: (lists: ChannelList[]) => void;
  setFilteredChannels: (channels: Channel[]) => void;
  
  // Async actions
  fetchChannels: (id?: number | null) => Promise<void>;
  fetchChannelLists: () => Promise<void>;
  loadDefaultChannelList: () => Promise<void>;
  syncGroupsForChannelList: (channelListId: number, allGroups: string[]) => Promise<void>;
}

const useChannelStore = create<ChannelState>((set, get) => ({
  // Initial state
  channels: [],
  selectedChannel: null,
  selectedChannelListId: null,
  channelLists: [],
  filteredChannels: [],
  
  // Actions
  setChannels: (channels) => set({ channels }),
  setSelectedChannel: (channel) => set({ selectedChannel: channel }),
  setSelectedChannelListId: (id) => set({ selectedChannelListId: id }),
  setChannelLists: (lists) => set({ channelLists: lists }),
  setFilteredChannels: (channels) => set({ filteredChannels: channels }),
  
  // Async actions
  fetchChannels: async (id = null) => {
    const fetchedChannels = await invoke<Channel[]>('get_channels', { id });
    set({ channels: fetchedChannels });
  },
  
  fetchChannelLists: async () => {
    const lists = await invoke<ChannelList[]>('get_channel_lists');
    set({ channelLists: lists });
  },
  
  loadDefaultChannelList: async () => {
    const lists = await invoke<ChannelList[]>('get_channel_lists');
    const defaultList = lists.find(list => list.is_default);
    if (defaultList && get().selectedChannelListId === null) {
      set({ selectedChannelListId: defaultList.id });
    }
  },
  
  syncGroupsForChannelList: async (channelListId, allGroups) => {
    await invoke('sync_channel_list_groups', { channelListId, groups: allGroups });
  },
}));

export default useChannelStore;