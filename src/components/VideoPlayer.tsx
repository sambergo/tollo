import { forwardRef, useEffect, useRef } from "react";
import { PlayIcon } from "./Icons";
import { useChannelStore } from "../stores";
import { useSettingsStore } from "../stores";

const VideoPlayer = forwardRef<HTMLVideoElement, {}>((_, ref) => {
  const { selectedChannel, isMpvPlaying, setIsMpvPlaying } = useChannelStore();
  const { muteOnStart, showControls, autoplay } = useSettingsStore();
  const previousChannelRef = useRef(selectedChannel);

  // Reset MPV playing state when a different channel is selected
  useEffect(() => {
    if (
      selectedChannel &&
      previousChannelRef.current &&
      selectedChannel.name !== previousChannelRef.current.name &&
      isMpvPlaying
    ) {
      setIsMpvPlaying(false);
    }
    previousChannelRef.current = selectedChannel;
  }, [selectedChannel, isMpvPlaying, setIsMpvPlaying]);

  return (
    <div className="video-preview">
      <div className="video-container">
        {selectedChannel && !isMpvPlaying ? (
          <>
            <video
              ref={ref}
              className="video-player"
              controls={showControls}
              muted={muteOnStart}
              autoPlay={autoplay}
            />
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
            <div className="video-placeholder-channel">
              Select a channel to start watching
            </div>
          </div>
        )}
      </div>
    </div>
  );
});

VideoPlayer.displayName = "VideoPlayer";

export default VideoPlayer;
