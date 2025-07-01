import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

export enum GroupDisplayMode {
  EnabledGroups = 'enabled',
  AllGroups = 'all'
}

export interface GroupState {
  // Data
  groups: string[];
  enabledGroups: Set<string>;
  selectedGroup: string | null;
  
  // Display state
  groupDisplayMode: GroupDisplayMode;
}

export interface GroupActions {
  // Group data actions
  setGroups: (groups: string[]) => void;
  setEnabledGroups: (groups: Set<string>) => void;
  setSelectedGroup: (group: string | null) => void;
  fetchGroups: (id?: number | null) => Promise<void>;
  fetchEnabledGroups: (channelListId: number) => Promise<string[]>;
  
  // Group management actions
  toggleGroupEnabled: (groupName: string, channelListId: number) => Promise<void>;
  selectAllGroups: (channelListId: number) => Promise<void>;
  unselectAllGroups: (channelListId: number) => Promise<void>;
  syncGroupsForChannelList: (channelListId: number, allGroups: string[]) => Promise<void>;
  
  // Display mode actions
  setGroupDisplayMode: (mode: GroupDisplayMode) => void;
  clearGroupFilter: () => void;
  
  // Utility actions
  reset: () => void;
}

type GroupStore = GroupState & GroupActions;

export const useGroupStore = create<GroupStore>((set, get) => ({
  // Initial state
  groups: [],
  enabledGroups: new Set(),
  selectedGroup: null,
  groupDisplayMode: GroupDisplayMode.EnabledGroups,

  // Actions
  setGroups: (groups: string[]) => set({ groups }),
  
  setEnabledGroups: (groups: Set<string>) => set({ enabledGroups: groups }),
  
  setSelectedGroup: (group: string | null) => set({ selectedGroup: group }),
  
  fetchGroups: async (id = null) => {
    try {
      const fetchedGroups = await invoke<string[]>('get_groups', { id });
      set({ groups: fetchedGroups });
    } catch (error) {
      console.error('Failed to fetch groups:', error);
      set({ groups: [] });
    }
  },
  
  fetchEnabledGroups: async (channelListId: number) => {
    try {
      const fetchedEnabledGroups = await invoke<string[]>('get_enabled_groups', { channelListId });
      set({ enabledGroups: new Set(fetchedEnabledGroups) });
      return fetchedEnabledGroups;
    } catch (error) {
      console.error('Failed to fetch enabled groups:', error);
      set({ enabledGroups: new Set() });
      return [];
    }
  },
  
  toggleGroupEnabled: async (groupName: string, channelListId: number) => {
    const { enabledGroups } = get();
    const newEnabledState = !enabledGroups.has(groupName);
    
    try {
      // Update database
      await invoke('update_group_selection', {
        channelListId,
        groupName,
        enabled: newEnabledState
      });
      
      // Update local state
      const newEnabledGroups = new Set(enabledGroups);
      if (newEnabledState) {
        newEnabledGroups.add(groupName);
      } else {
        newEnabledGroups.delete(groupName);
      }
      set({ enabledGroups: newEnabledGroups });
    } catch (error) {
      console.error('Failed to toggle group enabled state:', error);
    }
  },
  
  selectAllGroups: async (channelListId: number) => {
    const { groups, enabledGroups } = get();
    
    try {
      // Enable all groups that aren't already enabled
      for (const group of groups) {
        if (!enabledGroups.has(group)) {
          await invoke('update_group_selection', {
            channelListId,
            groupName: group,
            enabled: true
          });
        }
      }
      
      // Update local state to include all groups
      set({ enabledGroups: new Set(groups) });
    } catch (error) {
      console.error('Failed to select all groups:', error);
    }
  },
  
  unselectAllGroups: async (channelListId: number) => {
    const { groups, enabledGroups } = get();
    
    try {
      // Disable all groups that are currently enabled
      for (const group of groups) {
        if (enabledGroups.has(group)) {
          await invoke('update_group_selection', {
            channelListId,
            groupName: group,
            enabled: false
          });
        }
      }
      
      // Update local state to empty set
      set({ enabledGroups: new Set() });
    } catch (error) {
      console.error('Failed to unselect all groups:', error);
    }
  },
  
  syncGroupsForChannelList: async (channelListId: number, allGroups: string[]) => {
    try {
      await invoke('sync_channel_list_groups', { channelListId, groups: allGroups });
    } catch (error) {
      console.error('Failed to sync groups for channel list:', error);
    }
  },
  
  setGroupDisplayMode: (mode: GroupDisplayMode) => {
    set({ 
      groupDisplayMode: mode,
      selectedGroup: null // Reset selection when changing modes
    });
  },
  
  clearGroupFilter: () => {
    set({
      selectedGroup: null,
      groupDisplayMode: GroupDisplayMode.EnabledGroups
    });
  },
  
  reset: () => set({
    groups: [],
    enabledGroups: new Set(),
    selectedGroup: null,
    groupDisplayMode: GroupDisplayMode.EnabledGroups,
  }),
}));