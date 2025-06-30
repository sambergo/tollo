import CachedImage from "./CachedImage";

export interface Channel {
  name: string;
  logo: string;
  url: string;
  group_title: string;
  tvg_id: string;
  resolution: string;
  extra_info: string;
}

interface ChannelListProps {
  channels: Channel[];
  selectedChannel: Channel | null;
  focusedIndex: number;
  favorites: Channel[];
  selectedGroup?: string | null;
  onSelectChannel: (channel: Channel) => void;
  onToggleFavorite: (channel: Channel) => void;
  onClearGroupFilter?: () => void;
}

export default function ChannelList({ 
  channels, 
  selectedChannel, 
  focusedIndex, 
  favorites,
  selectedGroup,
  onSelectChannel, 
  onToggleFavorite,
  onClearGroupFilter
}: ChannelListProps) {
  const isFavorite = (channel: Channel) => {
    return favorites.some((fav) => fav.name === channel.name);
  };

  return (
    <div className="channel-list-container">
      {/* Group Filter Indicator */}
      {selectedGroup && onClearGroupFilter && (
        <div className="group-filter-indicator">
          <div className="filter-info">
            <svg className="folder-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
            <span className="filter-label">Group:</span>
            <span className="filter-value">{selectedGroup}</span>
          </div>
          <button 
            className="clear-filter-btn"
            onClick={onClearGroupFilter}
            title="Clear group filter"
          >
            <svg className="close-icon" width="12" height="12" viewBox="0 0 24 24" fill="currentColor" stroke="none">
              <path d="M18.3 5.71a.996.996 0 0 0-1.41 0L12 10.59 7.11 5.7A.996.996 0 1 0 5.7 7.11L10.59 12 5.7 16.89a.996.996 0 1 0 1.41 1.41L12 13.41l4.89 4.89a.996.996 0 1 0 1.41-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4z"/>
            </svg>
          </button>
        </div>
      )}

      <ul className="channel-list">
        {channels.map((channel, index) => (
          <li
            key={channel.name}
            className={`channel-item ${selectedChannel?.name === channel.name ? "selected" : ""} ${focusedIndex === index ? "focused" : ""}`}
            onClick={() => onSelectChannel(channel)}
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
                    onToggleFavorite(channel);
                  }}
                >
                  {isFavorite(channel) ? "★" : "☆"}
                </button>
              </div>
            </div>
          </li>
        ))}
      </ul>
    </div>
  );
} 