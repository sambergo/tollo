import { forwardRef, useEffect } from "react";
import { PlayIcon } from "./Icons";
import { useChannelStore } from "../stores";


const VideoPlayer = forwardRef<HTMLVideoElement, {}>(
  (_, ref) => {
    const { selectedChannel, isMpvPlaying, setIsMpvPlaying } = useChannelStore();

    // Reset MPV playing state when a new channel is selected
    useEffect(() => {
      if (selectedChannel && isMpvPlaying) {
        setIsMpvPlaying(false);
      }
    }, [selectedChannel, isMpvPlaying, setIsMpvPlaying]);

    return (
      <div className="video-preview">
        <div className="video-container">
          {selectedChannel && !isMpvPlaying ? (
            <>
              <video ref={ref} className="video-player" controls />
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
          ) : isMpvPlaying ? (
            <div className="video-placeholder">
              <PlayIcon />
              <div className="video-placeholder-text">Playing in MPV</div>
              <div className="video-placeholder-channel">{selectedChannel?.name}</div>
            </div>
          ) : (
            <div className="video-placeholder">
              <PlayIcon />
              <div className="video-placeholder-text">Preview Window</div>
              <div className="video-placeholder-channel">Select a channel to start watching</div>
            </div>
          )}
        </div>
      </div>
    );
  }
);

VideoPlayer.displayName = "VideoPlayer";

export default VideoPlayer; 