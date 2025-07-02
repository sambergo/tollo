import { useEffect, useRef, startTransition } from "react";
import { invoke } from "@tauri-apps/api/core";
import Hls from "hls.js";
import NavigationSidebar from "./components/NavigationSidebar";
import MainContent from "./components/MainContent";
import VideoPlayer from "./components/VideoPlayer";
import ChannelDetails from "./components/ChannelDetails";
import Settings from "./components/Settings";
import { useKeyboardNavigation } from "./hooks/useKeyboardNavigation";
import type { Channel } from "./components/ChannelList";
import { 
  useChannelStore,
  useUIStore,
  useGroupStore,
  useUserDataStore,
  useVideoPlayerStore,
  GroupDisplayMode
} from "./stores";
import { useSavedFilters, type SavedFilter } from "./hooks/useSavedFilters";
import { useChannelSearch } from "./hooks/useChannelSearch";
import "./App.css";

function App() {
  // Zustand stores
  const { 
    channels, 
    selectedChannel, 
    selectedChannelListId,
    filteredChannels,
    setChannels,
    setSelectedChannel,
    setSelectedChannelListId,
    setFilteredChannels,
    fetchChannels,
    loadDefaultChannelList,
    syncGroupsForChannelList
  } = useChannelStore();
  
  const {
    activeTab,
    focusedIndex,
    skipSearchEffect,
    debouncedSearchQuery,
    setActiveTab,
    setFocusedIndex,
    setIsLoadingChannelList,
    setSkipSearchEffect,
    setSearchQuery
  } = useUIStore();
  
  const {
    groups,
    selectedGroup,
    enabledGroups,
    groupDisplayMode,
    setGroups,
    setSelectedGroup,
    setGroupDisplayMode,
    fetchGroups,
    fetchEnabledGroups,
    enableAllGroups
  } = useGroupStore();
  
  const {
    favorites,
    history,
    fetchFavorites,
    fetchHistory,
    toggleFavorite
  } = useUserDataStore();
  
  const {
    setVideoRef,
    setHlsInstance
  } = useVideoPlayerStore();
  
  // Local refs
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);
  
  // Custom hooks
  const { searchChannels } = useChannelSearch(selectedChannelListId);
  const { savedFilters, saveFilter } = useSavedFilters(selectedChannelListId);

  // Set video ref in store when component mounts
  useEffect(() => {
    setVideoRef(videoRef);
  }, [setVideoRef]);

  // Load default channel list on app startup
  useEffect(() => {
    loadDefaultChannelList();
  }, [loadDefaultChannelList]);

  // Trigger search when debounced query changes
  useEffect(() => {
    if (skipSearchEffect) return;
    
    const performSearch = async () => {
      const searchedChannels = await searchChannels(debouncedSearchQuery);
      setChannels(searchedChannels);
    };
    performSearch();
  }, [debouncedSearchQuery, selectedChannelListId, skipSearchEffect, searchChannels, setChannels]);

  // Load channel list data
  useEffect(() => {
    const loadChannelListData = async () => {
      if (selectedChannelListId === null) {
        setIsLoadingChannelList(false);
        setSkipSearchEffect(false);
        return;
      }
      
      setSkipSearchEffect(true);
      
      try {
        await fetchChannels(selectedChannelListId);
        await fetchFavorites();
        await fetchGroups(selectedChannelListId);
        await fetchHistory();
        
        const fetchedGroups = await invoke<string[]>("get_groups", { id: selectedChannelListId });
        await syncGroupsForChannelList(selectedChannelListId, fetchedGroups);
        
        const currentEnabledGroups = await fetchEnabledGroups(selectedChannelListId);
        
        if (currentEnabledGroups.length === 0 && fetchedGroups.length > 0) {
          await enableAllGroups(selectedChannelListId);
          await fetchEnabledGroups(selectedChannelListId);
        }
        
        setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
        setSelectedGroup(null);
      } catch (error) {
        console.error("Failed to load channel list data:", error);
      } finally {
        setIsLoadingChannelList(false);
        setSkipSearchEffect(false);
      }
    };
    
    loadChannelListData();
  }, [selectedChannelListId, fetchChannels, fetchFavorites, fetchGroups, fetchHistory, 
      syncGroupsForChannelList, fetchEnabledGroups, enableAllGroups, 
      setGroupDisplayMode, setSelectedGroup, setIsLoadingChannelList, setSkipSearchEffect]);

  // Video playback effect
  useEffect(() => {
    if (hlsRef.current) {
      hlsRef.current.destroy();
    }

    if (selectedChannel && videoRef.current) {
      const video = videoRef.current;
      const isHlsUrl = selectedChannel.url.includes('.m3u8') || selectedChannel.url.includes('m3u8');
      
      if (isHlsUrl && Hls.isSupported()) {
        const hls = new Hls();
        hlsRef.current = hls;
        setHlsInstance(hls);
        hls.loadSource(selectedChannel.url);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          video.play();
        });
      } else if (isHlsUrl && video.canPlayType("application/vnd.apple.mpegurl")) {
        video.src = selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          video.play();
        });
      } else {
        video.src = selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          video.play();
        });
        
        video.addEventListener("error", (e: Event) => {
          console.warn(`Video load error for ${selectedChannel.name}:`, e);
        });
      }
    }
  }, [selectedChannel, setHlsInstance]);

  // Compute filtered channels based on group settings
  useEffect(() => {
    let filtered = [...channels];
    
    // Apply group filtering based on current mode
    if (groupDisplayMode === 'enabled') {
      // Show only channels from enabled groups
      filtered = filtered.filter(channel => enabledGroups.has(channel.group_title));
    } else if (groupDisplayMode === 'all' && selectedGroup) {
      // Traditional single group selection from all groups
      filtered = filtered.filter(channel => channel.group_title === selectedGroup);
    }
    // If AllGroups mode with no selection, show all channels
    
    setFilteredChannels(filtered);
  }, [channels, groupDisplayMode, enabledGroups, selectedGroup, setFilteredChannels]);

  const listItems = (() => {
    switch (activeTab) {
      case "channels":
        return filteredChannels;
      case "favorites":
        return favorites;
      case "groups":
        return groups;
      case "history":
        return history;
      default:
        return [];
    }
  })();

  // Keyboard navigation
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
    searchQuery: "",
    setFocusedIndex: (value: number | ((prev: number) => number)) => {
      if (typeof value === 'function') {
        setFocusedIndex(value(focusedIndex));
      } else {
        setFocusedIndex(value);
      }
    },
    setSelectedChannel,
    setActiveTab,
    handleSelectGroup: setSelectedGroup,
    handleToggleFavorite: toggleFavorite,
    handlePlayInMpv: async (channel: Channel) => {
      await invoke("play_channel", { channel });
      await fetchHistory();
    },
    savedFilters,
    onSaveFilter: async (slotNumber: number, searchQuery: string, selectedGroup: string | null, name: string) => {
      if (selectedChannelListId === null) return false;
      return await saveFilter(slotNumber, searchQuery, selectedGroup, name);
    },
    onApplyFilter: (filter: SavedFilter) => {
      setSearchQuery(filter.search_query);
      setSelectedGroup(filter.selected_group);
      setActiveTab("channels");
      setFocusedIndex(0);
    }
  });

  return (
    <div className="container">
      <NavigationSidebar />

      <div className="main-content">
        {activeTab === "settings" ? (
          <div className="settings-full-width">
            <div className="section-header">
              <h2 className="section-title">Settings</h2>
              <p className="section-subtitle">Application settings</p>
            </div>
            <div className="settings-container">
              <Settings 
                onSelectList={(id: number) => {
                  setIsLoadingChannelList(true);
                  setSkipSearchEffect(true);
                  setChannels([]);
                  setGroups([]);
                  setSelectedGroup(null);
                  setFocusedIndex(0);
                  setActiveTab("channels");
                  
                  setTimeout(() => {
                    startTransition(() => {
                      setSelectedChannelListId(id);
                    });
                  }, 50);
                }} 
                onFiltersChanged={async () => {}}
                selectedChannelListId={selectedChannelListId}
              />
            </div>
          </div>
        ) : (
          <>
            <MainContent />

            <div className="video-section">
              <VideoPlayer 
                ref={videoRef}
                selectedChannel={selectedChannel} 
              />

              <ChannelDetails />
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default App;
