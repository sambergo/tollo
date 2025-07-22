import CachedImage from "./CachedImage";
import { SignalIcon, StarIcon } from "./Icons";
import { useChannelStore, useUIStore, GroupDisplayMode } from "../stores";

export default function ChannelDetails() {
  const {
    selectedChannel,
    channels,
    favorites,
    toggleFavorite,
    playInExternalPlayer,
  } = useChannelStore();
  const { setSelectedGroup, setActiveTab, setGroupDisplayMode } = useUIStore();

  if (!selectedChannel) {
    return (
      <div className="channel-details">
        <div className="channel-details-content">
          <p>No channel selected</p>
        </div>
      </div>
    );
  }

  const isFavorite = favorites.some((fav) => fav.name === selectedChannel.name);

  const handleFilterByGroup = () => {
    if (selectedChannel?.group_title) {
      setGroupDisplayMode(GroupDisplayMode.AllGroups);
      setSelectedGroup(selectedChannel.group_title);
      setActiveTab("channels");
    }
  };

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
              <span className="channel-number-badge">
                CH {channels.indexOf(selectedChannel) + 1}
              </span>
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
              <span className="badge badge-category">
                {selectedChannel.group_title}
              </span>
            </div>
          </div>
        </div>

        <div className="separator"></div>

        <div className="actions-section">
          <button
            className="primary-button"
            onClick={() => playInExternalPlayer(selectedChannel)}
          >
            Play in External Player
          </button>
          <button
            className="secondary-button"
            onClick={() => toggleFavorite(selectedChannel)}
          >
            {isFavorite ? "Remove from Favorites" : "Add to Favorites"}
          </button>
        </div>

        <div className="details-grid">
          <div className="detail-item">
            <div className="detail-label">Group</div>
            <div className="detail-value-with-action">
              <span className="detail-text">{selectedChannel.group_title}</span>
              <button
                className="detail-action-button"
                onClick={handleFilterByGroup}
                title="Filter channels by this group"
              >
                <svg
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                >
                  <path d="M10 18h4v-2h-4v2zM3 6v2h18V6H3zm3 7h12v-2H6v2z" />
                </svg>
                Filter
              </button>
            </div>
          </div>
          <div className="detail-item">
            <div className="detail-label">TVG ID</div>
            <div className="detail-value">
              {selectedChannel.tvg_id || "N/A"}
            </div>
          </div>
          <div className="detail-item">
            <div className="detail-label">Resolution</div>
            <div className="detail-value">
              {selectedChannel.resolution || "HD"}
            </div>
          </div>
          <div className="detail-item">
            <div className="detail-label">Extra Info</div>
            <div className="detail-value">
              {selectedChannel.extra_info || "No additional information"}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
