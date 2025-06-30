import CachedImage from "./CachedImage";
import { SignalIcon, StarIcon } from "./Icons";
import type { Channel } from "./ChannelList";

interface ChannelDetailsProps {
  selectedChannel: Channel;
  channels: Channel[];
  isFavorite: boolean;
  onPlayInMpv: (channel: Channel) => void;
  onToggleFavorite: (channel: Channel) => void;
}

export default function ChannelDetails({ 
  selectedChannel, 
  channels, 
  isFavorite, 
  onPlayInMpv, 
  onToggleFavorite 
}: ChannelDetailsProps) {
  return (
    <div className="channel-details">
      <div className="channel-details-content">
        <div className="channel-main-info">
          <CachedImage 
            src={selectedChannel.logo} 
            alt={selectedChannel.name}
            className="channel-details-logo"
          />
          <div className="channel-meta">
            <div className="channel-title-row">
              <h1 className="channel-details-title">{selectedChannel.name}</h1>
              <span className="channel-number-badge">CH {channels.indexOf(selectedChannel) + 1}</span>
            </div>
            <div className="channel-meta-row">
              <div className="meta-item">
                <SignalIcon />
                {selectedChannel.resolution || "HD"}
              </div>
              <div className="meta-item">
                <StarIcon />
                4.5
              </div>
              <span className="badge badge-category">{selectedChannel.group_title}</span>
            </div>
          </div>
        </div>

        <div className="separator"></div>

        <div className="actions-section">
          <button 
            className="primary-button"
            onClick={() => onPlayInMpv(selectedChannel)}
          >
            Play in MPV
          </button>
          <button 
            className="secondary-button"
            onClick={() => onToggleFavorite(selectedChannel)}
          >
            {isFavorite ? "Remove from Favorites" : "Add to Favorites"}
          </button>
        </div>

        <div className="details-grid">
          <div className="detail-item">
            <div className="detail-label">Group</div>
            <div className="detail-value">{selectedChannel.group_title}</div>
          </div>
          <div className="detail-item">
            <div className="detail-label">TVG ID</div>
            <div className="detail-value">{selectedChannel.tvg_id || "N/A"}</div>
          </div>
          <div className="detail-item">
            <div className="detail-label">Resolution</div>
            <div className="detail-value">{selectedChannel.resolution || "HD"}</div>
          </div>
          <div className="detail-item">
            <div className="detail-label">Extra Info</div>
            <div className="detail-value">{selectedChannel.extra_info || "No additional information"}</div>
          </div>
        </div>
      </div>
    </div>
  );
} 