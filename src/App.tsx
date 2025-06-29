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

function App() {
  const [channels, setChannels] = useState<Channel[]>([]);
  const [selectedChannel, setSelectedChannel] = useState<Channel | null>(null);
  const videoRef = useRef<HTMLVideoElement>(null);
  const hlsRef = useRef<Hls | null>(null);

  useEffect(() => {
    async function fetchChannels() {
      const fetchedChannels = await invoke<Channel[]>("get_channels");
      setChannels(fetchedChannels);
      setSelectedChannel(fetchedChannels[0]);
    }
    fetchChannels();
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

  return (
    <div className="container">
      <div className="channel-list">
        <h2>Channels</h2>
        <ul>
          {channels.map((channel) => (
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
