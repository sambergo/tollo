import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import Hls from "hls.js";
import NavigationSidebar, { type Tab } from "./components/NavigationSidebar";
import MainContent from "./components/MainContent";
import VideoPlayer from "./components/VideoPlayer";
import ChannelDetails from "./components/ChannelDetails";
import Settings from "./components/Settings";
import { useChannelListName } from "./hooks/useChannelListName";
import { useKeyboardNavigation } from "./hooks/useKeyboardNavigation";
import { useChannelSearch } from "./hooks/useChannelSearch";
import type { Channel } from "./components/ChannelList";
import "./App.css";

enum GroupDisplayMode {
  EnabledGroups = 'enabled',
  AllGroups = 'all'
}

function App() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [favorites, setFavorites] = useState<Channel[]>([]);
  const [groups, setGroups] = useState<string[]>([]);
  const [history, setHistory] = useState<Channel[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [selectedChannel, setSelectedChannel] = useState<Channel | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>("channels");
  const [focusedIndex, setFocusedIndex] = useState(0);
  const [selectedChannelListId, setSelectedChannelListId] = useState<number | null>(null);
  
  // Group selection state
  const [groupDisplayMode, setGroupDisplayMode] = useState<GroupDisplayMode>(GroupDisplayMode.EnabledGroups);
  const [enabledGroups, setEnabledGroups] = useState<Set<string>>(new Set());
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);
  const channelListName = useChannelListName(selectedChannelListId);
  const { searchQuery, setSearchQuery, debouncedSearchQuery, isSearching, searchChannels } = useChannelSearch(selectedChannelListId);

  // Load default channel list on app startup
  useEffect(() => {
    const loadDefaultChannelList = async () => {
      try {
        const channelLists = await invoke<{id: number, name: string, source: string, is_default: boolean, last_fetched: number | null}[]>("get_channel_lists");
        const defaultList = channelLists.find(list => list.is_default);
        if (defaultList && selectedChannelListId === null) {
          setSelectedChannelListId(defaultList.id);
        }
      } catch (error) {
        console.error("Failed to load default channel list:", error);
      }
    };

    loadDefaultChannelList();
  }, []); // Only run once on mount

  async function fetchChannels(id: number | null = null) {
    const fetchedChannels = await invoke<Channel[]>("get_channels", { id });
    setChannels(fetchedChannels);
  }

  async function fetchFavorites() {
    const fetchedFavorites = await invoke<Channel[]>("get_favorites");
    setFavorites(fetchedFavorites);
  }

  async function fetchGroups(id: number | null = null) {
    const fetchedGroups = await invoke<string[]>("get_groups", { id });
    setGroups(fetchedGroups);
  }

  async function fetchEnabledGroups(channelListId: number) {
    const fetchedEnabledGroups = await invoke<string[]>("get_enabled_groups", { channelListId });
    setEnabledGroups(new Set(fetchedEnabledGroups));
    return fetchedEnabledGroups;
  }

  async function syncGroupsForChannelList(channelListId: number, allGroups: string[]) {
    await invoke("sync_channel_list_groups", { channelListId, groups: allGroups });
  }

  async function fetchHistory() {
    const fetchedHistory = await invoke<Channel[]>("get_history");
    setHistory(fetchedHistory);
  }

  // Trigger search when debounced query changes
  useEffect(() => {
    const performSearch = async () => {
      const searchedChannels = await searchChannels(debouncedSearchQuery);
      setChannels(searchedChannels);
    };
    performSearch();
  }, [debouncedSearchQuery, selectedChannelListId]);

  useEffect(() => {
    const loadChannelListData = async () => {
      await fetchChannels(selectedChannelListId);
      await fetchFavorites();
      await fetchGroups(selectedChannelListId);
      await fetchHistory();
      
      // Handle group selections for the new channel list
      if (selectedChannelListId !== null) {
        // Get all groups for this channel list
        const fetchedGroups = await invoke<string[]>("get_groups", { id: selectedChannelListId });
        
        // Sync groups with database (adds new groups, removes deleted ones)
        await syncGroupsForChannelList(selectedChannelListId, fetchedGroups);
        
        // Load enabled groups for this channel list
        const currentEnabledGroups = await fetchEnabledGroups(selectedChannelListId);
        
        // Auto-enable all groups if none are enabled (new or empty list)  
        if (currentEnabledGroups.length === 0 && fetchedGroups.length > 0) {
          console.log(`Auto-enabling all ${fetchedGroups.length} groups for new channel list`);
          // Use bulk operation instead of individual calls to avoid UI blocking
          await invoke("enable_all_groups", {
            channelListId: selectedChannelListId,
            groups: fetchedGroups
          });
          // Refresh enabled groups to get the updated list
          await fetchEnabledGroups(selectedChannelListId);
        }
        
        // Reset UI state for new channel list
        setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
        setSelectedGroup(null);
      }
    };
    
    loadChannelListData();
  }, [selectedChannelListId]);

  useEffect(() => {
    if (hlsRef.current) {
      hlsRef.current.destroy();
    }

    if (selectedChannel && videoRef.current) {
      const video = videoRef.current;
      const isHlsUrl = selectedChannel.url.includes('.m3u8') || selectedChannel.url.includes('m3u8');
      
      if (isHlsUrl && Hls.isSupported()) {
        // Use HLS.js for .m3u8 streams when supported
        const hls = new Hls();
        hlsRef.current = hls;
        hls.loadSource(selectedChannel.url);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          video.play();
        });
      } else if (isHlsUrl && video.canPlayType("application/vnd.apple.mpegurl")) {
        // Native HLS support (Safari)
        video.src = selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          video.play();
        });
      } else {
        // Fallback for direct video streams (MP4, WebM, etc.) and other protocols
        video.src = selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          video.play();
        });
        
        // Handle load errors gracefully
        video.addEventListener("error", (e) => {
          console.warn(`Video load error for ${selectedChannel.name}:`, e);
        });
      }
    }
  }, [selectedChannel]);

  const handlePlayInMpv = (channel: Channel) => {
    if (channel) {
      if (videoRef.current) {
        videoRef.current.pause();
        videoRef.current.src = "";
      }
      if (hlsRef.current) {
        hlsRef.current.destroy();
      }
      invoke("play_channel", { channel });
      fetchHistory();
    }
  };

  const isFavorite = (channel: Channel) => {
    return favorites.some((fav) => fav.name === channel.name);
  };

  const handleToggleFavorite = async (channel: Channel) => {
    if (isFavorite(channel)) {
      await invoke("remove_favorite", { name: channel.name });
    } else {
      await invoke("add_favorite", { channel });
    }
    fetchFavorites();
  };

  const handleSelectGroup = (group: string | null) => {
    setSelectedGroup(group);
    setActiveTab("channels");
  };

  const handleToggleGroupEnabled = async (groupName: string) => {
    if (selectedChannelListId === null) return;
    
    const newEnabledState = !enabledGroups.has(groupName);
    
    // Update database for current channel list
    await invoke("update_group_selection", {
      channelListId: selectedChannelListId,
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
    setEnabledGroups(newEnabledGroups);
  };

  const handleChangeDisplayMode = (mode: GroupDisplayMode) => {
    setGroupDisplayMode(mode);
    
    // Reset selection state when changing modes
    setSelectedGroup(null);
  };

  const handleSelectAllGroups = async () => {
    if (selectedChannelListId === null) return;
    
    // Enable all groups
    for (const group of groups) {
      if (!enabledGroups.has(group)) {
        await invoke("update_group_selection", {
          channelListId: selectedChannelListId,
          groupName: group,
          enabled: true
        });
      }
    }
    
    // Update local state to include all groups
    setEnabledGroups(new Set(groups));
  };

  const handleUnselectAllGroups = async () => {
    if (selectedChannelListId === null) return;
    
    // Disable all groups
    for (const group of groups) {
      if (enabledGroups.has(group)) {
        await invoke("update_group_selection", {
          channelListId: selectedChannelListId,
          groupName: group,
          enabled: false
        });
      }
    }
    
    // Update local state to empty set
    setEnabledGroups(new Set());
  };

  const handleClearGroupFilter = () => {
    // Clear the selected group and go back to enabled groups mode
    setSelectedGroup(null);
    setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
  };

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchQuery(e.target.value);
  };

  const handleSelectChannelList = (id: number) => {
    setSelectedChannelListId(id);
    setActiveTab("channels");
  };

  const filteredChannels = (() => {
    let filtered = [...channels];
    
    // Apply group filtering based on current mode
    if (groupDisplayMode === GroupDisplayMode.EnabledGroups) {
      // Show only channels from enabled groups
      filtered = filtered.filter(channel => enabledGroups.has(channel.group_title));
    } else if (groupDisplayMode === GroupDisplayMode.AllGroups && selectedGroup) {
      // Traditional single group selection from all groups
      filtered = filtered.filter(channel => channel.group_title === selectedGroup);
    }
    // If AllGroups mode with no selection, show all channels
    
    return filtered;
  })();

  // Show all groups in the groups tab for both Enabled Groups and All Groups modes
  const displayedGroups = groups;

  const listItems = (() => {
    switch (activeTab) {
      case "channels":
        return filteredChannels;
      case "favorites":
        return favorites;
      case "groups":
        return displayedGroups;
      case "history":
        return history;
      default:
        return [];
    }
  })();

  useKeyboardNavigation({
    activeTab,
    channels,
    favorites,
    groups,
    history,
    selectedGroup,
    selectedChannel,
    focusedIndex,
    listItems,
    setFocusedIndex,
    setSelectedChannel,
    setActiveTab,
    handleSelectGroup,
    handleToggleFavorite,
    handlePlayInMpv
  });

  return (
    <div className="container">
      <NavigationSidebar 
        activeTab={activeTab} 
        onTabChange={setActiveTab} 
      />

      <div className="main-content">
        {activeTab === "settings" ? (
          <div className="settings-full-width">
            <div className="section-header">
              <h2 className="section-title">Settings</h2>
              <p className="section-subtitle">Application settings</p>
            </div>
            <div className="settings-container">
              <Settings onSelectList={handleSelectChannelList} />
            </div>
          </div>
        ) : (
          <>
                         <MainContent
               activeTab={activeTab}
               channelListName={channelListName}
               searchQuery={searchQuery}
               isSearching={isSearching}
               filteredChannels={filteredChannels}
               favorites={favorites}
               groups={displayedGroups}
               history={history}
               selectedGroup={selectedGroup}
               selectedChannel={selectedChannel}
               focusedIndex={focusedIndex}
               enabledGroups={enabledGroups}
               groupDisplayMode={groupDisplayMode}
               onSearch={handleSearch}
               onSelectChannel={setSelectedChannel}
               onToggleFavorite={handleToggleFavorite}
               onSelectGroup={handleSelectGroup}
               onToggleGroupEnabled={handleToggleGroupEnabled}
               onChangeDisplayMode={handleChangeDisplayMode}
               onSelectAllGroups={handleSelectAllGroups}
               onUnselectAllGroups={handleUnselectAllGroups}
               onClearGroupFilter={handleClearGroupFilter}
             />

            <div className="video-section">
              <VideoPlayer 
                ref={videoRef}
                selectedChannel={selectedChannel} 
              />

              {selectedChannel && (
                <ChannelDetails
                  selectedChannel={selectedChannel}
                  channels={channels}
                  isFavorite={isFavorite(selectedChannel)}
                  onPlayInMpv={handlePlayInMpv}
                  onToggleFavorite={handleToggleFavorite}
                />
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default App;
