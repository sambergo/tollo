import React from "react";
import { useChannelStore } from "../stores/channelStore";
import "./css/loading-progress.css";

interface ChannelLoadingProgressProps {
  className?: string;
  showWhenComplete?: boolean;
}

export const ChannelLoadingProgress: React.FC<ChannelLoadingProgressProps> = ({
  className = "",
  showWhenComplete = false,
}) => {
  const { loadingProgress, isAsyncLoading } = useChannelStore();

  // Don't render if not loading and not showing complete state
  if (!loadingProgress && !isAsyncLoading) {
    return null;
  }

  // Don't render if complete and showWhenComplete is false
  if (loadingProgress?.is_complete && !showWhenComplete) {
    return null;
  }

  const progress = loadingProgress?.progress ?? 0;
  const message = loadingProgress?.message ?? "Loading...";
  const channelCount = loadingProgress?.channel_count;
  const isComplete = loadingProgress?.is_complete ?? false;

  return (
    <div className={`channel-loading-progress ${className}`}>
      <div className="loading-progress-content">
        <div className="loading-progress-message">
          {message}
          {channelCount && (
            <span className="channel-count">
              {channelCount.toLocaleString()} channels
            </span>
          )}
        </div>

        <div className="loading-progress-bar-container">
          <div
            className={`loading-progress-bar ${isComplete ? "complete" : ""}`}
            style={{
              width: `${Math.max(progress * 100, 5)}%`, // Minimum 5% for visibility
            }}
          />
          <div className="loading-progress-percentage">
            {Math.round(progress * 100)}%
          </div>
        </div>
      </div>
    </div>
  );
};

export default ChannelLoadingProgress;
