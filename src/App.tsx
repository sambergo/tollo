import { useEffect, useRef } from "react";
import Hls from "hls.js";
import NavigationSidebar from "./components/NavigationSidebar";
import MainContent from "./components/MainContent";
import VideoPlayer from "./components/VideoPlayer";
import ChannelDetails from "./components/ChannelDetails";
import Settings from "./components/Settings";
import { useChannelListName } from "./hooks/useChannelListName";
import { useKeyboardNavigation } from "./hooks/useKeyboardNavigation";
import { useAppState } from "./hooks/useAppState";
import { usePlayerStore } from "./stores";
import "./App.css";

function App() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);
  
  // Get state and actions from Zustand stores
  const appState = useAppState();
  const playerStore = usePlayerStore();
  
  // Legacy hooks - these will eventually be migrated to stores too
  const channelListName = useChannelListName(appState.selectedChannelListId);

  // Set video refs in player store for cleanup operations
  useEffect(() => {
    playerStore.setVideoRef(videoRef);
    playerStore.setHlsRef(hlsRef);
  }, [playerStore]);

  // Update channel list name in the store when it changes
  useEffect(() => {
    appState.setChannelListName(channelListName);
  }, [channelListName, appState]);

  // Handle video player setup when selected channel changes
  useEffect(() => {
    if (hlsRef.current) {
      hlsRef.current.destroy();
    }

    if (appState.selectedChannel && videoRef.current) {
      const video = videoRef.current;
      const isHlsUrl = appState.selectedChannel.url.includes('.m3u8') || appState.selectedChannel.url.includes('m3u8');
      
      if (isHlsUrl && Hls.isSupported()) {
        // Use HLS.js for .m3u8 streams when supported
        const hls = new Hls();
        hlsRef.current = hls;
        hls.loadSource(appState.selectedChannel.url);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          video.play();
        });
      } else if (isHlsUrl && video.canPlayType("application/vnd.apple.mpegurl")) {
        // Native HLS support (Safari)
        video.src = appState.selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          video.play();
        });
      } else {
        // Fallback for direct video streams (MP4, WebM, etc.) and other protocols
        video.src = appState.selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          video.play();
        });
        
        // Handle load errors gracefully
        video.addEventListener("error", (e) => {
          console.warn(`Video load error for ${appState.selectedChannel?.name}:`, e);
        });
      }
    }
  }, [appState.selectedChannel]);

  // Keyboard navigation with all the required parameters
  useKeyboardNavigation({
    activeTab: appState.activeTab,
    channels: appState.channels,
    favorites: appState.favorites,
    groups: appState.groups,
    history: appState.history,
    selectedGroup: appState.selectedGroup,
    selectedChannel: appState.selectedChannel,
    focusedIndex: appState.focusedIndex,
    listItems: appState.listItems,
    searchQuery: appState.searchQuery,
    setFocusedIndex: appState.setFocusedIndex,
    setSelectedChannel: appState.setSelectedChannel,
    setActiveTab: appState.setActiveTab,
    handleSelectGroup: appState.handleSelectGroup,
    handleToggleFavorite: appState.handleToggleFavorite,
    handlePlayInMpv: appState.handlePlayInMpv,
    savedFilters: appState.savedFilters,
    onSaveFilter: appState.handleSaveFilter,
    onApplyFilter: appState.handleApplyFilter
  });

  return (
    <div className="container">
      <NavigationSidebar 
        activeTab={appState.activeTab} 
        onTabChange={appState.setActiveTab}
        savedFilters={appState.savedFilters}
        onApplyFilter={appState.handleApplyFilter}
      />

      <div className="main-content">
        {appState.activeTab === "settings" ? (
          <div className="settings-full-width">
            <div className="section-header">
              <h2 className="section-title">Settings</h2>
              <p className="section-subtitle">Application settings</p>
            </div>
            <div className="settings-container">
              <Settings 
                onSelectList={appState.handleSelectChannelList} 
                onFiltersChanged={() => appState.refreshFilters(appState.selectedChannelListId)}
                selectedChannelListId={appState.selectedChannelListId}
              />
            </div>
          </div>
        ) : (
          <>
            <MainContent
              activeTab={appState.activeTab}
              channelListName={appState.channelListName}
              searchQuery={appState.searchQuery}
              isSearching={appState.isSearching}
              filteredChannels={appState.filteredChannels}
              favorites={appState.favorites}
              groups={appState.groups}
              history={appState.history}
              selectedGroup={appState.selectedGroup}
              selectedChannel={appState.selectedChannel}
              focusedIndex={appState.focusedIndex}
              enabledGroups={appState.enabledGroups}
              groupDisplayMode={appState.groupDisplayMode}
              isLoadingChannelList={appState.isLoadingChannelList}
              onSearch={appState.handleSearch}
              onClearSearch={appState.handleClearSearch}
              onSelectChannel={appState.setSelectedChannel}
              onToggleFavorite={appState.handleToggleFavorite}
              onSelectGroup={appState.handleSelectGroup}
              onToggleGroupEnabled={appState.handleToggleGroupEnabled}
              onChangeDisplayMode={appState.handleChangeDisplayMode}
              onSelectAllGroups={appState.handleSelectAllGroups}
              onUnselectAllGroups={appState.handleUnselectAllGroups}
              onClearGroupFilter={appState.handleClearGroupFilter}
            />

            <div className="video-section">
              <VideoPlayer 
                ref={videoRef}
                selectedChannel={appState.selectedChannel} 
              />

              {appState.selectedChannel && (
                <ChannelDetails
                  selectedChannel={appState.selectedChannel}
                  channels={appState.channels}
                  isFavorite={appState.isFavorite(appState.selectedChannel)}
                  onPlayInMpv={appState.handlePlayInMpv}
                  onToggleFavorite={appState.handleToggleFavorite}
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