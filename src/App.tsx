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

function App() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [favorites, setFavorites] = useState<Channel[]>([]);
  const [groups, setGroups] = useState<string[]>([]);
  const [history, setHistory] = useState<Channel[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [selectedChannel, setSelectedChannel] = useState<Channel | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>("channels");
  const [focusedIndex, setFocusedIndex] = useState(0);
  const [selectedChannelListId, setSelectedChannelListId] = useState<number | null>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);
  const channelListName = useChannelListName(selectedChannelListId);

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
    if (query === "") {
      fetchChannels(selectedChannelListId);
    } else {
      const searchedChannels = await invoke<Channel[]>("search_channels", { 
        query, 
        id: selectedChannelListId 
      });
      setChannels(searchedChannels);
    }
  }

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
    searchChannels(e.target.value);
  };

  const handleSelectChannelList = (id: number) => {
    setSelectedChannelListId(id);
    setActiveTab("channels");
  };

  const filteredChannels = selectedGroup
    ? channels.filter((channel) => channel.group_title === selectedGroup)
    : channels;

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

  const renderContent = () => {
    switch (activeTab) {
      case "channels":
        return (
          <>
            {channelListName && (
              <div className="channel-list-name">
                <strong>Channel List:</strong> {channelListName}
              </div>
            )}
            <input
              type="text"
              placeholder="Search channels..."
              value={searchQuery}
              onChange={handleSearch}
            />
            <ul>
              {filteredChannels.map((channel, index) => (
                <li
                  key={channel.name}
                  className={`${selectedChannel?.name === channel.name ? "selected" : ""} ${focusedIndex === index ? "focused" : ""}`}
                  onClick={() => setSelectedChannel(channel)}
                >
                  <CachedImage src={channel.logo} alt={channel.name} />
                  <span>{channel.name}</span>
                  <button onClick={() => handleToggleFavorite(channel)}>
                    {isFavorite(channel) ? "Unfavorite" : "Favorite"}
                  </button>
                </li>
              ))}
            </ul>
          </>
        );
      case "favorites":
        return (
          <ul>
            {favorites.map((channel, index) => (
              <li
                key={channel.name}
                className={`${selectedChannel?.name === channel.name ? "selected" : ""} ${focusedIndex === index ? "focused" : ""}`}
                onClick={() => setSelectedChannel(channel)}
              >
                <CachedImage src={channel.logo} alt={channel.name} />
                <span>{channel.name}</span>
              </li>
            ))}
          </ul>
        );
      case "groups":
        return (
          <ul>
            <li
              className={`${selectedGroup === null ? "selected" : ""} ${focusedIndex === 0 ? "focused" : ""}`}
              onClick={() => handleSelectGroup(null)}
            >
              All
            </li>
            {groups.map((group, index) => (
              <li
                key={group}
                className={`${selectedGroup === group ? "selected" : ""} ${focusedIndex === index + 1 ? "focused" : ""}`}
                onClick={() => handleSelectGroup(group)}
              >
                {group}
              </li>
            ))}
          </ul>
        );
      case "history":
        return (
          <ul>
            {history.map((channel, index) => (
              <li
                key={channel.name}
                className={`${selectedChannel?.name === channel.name ? "selected" : ""} ${focusedIndex === index ? "focused" : ""}`}
                onClick={() => setSelectedChannel(channel)}
              >
                <CachedImage src={channel.logo} alt={channel.name} />
                <span>{channel.name}</span>
              </li>
            ))}
          </ul>
        );
      case "settings":
        return <Settings onSelectList={handleSelectChannelList} />;
      default:
        return null;
    }
  };

  return (
    <div className="container">
      <div className="sidebar">
        <div className="navbar">
          <button onClick={() => setActiveTab("channels")}>Channels</button>
          <button onClick={() => setActiveTab("favorites")}>Favorites</button>
          <button onClick={() => setActiveTab("groups")}>Groups</button>
          <button onClick={() => setActiveTab("history")}>History</button>
          <button onClick={() => setActiveTab("settings")}>Settings</button>
        </div>
        <div className="content">{renderContent()}</div>
      </div>
      <div className="video-player">
        {selectedChannel && (
          <>
            <video ref={videoRef} controls></video>
            <div className="video-info">
              <h3>{selectedChannel.name}</h3>
              <p>
                <strong>Group:</strong> {selectedChannel.group_title}
              </p>
              <p>
                <strong>TVG ID:</strong> {selectedChannel.tvg_id}
              </p>
              <p>
                <strong>Resolution:</strong> {selectedChannel.resolution}
              </p>
              <p>
                <strong>Extra Info:</strong> {selectedChannel.extra_info}
              </p>
              <button onClick={() => handlePlayInMpv(selectedChannel)}>Play in MPV</button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default App;
