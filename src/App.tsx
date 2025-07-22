import { useEffect, useRef, startTransition } from "react";
import { invoke } from "@tauri-apps/api/core";
import Hls from "hls.js";
import NavigationSidebar from "./components/NavigationSidebar";
import MainContent from "./components/MainContent";
import VideoPlayer from "./components/VideoPlayer";
import ChannelDetails from "./components/ChannelDetails";
import Settings from "./components/Settings";
import Help from "./components/Help";

import { useKeyboardNavigation } from "./hooks/useKeyboardNavigation";
import { useChannelSearch } from "./hooks/useChannelSearch";
import { useSavedFilters } from "./hooks/useSavedFilters";
import {
  useChannelStore,
  useUIStore,
  useSearchStore,
  useSettingsStore,
  GroupDisplayMode,
  type SavedFilter,
} from "./stores";
import { asyncPlaylistStore } from "./stores/asyncPlaylistStore";
import type { Channel } from "./components/ChannelList";
import "./App.css";

function App() {
  // Zustand store hooks
  const {
    channels,
    favorites,
    groups,
    history,
    selectedChannel,
    selectedChannelListId,
    setChannels,
    setSelectedChannel,
    setSelectedChannelListId,
    setIsLoadingChannelList,
    toggleFavorite,
    playInExternalPlayer,
    // NEW: Async operations
    fetchChannelsAsync,
    fetchFavoritesAsync,
    fetchGroupsAsync,
    fetchHistoryAsync,
  } = useChannelStore();

  const {
    activeTab,
    focusedIndex,
    selectedGroup,
    groupDisplayMode,
    enabledGroups,
    groupSearchTerm,
    skipSearchEffect,
    setActiveTab,
    setFocusedIndex,
    setSelectedGroup,
    setGroupDisplayMode,
    setGroupSearchTerm,
    setSkipSearchEffect,
    fetchEnabledGroups,
    selectAllGroups,
    unselectAllGroups,
    toggleGroupEnabled,
  } = useUIStore();

  const { searchQuery, setSearchQuery, clearSearch } = useSearchStore();

  // Get settings
  const { enablePreview, fetchEnablePreview, autoplay } = useSettingsStore();

  // Refs for video player
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);

  // Custom hooks (keeping existing functionality)
  const { debouncedSearchQuery, searchChannels } = useChannelSearch(
    selectedChannelListId,
  );
  const { savedFilters, saveFilter } = useSavedFilters(selectedChannelListId);

  // Fetch settings on app load
  useEffect(() => {
    fetchEnablePreview();
  }, [fetchEnablePreview]);

  // Load default channel list on app startup
  useEffect(() => {
    const loadDefaultChannelList = async () => {
      try {
        const channelLists = await invoke<
          {
            id: number;
            name: string;
            source: string;
            is_default: boolean;
            last_fetched: number | null;
          }[]
        >("get_channel_lists");
        const defaultList = channelLists.find((list) => list.is_default);
        if (defaultList && selectedChannelListId === null) {
          // Set loading state immediately for initial app load
          setIsLoadingChannelList(true);
          setSkipSearchEffect(true);

          // Clear current data to show loading state immediately
          setChannels([]);
          setSelectedGroup(null);
          setFocusedIndex(0);

          // Make sure user is on channels tab to see the loading screen
          setActiveTab("channels");

          // Give React a moment to render the loading screen before starting heavy operations
          setTimeout(() => {
            startTransition(() => {
              setSelectedChannelListId(defaultList.id);
            });
          }, 50);
        }
      } catch (error) {
        console.error("Failed to load default channel list:", error);
        setIsLoadingChannelList(false);
        setSkipSearchEffect(false);
      }
    };

    loadDefaultChannelList();
  }, []); // Only run once on mount

  async function syncGroupsForChannelList(
    channelListId: number,
    allGroups: string[],
  ) {
    await invoke("sync_channel_list_groups", {
      channelListId,
      groups: allGroups,
    });
  }

  // Trigger search when debounced query changes
  useEffect(() => {
    if (skipSearchEffect) return;

    const performSearch = async () => {
      const searchedChannels = await searchChannels(debouncedSearchQuery);
      setChannels(searchedChannels);
    };
    performSearch();
  }, [debouncedSearchQuery, selectedChannelListId, skipSearchEffect]);

  useEffect(() => {
    const loadChannelListData = async () => {
      if (selectedChannelListId === null) {
        setIsLoadingChannelList(false);
        setSkipSearchEffect(false);
        return;
      }

      // Skip search effect during channel list loading
      setSkipSearchEffect(true);

      try {
        // Use setTimeout to break up the work and keep UI responsive
        const performStep = (step: () => Promise<void>) => {
          return new Promise<void>((resolve) => {
            setTimeout(async () => {
              await step();
              resolve();
            }, 10); // Small delay to allow UI updates
          });
        };

        // Step 1: Fetch core data with async progress
        await performStep(async () => {
          await fetchChannelsAsync(selectedChannelListId);
          await fetchFavoritesAsync();
        });

        // Step 2: Fetch groups and history with async operations
        await performStep(async () => {
          await fetchGroupsAsync(selectedChannelListId);
          await fetchHistoryAsync();
        });

        // Step 3: Handle group setup
        await performStep(async () => {
          // Get all groups for this channel list
          const fetchedGroups = await invoke<string[]>("get_groups", {
            id: selectedChannelListId,
          });

          // Sync groups with database (adds new groups, removes deleted ones)
          await syncGroupsForChannelList(selectedChannelListId, fetchedGroups);

          // Load enabled groups for this channel list
          const currentEnabledGroups = await fetchEnabledGroups(
            selectedChannelListId,
          );

          // Auto-enable all groups if none are enabled (new or empty list)
          if (currentEnabledGroups.length === 0 && fetchedGroups.length > 0) {
            console.log(
              `Auto-enabling all ${fetchedGroups.length} groups for new channel list`,
            );
            // Use bulk operation instead of individual calls to avoid UI blocking
            await invoke("enable_all_groups", {
              channelListId: selectedChannelListId,
              groups: fetchedGroups,
            });
            // Refresh enabled groups to get the updated list
            await fetchEnabledGroups(selectedChannelListId);
          }

          // Reset UI state for new channel list
          setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
          setSelectedGroup(null);
        });
      } catch (error) {
        console.error("Failed to load channel list data:", error);
      } finally {
        setIsLoadingChannelList(false);
        setSkipSearchEffect(false);
      }
    };

    loadChannelListData();
  }, [selectedChannelListId]);

  useEffect(() => {
    if (hlsRef.current) {
      hlsRef.current.destroy();
    }

    // Only load video if preview is enabled
    if (enablePreview && selectedChannel && videoRef.current) {
      const video = videoRef.current;
      const isHlsUrl =
        selectedChannel.url.includes(".m3u8") ||
        selectedChannel.url.includes("m3u8");

      if (isHlsUrl && Hls.isSupported()) {
        // Use HLS.js for .m3u8 streams when supported
        const hls = new Hls();
        hlsRef.current = hls;
        hls.loadSource(selectedChannel.url);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          if (autoplay) video.play();
        });
      } else if (
        isHlsUrl &&
        video.canPlayType("application/vnd.apple.mpegurl")
      ) {
        // Native HLS support (Safari)
        video.src = selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          if (autoplay) video.play();
        });
      } else {
        // Fallback for direct video streams (MP4, WebM, etc.) and other protocols
        video.src = selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          if (autoplay) video.play();
        });

        // Handle load errors gracefully
        video.addEventListener("error", (e) => {
          console.warn(`Video load error for ${selectedChannel.name}:`, e);
        });
      }
    }
  }, [selectedChannel, enablePreview, autoplay]);

  const handleSelectGroup = (group: string | null) => {
    setSelectedGroup(group);
    setActiveTab("channels");
  };

  const handleToggleFavorite = async (channel: Channel) => {
    await toggleFavorite(channel);
  };

  const handlePlayInExternalPlayer = (channel: Channel) => {
    playInExternalPlayer(channel);
  };

  const filteredChannels = (() => {
    let filtered = [...channels];

    // Apply group filtering based on current mode
    if (groupDisplayMode === GroupDisplayMode.EnabledGroups) {
      // Show only channels from enabled groups
      filtered = filtered.filter((channel) =>
        enabledGroups.has(channel.group_title),
      );
    } else if (
      groupDisplayMode === GroupDisplayMode.AllGroups &&
      selectedGroup
    ) {
      // Traditional single group selection from all groups
      filtered = filtered.filter(
        (channel) => channel.group_title === selectedGroup,
      );
    }
    // If AllGroups mode with no selection, show all channels

    return filtered;
  })();

  // Filter groups based on search term for keyboard navigation
  const filteredDisplayedGroups = groups.filter((group: string) =>
    group.toLowerCase().includes(groupSearchTerm.toLowerCase()),
  );

  // Include "All Groups" option for AllGroups mode in keyboard navigation
  const displayedGroups = (() => {
    if (groupDisplayMode === GroupDisplayMode.AllGroups) {
      return [null, ...filteredDisplayedGroups]; // null represents "All Groups"
    }
    return filteredDisplayedGroups;
  })();

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

  // Saved filter handlers
  const handleSaveFilter = async (
    slotNumber: number,
    searchQuery: string,
    selectedGroup: string | null,
    name: string,
  ): Promise<boolean> => {
    const success = await saveFilter(
      slotNumber,
      searchQuery,
      selectedGroup,
      name,
    );
    if (success) {
      // Show some feedback to user (you could add a toast here)
      console.log(`Saved filter to slot ${slotNumber}: ${name}`);
    }
    return success;
  };

  const handleApplyFilter = (filter: SavedFilter) => {
    // Apply the search query
    setSearchQuery(filter.search_query);

    // Apply the group selection and set appropriate display mode
    setSelectedGroup(filter.selected_group);

    // If the filter has a selected group, switch to AllGroups mode to make the group filter active
    // If no group is selected, use EnabledGroups mode
    if (filter.selected_group) {
      setGroupDisplayMode(GroupDisplayMode.AllGroups);
    } else {
      setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
    }

    // Switch to channels tab to see the results
    setActiveTab("channels");
    setFocusedIndex(0);
  };

  const handleClearGroupSearch = () => {
    setGroupSearchTerm("");
    setFocusedIndex(0); // Reset focus when clearing search
  };

  const handleClearAllFilters = () => {
    // Clear search query
    clearSearch();
    // Clear group selection
    setSelectedGroup(null);
    // Reset to enabled groups mode
    setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
    // Switch to channels tab
    setActiveTab("channels");
    setFocusedIndex(0);
  };

  // Channel list management
  const handleRefreshCurrentChannelList = () => {
    if (selectedChannelListId !== null) {
      asyncPlaylistStore.refreshPlaylistAsync(selectedChannelListId);
    }
  };

  // Group management handlers
  const handleSelectAllGroups = () => {
    if (selectedChannelListId !== null) {
      selectAllGroups(groups, selectedChannelListId);
    }
  };

  const handleUnselectAllGroups = () => {
    if (selectedChannelListId !== null) {
      unselectAllGroups(groups, selectedChannelListId);
    }
  };

  const handleToggleGroupDisplayMode = () => {
    const newMode =
      groupDisplayMode === GroupDisplayMode.EnabledGroups
        ? GroupDisplayMode.AllGroups
        : GroupDisplayMode.EnabledGroups;
    setGroupDisplayMode(newMode);
    setFocusedIndex(0); // Reset focus when switching modes
  };

  const handleToggleCurrentGroupSelection = () => {
    if (activeTab === "groups" && selectedChannelListId !== null) {
      const currentGroup = listItems[focusedIndex] as string | null;
      if (currentGroup) {
        toggleGroupEnabled(currentGroup, selectedChannelListId);
      }
    }
  };

  // Video control handlers
  const handleToggleMute = () => {
    if (videoRef.current) {
      videoRef.current.muted = !videoRef.current.muted;
    }
  };

  const handleToggleFullscreen = () => {
    if (videoRef.current) {
      if (document.fullscreenElement) {
        document.exitFullscreen();
      } else {
        videoRef.current.requestFullscreen();
      }
    }
  };

  const handleTogglePlayPause = () => {
    if (videoRef.current) {
      if (videoRef.current.paused) {
        videoRef.current.play();
      } else {
        videoRef.current.pause();
      }
    }
  };

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
    searchQuery,
    setFocusedIndex,
    setSelectedChannel,
    setActiveTab,
    handleSelectGroup,
    handleToggleFavorite,
    handlePlayInExternalPlayer,
    savedFilters,
    onSaveFilter: handleSaveFilter,
    onApplyFilter: handleApplyFilter,
    clearSearch,
    clearGroupSearch: handleClearGroupSearch,
    clearAllFilters: handleClearAllFilters,
    refreshCurrentChannelList: handleRefreshCurrentChannelList,
    selectAllGroups: handleSelectAllGroups,
    unselectAllGroups: handleUnselectAllGroups,
    toggleGroupDisplayMode: handleToggleGroupDisplayMode,
    toggleCurrentGroupSelection: handleToggleCurrentGroupSelection,
    toggleMute: handleToggleMute,
    toggleFullscreen: handleToggleFullscreen,
    togglePlayPause: handleTogglePlayPause,
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
              <Settings />
            </div>
          </div>
        ) : activeTab === "help" ? (
          <div className="settings-full-width">
            <div className="section-header">
              <h2 className="section-title">Help</h2>
              <p className="section-subtitle">
                Keyboard shortcuts and keybindings
              </p>
            </div>
            <div className="settings-container">
              <Help />
            </div>
          </div>
        ) : (
          <>
            <MainContent filteredChannels={filteredChannels} />

            <div
              className={`video-section ${!enablePreview ? "preview-disabled" : ""}`}
            >
              {enablePreview && <VideoPlayer ref={videoRef} />}

              {selectedChannel && <ChannelDetails />}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default App;
