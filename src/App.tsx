import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import Hls from "hls.js";
import "./App.css";

interface Channel {
  name: string;
  logo: string;
  url: string;
  group_title: string;
}

type Tab = "channels" | "favorites" | "groups";

function App() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [favorites, setFavorites] = useState<Channel[]>([]);
  const [groups, setGroups] = useState<string[]>([]);
  const [selectedGroup, setSelectedGroup] = useState<string | null>(null);
  const [selectedChannel, setSelectedChannel] = useState<Channel | null>(null);
  const [activeTab, setActiveTab] = useState<Tab>("channels");
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);

  async function fetchChannels() {
    const fetchedChannels = await invoke<Channel[]>("get_channels");
    setChannels(fetchedChannels);
    setSelectedChannel(fetchedChannels[0]);
  }

  async function fetchFavorites() {
    const fetchedFavorites = await invoke<Channel[]>("get_favorites");
    setFavorites(fetchedFavorites);
  }

  async function fetchGroups() {
    const fetchedGroups = await invoke<string[]>("get_groups");
    setGroups(fetchedGroups);
  }

  useEffect(() => {
    fetchChannels();
    fetchFavorites();
    fetchGroups();
  }, []);

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

  const handlePlayInMpv = () => {
    if (selectedChannel) {
      if (videoRef.current) {
        videoRef.current.pause();
        videoRef.current.src = "";
      }
      if (hlsRef.current) {
        hlsRef.current.destroy();
      }
      invoke("play_channel", { url: selectedChannel.url });
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

  const filteredChannels = selectedGroup
    ? channels.filter((channel) => channel.group_title === selectedGroup)
    : channels;

  const renderContent = () => {
    switch (activeTab) {
      case "channels":
        return (
          <ul>
            {filteredChannels.map((channel) => (
              <li
                key={channel.name}
                className={selectedChannel?.name === channel.name ? "selected" : ""}
                onClick={() => setSelectedChannel(channel)}
              >
                <img src={channel.logo} alt={channel.name} />
                <span>{channel.name}</span>
                <button onClick={() => handleToggleFavorite(channel)}>
                  {isFavorite(channel) ? "Unfavorite" : "Favorite"}
                </button>
              </li>
            ))}
          </ul>
        );
      case "favorites":
        return (
          <ul>
            {favorites.map((channel) => (
              <li
                key={channel.name}
                className={selectedChannel?.name === channel.name ? "selected" : ""}
                onClick={() => setSelectedChannel(channel)}
              >
                <img src={channel.logo} alt={channel.name} />
                <span>{channel.name}</span>
              </li>
            ))}
          </ul>
        );
      case "groups":
        return (
          <ul>
            <li
              className={selectedGroup === null ? "selected" : ""}
              onClick={() => handleSelectGroup(null)}
            >
              All
            </li>
            {groups.map((group) => (
              <li
                key={group}
                className={selectedGroup === group ? "selected" : ""}
                onClick={() => handleSelectGroup(group)}
              >
                {group}
              </li>
            ))}
          </ul>
        );
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
        </div>
        <div className="content">{renderContent()}</div>
      </div>
      <div className="video-player">
        {selectedChannel && (
          <>
            <video ref={videoRef} controls></video>
            <div className="video-info">
              <h3>{selectedChannel.name}</h3>
              <p>{selectedChannel.group_title}</p>
              <button onClick={handlePlayInMpv}>Play in MPV</button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default App;