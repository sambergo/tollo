import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import Hls from "hls.js";
import Settings from "./components/Settings";
import CachedImage from "./components/CachedImage";
import "./App.css";

interface Channel {
  name: string;
  logo: string;
  url: string;
  group_title: string;
  tvg_id: string;
  resolution: string;
  extra_info: string;
}

type Tab = "channels" | "favorites" | "groups" | "history" | "settings";

function useChannelListName(selectedChannelListId: number | null) {
  const [channelListName, setChannelListName] = useState<string>("");

  useEffect(() => {
    async function fetchChannelLists() {
      const lists = await invoke<any[]>("get_channel_lists");
      const found = lists.find((l) => l.id === selectedChannelListId);
      setChannelListName(found ? found.name : "");
    }
    if (selectedChannelListId !== null) {
      fetchChannelLists();
    } else {
      setChannelListName("");
    }
  }, [selectedChannelListId]);

  return channelListName;
}

// Icon components (using SVG for better styling)
const TvIcon = () => (
  <svg className="nav-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
    <line x1="8" y1="21" x2="16" y2="21"/>
    <line x1="12" y1="17" x2="12" y2="21"/>
  </svg>
);

const HeartIcon = () => (
  <svg className="nav-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path d="20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/>
  </svg>
);

const UsersIcon = () => (
  <svg className="nav-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path d="17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
    <circle cx="9" cy="7" r="4"/>
    <path d="23 21v-2a4 4 0 0 0-3-3.87"/>
    <path d="16 3.13a4 4 0 0 1 0 7.75"/>
  </svg>
);

const HistoryIcon = () => (
  <svg className="nav-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path d="3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
    <path d="3 3v5h5"/>
    <path d="12 7v5l4 2"/>
  </svg>
);

const SettingsIcon = () => (
  <svg className="nav-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <circle cx="12" cy="12" r="3"/>
    <path d="19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1 1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
  </svg>
);

const PlayIcon = () => (
  <svg className="video-placeholder-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <polygon points="5,3 19,12 5,21"/>
  </svg>
);

const SignalIcon = () => (
  <svg className="meta-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path d="2 20h.01"/>
    <path d="7 20v-4"/>
    <path d="12 20v-8"/>
    <path d="17 20V8"/>
    <path d="22 4v16"/>
  </svg>
);

const StarIcon = () => (
  <svg className="meta-icon rating-star" fill="currentColor" stroke="currentColor" viewBox="0 0 24 24">
    <polygon points="12,2 15.09,8.26 22,9.27 17,14.14 18.18,21.02 12,17.77 5.82,21.02 7,14.14 2,9.27 8.91,8.26"/>
  </svg>
);

function App() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [favorites, setFavorites] = useState<Channel[]>([]);
  const [groups, setGroups] = useState<string[]>([]);
  const [history, setHistory] = useState<Channel[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedSearchQuery, setDebouncedSearchQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [selectedChannel, setSelectedChannel] = useState<Channel | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>("channels");
  const [focusedIndex, setFocusedIndex] = useState(0);
  const [selectedChannelListId, setSelectedChannelListId] = useState<number | null>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);
  const channelListName = useChannelListName(selectedChannelListId);

  // Debounce search query
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearchQuery(searchQuery);
    }, 400);

    return () => clearTimeout(timer);
  }, [searchQuery]);

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

  async function searchChannels(query: string) {
    if (query === "" || query.length < 3) {
      fetchChannels(selectedChannelListId);
    } else {
      setIsSearching(true);
      try {
        const searchedChannels = await invoke<Channel[]>("search_channels", { 
          query, 
          id: selectedChannelListId 
        });
        setChannels(searchedChannels);
      } catch (error) {
        console.error("Search failed:", error);
        fetchChannels(selectedChannelListId);
      } finally {
        setIsSearching(false);
      }
    }
  }

  // Trigger search when debounced query changes
  useEffect(() => {
    searchChannels(debouncedSearchQuery);
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

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement) {
        return;
      }

      if (e.key === "j" || e.key === "ArrowDown") {
        setFocusedIndex((prev) => Math.min(prev + 1, listItems.length - 1));
      } else if (e.key === "k" || e.key === "ArrowUp") {
        setFocusedIndex((prev) => Math.max(prev - 1, 0));
      } else if (e.key === "Enter") {
        if (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") {
          setSelectedChannel(listItems[focusedIndex] as Channel);
        } else if (activeTab === "groups") {
          handleSelectGroup(listItems[focusedIndex] as string);
        }
      } else if (e.key === "l" || e.key === "ArrowRight") {
        const tabs: Tab[] = ["channels", "favorites", "groups", "history", "settings"];
        const currentIndex = tabs.indexOf(activeTab);
        const nextIndex = (currentIndex + 1) % tabs.length;
        setActiveTab(tabs[nextIndex]);
        setFocusedIndex(0);
      } else if (e.key === "h" || e.key === "ArrowLeft") {
        const tabs: Tab[] = ["channels", "favorites", "groups", "history", "settings"];
        const currentIndex = tabs.indexOf(activeTab);
        const prevIndex = (currentIndex - 1 + tabs.length) % tabs.length;
        setActiveTab(tabs[prevIndex]);
        setFocusedIndex(0);
      } else if (e.key === "f") {
        if (activeTab === "channels") {
          handleToggleFavorite(listItems[focusedIndex] as Channel);
        }
      } else if (e.key === "p") {
        if (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") {
          handlePlayInMpv(listItems[focusedIndex] as Channel);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [activeTab, channels, favorites, groups, history, selectedGroup, selectedChannel, focusedIndex, listItems]);

  const renderChannelList = (channelsList: Channel[]) => (
    <ul className="channel-list">
      {channelsList.map((channel, index) => (
        <li
          key={channel.name}
          className={`channel-item ${selectedChannel?.name === channel.name ? "selected" : ""} ${focusedIndex === index ? "focused" : ""}`}
          onClick={() => setSelectedChannel(channel)}
        >
          <div className="channel-content">
            <div className="channel-logo-container">
              <CachedImage 
                src={channel.logo} 
                alt={channel.name} 
                className="channel-logo"
              />
              <div className="channel-status"></div>
            </div>
            <div className="channel-info">
              <div className="channel-header">
                <span className="channel-number">{index + 1}</span>
                <span className="channel-name">{channel.name}</span>
              </div>
              <div className="channel-badges">
                <span className="badge badge-category">{channel.group_title}</span>
                <span className="badge badge-quality">{channel.resolution || "HD"}</span>
              </div>
              <div className="channel-group">{channel.extra_info}</div>
            </div>
            <div className="channel-actions">
              <button 
                className={`action-button ${isFavorite(channel) ? "favorite" : ""}`}
                onClick={(e) => {
                  e.stopPropagation();
                  handleToggleFavorite(channel);
                }}
              >
                {isFavorite(channel) ? "★" : "☆"}
              </button>
            </div>
          </div>
        </li>
      ))}
    </ul>
  );

  const renderGroupList = () => (
    <ul className="group-list">
      <li
        className={`group-item ${selectedGroup === null ? "selected" : ""} ${focusedIndex === 0 ? "focused" : ""}`}
        onClick={() => handleSelectGroup(null)}
      >
        All Groups
      </li>
      {groups.map((group, index) => (
        <li
          key={group}
          className={`group-item ${selectedGroup === group ? "selected" : ""} ${focusedIndex === index + 1 ? "focused" : ""}`}
          onClick={() => handleSelectGroup(group)}
        >
          {group}
        </li>
      ))}
    </ul>
  );

  const renderContent = () => {
    switch (activeTab) {
      case "channels":
        return (
          <>
            {channelListName && (
              <div className="channel-list-info">
                <strong>Channel List:</strong> {channelListName}
              </div>
            )}
            <div className="search-container">
              <input
                type="text"
                className="search-input"
                placeholder="Search channels (min 3 characters)..."
                value={searchQuery}
                onChange={handleSearch}
              />
            </div>
            {searchQuery.length > 0 && searchQuery.length < 3 && (
              <div className="search-status">
                Type at least 3 characters to search...
              </div>
            )}
            {isSearching && (
              <div className="search-status">
                Searching...
              </div>
            )}
            <div className="content-list">
              {renderChannelList(filteredChannels)}
            </div>
          </>
        );
      case "favorites":
        return (
          <div className="content-list">
            {renderChannelList(favorites)}
          </div>
        );
      case "groups":
        return (
          <div className="content-list">
            {renderGroupList()}
          </div>
        );
      case "history":
        return (
          <div className="content-list">
            {renderChannelList(history)}
          </div>
        );
      case "settings":
        return (
          <div className="settings-container">
            <Settings onSelectList={handleSelectChannelList} />
          </div>
        );
      default:
        return null;
    }
  };

  const getTabTitle = () => {
    switch (activeTab) {
      case "channels":
        return "Channels";
      case "favorites":
        return "Favorites";
      case "groups":
        return "Groups";
      case "history":
        return "History";
      case "settings":
        return "Settings";
      default:
        return "IPTV Player";
    }
  };

  const getTabSubtitle = () => {
    switch (activeTab) {
      case "channels":
        return `${filteredChannels.length} channels available`;
      case "favorites":
        return `${favorites.length} favorite channels`;
      case "groups":
        return `${groups.length} groups available`;
      case "history":
        return `${history.length} recently watched`;
      case "settings":
        return "Application settings";
      default:
        return "";
    }
  };

  return (
    <div className="container">
      {/* Navigation Sidebar */}
      <div className="nav-sidebar">
        <div className="app-header">
          <div className="app-logo">
            <div className="app-logo-icon">TV</div>
            <h1 className="app-title">IPTV Pro</h1>
          </div>
          <nav className="nav-menu">
            <button 
              className={`nav-button ${activeTab === "channels" ? "active" : ""}`}
              onClick={() => setActiveTab("channels")}
            >
              <TvIcon />
              Channels
            </button>
            <button 
              className={`nav-button ${activeTab === "favorites" ? "active" : ""}`}
              onClick={() => setActiveTab("favorites")}
            >
              <HeartIcon />
              Favorites
            </button>
            <button 
              className={`nav-button ${activeTab === "groups" ? "active" : ""}`}
              onClick={() => setActiveTab("groups")}
            >
              <UsersIcon />
              Groups
            </button>
            <button 
              className={`nav-button ${activeTab === "history" ? "active" : ""}`}
              onClick={() => setActiveTab("history")}
            >
              <HistoryIcon />
              History
            </button>
            <button 
              className={`nav-button ${activeTab === "settings" ? "active" : ""}`}
              onClick={() => setActiveTab("settings")}
            >
              <SettingsIcon />
              Settings
            </button>
          </nav>
        </div>
      </div>

      {/* Main Content Area */}
      <div className="main-content">
        {activeTab === "settings" ? (
          /* Settings take full width */
          <div className="settings-full-width">
            <div className="section-header">
              <h2 className="section-title">{getTabTitle()}</h2>
              <p className="section-subtitle">{getTabSubtitle()}</p>
            </div>
            {renderContent()}
          </div>
        ) : (
          <>
            {/* Channels List */}
            <div className="channels-section">
              <div className="section-header">
                <h2 className="section-title">{getTabTitle()}</h2>
                <p className="section-subtitle">{getTabSubtitle()}</p>
              </div>
              {renderContent()}
            </div>

            {/* Video Player and Information Area */}
            <div className="video-section">
              {/* Video Preview */}
              <div className="video-preview">
                <div className="video-container">
                  {selectedChannel ? (
                    <>
                      <video ref={videoRef} className="video-player" controls />
                      <div className="video-controls">
                        <div className="video-status">
                          <div className="status-dot"></div>
                          <span className="status-text">Live</span>
                        </div>
                        <div className="quality-badge">
                          {selectedChannel.resolution || "HD"}
                        </div>
                      </div>
                    </>
                  ) : (
                    <div className="video-placeholder">
                      <PlayIcon />
                      <div className="video-placeholder-text">Preview Window</div>
                      <div className="video-placeholder-channel">Select a channel to start watching</div>
                    </div>
                  )}
                </div>
              </div>

              {/* Channel Information */}
              {selectedChannel && (
                <div className="channel-details">
                  <div className="channel-details-content">
                    <div className="channel-main-info">
                      <CachedImage 
                        src={selectedChannel.logo} 
                        alt={selectedChannel.name}
                        className="channel-details-logo"
                      />
                      <div className="channel-meta">
                        <div className="channel-title-row">
                          <h1 className="channel-details-title">{selectedChannel.name}</h1>
                          <span className="channel-number-badge">CH {channels.indexOf(selectedChannel) + 1}</span>
                        </div>
                        <div className="channel-meta-row">
                          <div className="meta-item">
                            <SignalIcon />
                            {selectedChannel.resolution || "HD"}
                          </div>
                          <div className="meta-item">
                            <StarIcon />
                            4.5
                          </div>
                          <span className="badge badge-category">{selectedChannel.group_title}</span>
                        </div>
                      </div>
                    </div>

                    <div className="separator"></div>

                    <div className="actions-section">
                      <button 
                        className="primary-button"
                        onClick={() => handlePlayInMpv(selectedChannel)}
                      >
                        Play in MPV
                      </button>
                      <button 
                        className="secondary-button"
                        onClick={() => handleToggleFavorite(selectedChannel)}
                      >
                        {isFavorite(selectedChannel) ? "Remove from Favorites" : "Add to Favorites"}
                      </button>
                    </div>

                    <div className="details-grid">
                      <div className="detail-item">
                        <div className="detail-label">Group</div>
                        <div className="detail-value">{selectedChannel.group_title}</div>
                      </div>
                      <div className="detail-item">
                        <div className="detail-label">TVG ID</div>
                        <div className="detail-value">{selectedChannel.tvg_id || "N/A"}</div>
                      </div>
                      <div className="detail-item">
                        <div className="detail-label">Resolution</div>
                        <div className="detail-value">{selectedChannel.resolution || "HD"}</div>
                      </div>
                      <div className="detail-item">
                        <div className="detail-label">Extra Info</div>
                        <div className="detail-value">{selectedChannel.extra_info || "No additional information"}</div>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default App;
