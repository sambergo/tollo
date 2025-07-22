import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { Tab } from "../components/NavigationSidebar";

export enum GroupDisplayMode {
  EnabledGroups = "enabled",
  AllGroups = "all",
}

interface UIState {
  // Navigation and focus
  activeTab: Tab;
  focusedIndex: number;

  // Group selection
  selectedGroup: string | null;
  groupDisplayMode: GroupDisplayMode;
  enabledGroups: Set<string>;
  groupSearchTerm: string;

  // Control flags
  skipSearchEffect: boolean;

  // Actions
  setActiveTab: (tab: Tab) => void;
  setFocusedIndex: (index: number | ((prev: number) => number)) => void;
  setSelectedGroup: (group: string | null) => void;
  setGroupDisplayMode: (mode: GroupDisplayMode) => void;
  setEnabledGroups: (groups: Set<string>) => void;
  setGroupSearchTerm: (term: string) => void;
  setSkipSearchEffect: (skip: boolean) => void;

  // Group management actions
  toggleGroupEnabled: (
    groupName: string,
    channelListId: number,
  ) => Promise<void>;
  selectAllGroups: (groups: string[], channelListId: number) => Promise<void>;
  unselectAllGroups: (groups: string[], channelListId: number) => Promise<void>;
  fetchEnabledGroups: (channelListId: number) => Promise<string[]>;
  clearGroupFilter: () => void;
}

export const useUIStore = create<UIState>((set, get) => ({
  // Initial state
  activeTab: "channels" as Tab,
  focusedIndex: 0,
  selectedGroup: null,
  groupDisplayMode: GroupDisplayMode.EnabledGroups,
  enabledGroups: new Set(),
  groupSearchTerm: "",
  skipSearchEffect: false,

  // Basic setters
  setActiveTab: (activeTab) => set({ activeTab }),
  setFocusedIndex: (focusedIndex) => {
    if (typeof focusedIndex === "function") {
      set((state) => ({ focusedIndex: focusedIndex(state.focusedIndex) }));
    } else {
      set({ focusedIndex });
    }
  },
  setSelectedGroup: (selectedGroup) => set({ selectedGroup }),
  setGroupDisplayMode: (groupDisplayMode) => set({ groupDisplayMode }),
  setEnabledGroups: (enabledGroups) => set({ enabledGroups }),
  setGroupSearchTerm: (groupSearchTerm) => set({ groupSearchTerm }),
  setSkipSearchEffect: (skipSearchEffect) => set({ skipSearchEffect }),

  // Group management actions
  toggleGroupEnabled: async (groupName, channelListId) => {
    const { enabledGroups } = get();
    const newEnabledState = !enabledGroups.has(groupName);

    // Update database
    await invoke("update_group_selection", {
      channelListId,
      groupName,
      enabled: newEnabledState,
    });

    // Update local state
    const newEnabledGroups = new Set(enabledGroups);
    if (newEnabledState) {
      newEnabledGroups.add(groupName);
    } else {
      newEnabledGroups.delete(groupName);
    }
    set({ enabledGroups: newEnabledGroups });
  },

  selectAllGroups: async (groups, channelListId) => {
    // Enable all groups in bulk using the optimized backend command
    await invoke("enable_all_groups", {
      channelListId,
      groups,
    });

    // Update local state to include all groups
    set({ enabledGroups: new Set(groups) });
  },

  unselectAllGroups: async (groups, channelListId) => {
    // Disable all groups in bulk using the optimized backend command
    await invoke("disable_all_groups", {
      channelListId,
      groups,
    });

    // Update local state to empty set
    set({ enabledGroups: new Set() });
  },

  fetchEnabledGroups: async (channelListId) => {
    const fetchedEnabledGroups = await invoke<string[]>("get_enabled_groups", {
      channelListId,
    });
    set({ enabledGroups: new Set(fetchedEnabledGroups) });
    return fetchedEnabledGroups;
  },

  clearGroupFilter: () => {
    set({
      selectedGroup: null,
      groupDisplayMode: GroupDisplayMode.EnabledGroups,
      groupSearchTerm: "",
    });
  },
}));
