import { useEffect, useCallback, startTransition } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  useChannelStore,
  useGroupStore,
  useFavoritesStore,
  useHistoryStore,
  useUIStore,
  useFilterStore,
  usePlayerStore,
  GroupDisplayMode
} from '../stores';
import type { Channel } from '../components/ChannelList';
import type { SavedFilter } from '../hooks/useSavedFilters';

export function useAppState() {
  // Store hooks
  const channelStore = useChannelStore();
  const groupStore = useGroupStore();
  const favoritesStore = useFavoritesStore();
  const historyStore = useHistoryStore();
  const uiStore = useUIStore();
  const filterStore = useFilterStore();
  const playerStore = usePlayerStore();

    // Debounced search effect
  useEffect(() => {
    if (channelStore.skipSearchEffect) return;
    
    const performSearch = async () => {
      await channelStore.performSearch(
        channelStore.debouncedSearchQuery, 
        channelStore.selectedChannelListId
      );
    };
    performSearch();
  }, [channelStore.debouncedSearchQuery, channelStore.selectedChannelListId, channelStore.skipSearchEffect]);

  // Debounce search query
  useEffect(() => {
    const timer = setTimeout(() => {
      channelStore.setDebouncedSearchQuery(channelStore.searchQuery);
    }, 400);

    return () => clearTimeout(timer);
  }, [channelStore.searchQuery]);

  // Load default channel list on app startup
  useEffect(() => {
    const loadDefaultChannelList = async () => {
      try {
        const channelLists = await invoke<{id: number, name: string, source: string, is_default: boolean, last_fetched: number | null}[]>("get_channel_lists");
        const defaultList = channelLists.find(list => list.is_default);
        if (defaultList && channelStore.selectedChannelListId === null) {
          // Set loading state immediately for initial app load
          channelStore.setIsLoadingChannelList(true);
          channelStore.setSkipSearchEffect(true);
          
          // Clear current data to show loading state immediately
          channelStore.clearChannels();
          groupStore.setGroups([]);
          groupStore.setSelectedGroup(null);
          uiStore.setFocusedIndex(0);
          
          // Make sure user is on channels tab to see the loading screen
          uiStore.setActiveTab("channels");
          
          // Give React a moment to render the loading screen before starting heavy operations
          setTimeout(() => {
            startTransition(() => {
              channelStore.setSelectedChannelListId(defaultList.id);
            });
          }, 50);
        }
      } catch (error) {
        console.error("Failed to load default channel list:", error);
        channelStore.setIsLoadingChannelList(false);
        channelStore.setSkipSearchEffect(false);
      }
    };

    loadDefaultChannelList();
  }, []); // Only run once on mount

  // Handle channel list changes
  useEffect(() => {
    const loadChannelListData = async () => {
      if (channelStore.selectedChannelListId === null) {
        channelStore.setIsLoadingChannelList(false);
        channelStore.setSkipSearchEffect(false);
        return;
      }
      
      // Skip search effect during channel list loading
      channelStore.setSkipSearchEffect(true);
      
      try {
        await Promise.all([
          channelStore.fetchChannels(channelStore.selectedChannelListId),
          favoritesStore.fetchFavorites(),
          groupStore.fetchGroups(channelStore.selectedChannelListId),
          historyStore.fetchHistory()
        ]);
        
        // Handle group selections for the new channel list
        const fetchedGroups = await invoke<string[]>("get_groups", { id: channelStore.selectedChannelListId });
        
        // Sync groups with database
        await groupStore.syncGroupsForChannelList(channelStore.selectedChannelListId, fetchedGroups);
        
        // Load enabled groups for this channel list
        const currentEnabledGroups = await groupStore.fetchEnabledGroups(channelStore.selectedChannelListId);
        
        // Auto-enable all groups if none are enabled (new or empty list)  
        if (currentEnabledGroups.length === 0 && fetchedGroups.length > 0) {
          console.log(`Auto-enabling all ${fetchedGroups.length} groups for new channel list`);
          await invoke("enable_all_groups", {
            channelListId: channelStore.selectedChannelListId,
            groups: fetchedGroups
          });
          // Refresh enabled groups to get the updated list
          await groupStore.fetchEnabledGroups(channelStore.selectedChannelListId);
        }
        
        // Reset UI state for new channel list
        groupStore.setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
        groupStore.setSelectedGroup(null);
        
        // Load saved filters for this channel list
        await filterStore.fetchSavedFilters(channelStore.selectedChannelListId);
      } catch (error) {
        console.error("Failed to load channel list data:", error);
      } finally {
        channelStore.setIsLoadingChannelList(false);
        channelStore.setSkipSearchEffect(false);
      }
    };
    
    loadChannelListData();
  }, [channelStore.selectedChannelListId]);

  // Computed values
  const filteredChannels = useCallback(() => {
    let filtered = [...channelStore.channels];
    
    // Apply group filtering based on current mode
    if (groupStore.groupDisplayMode === GroupDisplayMode.EnabledGroups) {
      // Show only channels from enabled groups
      filtered = filtered.filter(channel => groupStore.enabledGroups.has(channel.group_title));
    } else if (groupStore.groupDisplayMode === GroupDisplayMode.AllGroups && groupStore.selectedGroup) {
      // Traditional single group selection from all groups
      filtered = filtered.filter(channel => channel.group_title === groupStore.selectedGroup);
    }
    // If AllGroups mode with no selection, show all channels
    
    return filtered;
  }, [channelStore.channels, groupStore.enabledGroups, groupStore.groupDisplayMode, groupStore.selectedGroup]);

  const listItems = useCallback(() => {
    switch (uiStore.activeTab) {
      case "channels":
        return filteredChannels();
      case "favorites":
        return favoritesStore.favorites;
      case "groups":
        return groupStore.groups;
      case "history":
        return historyStore.history;
      default:
        return [];
    }
  }, [uiStore.activeTab, filteredChannels, favoritesStore.favorites, groupStore.groups, historyStore.history]);

  // Action handlers
  const handlePlayInMpv = useCallback(async (channel: Channel) => {
    if (channel) {
      await playerStore.playInMpv(channel);
      await historyStore.fetchHistory();
    }
  }, [playerStore, historyStore]);

  const handleToggleFavorite = useCallback(async (channel: Channel) => {
    await favoritesStore.toggleFavorite(channel);
  }, [favoritesStore]);

  const handleSelectGroup = useCallback((group: string | null) => {
    groupStore.setSelectedGroup(group);
    uiStore.setActiveTab("channels");
  }, [groupStore, uiStore]);

  const handleToggleGroupEnabled = useCallback(async (groupName: string) => {
    if (channelStore.selectedChannelListId === null) return;
    await groupStore.toggleGroupEnabled(groupName, channelStore.selectedChannelListId);
  }, [groupStore, channelStore.selectedChannelListId]);

  const handleChangeDisplayMode = useCallback((mode: GroupDisplayMode) => {
    groupStore.setGroupDisplayMode(mode);
  }, [groupStore]);

  const handleSelectAllGroups = useCallback(async () => {
    if (channelStore.selectedChannelListId === null) return;
    await groupStore.selectAllGroups(channelStore.selectedChannelListId);
  }, [groupStore, channelStore.selectedChannelListId]);

  const handleUnselectAllGroups = useCallback(async () => {
    if (channelStore.selectedChannelListId === null) return;
    await groupStore.unselectAllGroups(channelStore.selectedChannelListId);
  }, [groupStore, channelStore.selectedChannelListId]);

  const handleClearGroupFilter = useCallback(() => {
    groupStore.clearGroupFilter();
  }, [groupStore]);

  const handleSearch = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    channelStore.setSearchQuery(e.target.value);
  }, [channelStore]);

  const handleClearSearch = useCallback(() => {
    channelStore.setSearchQuery("");
  }, [channelStore]);

  const handleSelectChannelList = useCallback((id: number) => {
    // Set loading state immediately when user clicks Select
    channelStore.setIsLoadingChannelList(true);
    channelStore.setSkipSearchEffect(true);
    
    // Clear current data to show loading state immediately
    channelStore.clearChannels();
    groupStore.setGroups([]);
    groupStore.setSelectedGroup(null);
    uiStore.setFocusedIndex(0);
    
    // Switch to channels tab immediately
    uiStore.setActiveTab("channels");
    
    // Give React a proper moment to render the loading screen before starting heavy operations
    setTimeout(() => {
      startTransition(() => {
        channelStore.setSelectedChannelListId(id);
      });
    }, 50);
  }, [channelStore, groupStore, uiStore]);

  // Filter handlers
  const handleSaveFilter = useCallback(async (slotNumber: number, searchQuery: string, selectedGroup: string | null, name: string): Promise<boolean> => {
    if (channelStore.selectedChannelListId === null) return false;
    return await filterStore.saveFilter(channelStore.selectedChannelListId, slotNumber, searchQuery, selectedGroup, name);
  }, [filterStore, channelStore.selectedChannelListId]);

  const handleApplyFilter = useCallback((filter: SavedFilter) => {
    // Apply the search query
    channelStore.setSearchQuery(filter.search_query);
    
    // Apply the group selection and set appropriate display mode
    groupStore.setSelectedGroup(filter.selected_group);
    
    // If the filter has a selected group, switch to AllGroups mode to make the group filter active
    // If no group is selected, use EnabledGroups mode
    if (filter.selected_group) {
      groupStore.setGroupDisplayMode(GroupDisplayMode.AllGroups);
    } else {
      groupStore.setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
    }
    
    // Switch to channels tab to see the results
    uiStore.setActiveTab("channels");
    uiStore.setFocusedIndex(0);
  }, [channelStore, groupStore, uiStore]);

  return {
    // State
    channels: channelStore.channels,
    favorites: favoritesStore.favorites,
    groups: groupStore.groups,
    history: historyStore.history,
    selectedGroup: groupStore.selectedGroup,
    selectedChannel: uiStore.selectedChannel,
    activeTab: uiStore.activeTab,
    focusedIndex: uiStore.focusedIndex,
    selectedChannelListId: channelStore.selectedChannelListId,
    channelListName: channelStore.channelListName,
    searchQuery: channelStore.searchQuery,
    isSearching: channelStore.isSearching,
    enabledGroups: groupStore.enabledGroups,
    groupDisplayMode: groupStore.groupDisplayMode,
    isLoadingChannelList: channelStore.isLoadingChannelList,
    savedFilters: filterStore.savedFilters,
    
    // Computed values
    filteredChannels: filteredChannels(),
    listItems: listItems(),
    
    // Utility functions
    isFavorite: favoritesStore.isFavorite,
    
    // Action handlers
    handlePlayInMpv,
    handleToggleFavorite,
    handleSelectGroup,
    handleToggleGroupEnabled,
    handleChangeDisplayMode,
    handleSelectAllGroups,
    handleUnselectAllGroups,
    handleClearGroupFilter,
    handleSearch,
    handleClearSearch,
    handleSelectChannelList,
    handleSaveFilter,
    handleApplyFilter,
    
    // Direct store actions
    setSelectedChannel: uiStore.setSelectedChannel,
    setActiveTab: uiStore.setActiveTab,
    setFocusedIndex: uiStore.setFocusedIndex,
    setChannelListName: channelStore.setChannelListName,
    refreshFilters: filterStore.refreshFilters,
  };
}