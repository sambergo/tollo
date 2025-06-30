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
  onSelectChannel: (channel: Channel) => void;
  onToggleFavorite: (channel: Channel) => void;
}

export default function ChannelList({ 
  channels, 
  selectedChannel, 
  focusedIndex, 
  favorites,
  onSelectChannel, 
  onToggleFavorite 
}: ChannelListProps) {
  const isFavorite = (channel: Channel) => {
    return favorites.some((fav) => fav.name === channel.name);
  };

  return (
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
  );
} 