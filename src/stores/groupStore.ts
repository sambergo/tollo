import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { GroupDisplayMode } from './uiStore';

interface GroupState {
  // State
  groups: string[];
  selectedGroup: string | null;
  enabledGroups: Set<string>;
  groupDisplayMode: GroupDisplayMode;
  
  // Actions
  setGroups: (groups: string[]) => void;
  setSelectedGroup: (group: string | null) => void;
  setEnabledGroups: (groups: Set<string>) => void;
  setGroupDisplayMode: (mode: GroupDisplayMode) => void;
  
  // Complex actions
  toggleGroupEnabled: (groupName: string) => void;
  selectAllGroups: () => void;
  unselectAllGroups: () => void;
  clearGroupFilter: () => void;
  
  // Async actions
  fetchGroups: (id?: number | null) => Promise<void>;
  fetchEnabledGroups: (channelListId: number) => Promise<string[]>;
  updateGroupSelection: (channelListId: number, groupName: string, enabled: boolean) => Promise<void>;
  enableAllGroups: (channelListId: number) => Promise<void>;
  disableAllGroups: (channelListId: number) => Promise<void>;
}

const useGroupStore = create<GroupState>((set, get) => ({
  // Initial state
  groups: [],
  selectedGroup: null,
  enabledGroups: new Set(),
  groupDisplayMode: GroupDisplayMode.EnabledGroups,
  
  // Actions
  setGroups: (groups) => set({ groups }),
  setSelectedGroup: (group) => set({ selectedGroup: group }),
  setEnabledGroups: (groups) => set({ enabledGroups: groups }),
  setGroupDisplayMode: (mode) => set({ groupDisplayMode: mode }),
  
  // Complex actions
  toggleGroupEnabled: (groupName) => {
    const { enabledGroups } = get();
    const newEnabledGroups = new Set(enabledGroups);
    
    if (newEnabledGroups.has(groupName)) {
      newEnabledGroups.delete(groupName);
    } else {
      newEnabledGroups.add(groupName);
    }
    
    set({ enabledGroups: newEnabledGroups });
  },
  
  selectAllGroups: () => {
    const { groups } = get();
    set({ enabledGroups: new Set(groups) });
  },
  
  unselectAllGroups: () => {
    set({ enabledGroups: new Set() });
  },
  
  clearGroupFilter: () => {
    set({ 
      selectedGroup: null,
      groupDisplayMode: GroupDisplayMode.EnabledGroups 
    });
  },
  
  // Async actions
  fetchGroups: async (id = null) => {
    const fetchedGroups = await invoke<string[]>('get_groups', { id });
    set({ groups: fetchedGroups });
  },
  
  fetchEnabledGroups: async (channelListId) => {
    const fetchedEnabledGroups = await invoke<string[]>('get_enabled_groups', { channelListId });
    set({ enabledGroups: new Set(fetchedEnabledGroups) });
    return fetchedEnabledGroups;
  },
  
  updateGroupSelection: async (channelListId, groupName, enabled) => {
    await invoke('update_group_selection', {
      channelListId,
      groupName,
      enabled
    });
    
    // Update local state
    const { enabledGroups } = get();
    const newEnabledGroups = new Set(enabledGroups);
    
    if (enabled) {
      newEnabledGroups.add(groupName);
    } else {
      newEnabledGroups.delete(groupName);
    }
    
    set({ enabledGroups: newEnabledGroups });
  },
  
  enableAllGroups: async (channelListId) => {
    const { groups } = get();
    await invoke('enable_all_groups', {
      channelListId,
      groups
    });
    set({ enabledGroups: new Set(groups) });
  },
  
  disableAllGroups: async (channelListId) => {
    const { groups, enabledGroups } = get();
    
    for (const group of groups) {
      if (enabledGroups.has(group)) {
        await invoke('update_group_selection', {
          channelListId,
          groupName: group,
          enabled: false
        });
      }
    }
    
    set({ enabledGroups: new Set() });
  },
}));

export default useGroupStore;