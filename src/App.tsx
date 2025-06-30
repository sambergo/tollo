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
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);
  const channelListName = useChannelListName(selectedChannelListId);
  const { searchQuery, setSearchQuery, debouncedSearchQuery, isSearching, searchChannels } = useChannelSearch(selectedChannelListId);

  async function fetchChannels(id: number | null = null) {
    const fetchedChannels = await invoke<Channel[]>("get_channels", { id });
    setChannels(fetchedChannels);
    setSelectedChannel(fetchedChannels[0]);
  }

  async function fetchFavorites() {
    const fetchedFavorites = await invoke<Channel[]>("get_favorites");
    setFavorites(fetchedFavorites);
  }

  async function fetchGroups(id: number | null = null) {
    const fetchedGroups = await invoke<string[]>("get_groups", { id });
    setGroups(fetchedGroups);
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
    fetchChannels(selectedChannelListId);
    fetchFavorites();
    fetchGroups(selectedChannelListId);
    fetchHistory();
  }, [selectedChannelListId]);

  useEffect(() => {
    if (hlsRef.current) {
      hlsRef.current.destroy();
    }

    if (selectedChannel && videoRef.current) {
      const video = videoRef.current;
      if (Hls.isSupported()) {
        const hls = new Hls();
        hlsRef.current = hls;
        hls.loadSource(selectedChannel.url);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          video.play();
        });
      } else if (video.canPlayType("application/vnd.apple.mpegurl")) {
        video.src = selectedChannel.url;
        video.addEventListener("loadedmetadata", () => {
          video.play();
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

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchQuery(e.target.value);
  };

  const handleSelectChannelList = (id: number) => {
    setSelectedChannelListId(id);
    setActiveTab("channels");
  };

  const filteredChannels = selectedGroup
    ? channels.filter((channel) => channel.group_title === selectedGroup)
    : [...channels];

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
               groups={groups}
               history={history}
               selectedGroup={selectedGroup}
               selectedChannel={selectedChannel}
               focusedIndex={focusedIndex}
               onSearch={handleSearch}
               onSelectChannel={setSelectedChannel}
               onToggleFavorite={handleToggleFavorite}
               onSelectGroup={handleSelectGroup}
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
