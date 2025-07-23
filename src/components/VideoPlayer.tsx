import { forwardRef, useEffect, useRef, useState } from "react";
import { PlayIcon } from "./Icons";
import { useChannelStore } from "../stores";
import { useSettingsStore } from "../stores";

const VideoPlayer = forwardRef<HTMLVideoElement, {}>((_, ref) => {
  const {
    selectedChannel,
    isExternalPlayerPlaying,
    setIsExternalPlayerPlaying,
  } = useChannelStore();
  const { muteOnStart, showControls, autoplay } = useSettingsStore();
  const previousChannelRef = useRef(selectedChannel);
  const [codecWarning, setCodecWarning] = useState(false);

  // Reset external player playing state when a different channel is selected
  useEffect(() => {
    if (
      selectedChannel &&
      previousChannelRef.current &&
      selectedChannel.name !== previousChannelRef.current.name &&
      isExternalPlayerPlaying
    ) {
      setIsExternalPlayerPlaying(false);
    }
    previousChannelRef.current = selectedChannel;
  }, [selectedChannel, isExternalPlayerPlaying, setIsExternalPlayerPlaying]);

  return (
    <div className="video-preview">
      <div className="video-container">
        {selectedChannel && !isExternalPlayerPlaying ? (
          <>
            <video
              ref={ref}
              className="video-player"
              controls={showControls}
              muted={muteOnStart}
              autoPlay={autoplay}
              onError={() => setCodecWarning(true)}
              onLoadStart={() => setCodecWarning(false)}
            />
            {codecWarning && (
              <div className="codec-warning">
                ⚠️ Video codec issue detected. Install GStreamer plugins: gstreamer1.0-plugins-bad gstreamer1.0-libav
              </div>
            )}
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
