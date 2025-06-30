import { forwardRef } from "react";
import { PlayIcon } from "./Icons";
import type { Channel } from "./ChannelList";

interface VideoPlayerProps {
  selectedChannel: Channel | null;
}

const VideoPlayer = forwardRef<HTMLVideoElement, VideoPlayerProps>(
  ({ selectedChannel }, ref) => {
    return (
      <div className="video-preview">
        <div className="video-container">
          {selectedChannel ? (
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