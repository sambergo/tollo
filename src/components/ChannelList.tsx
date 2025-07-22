import { useState, useEffect, useRef } from "react";
import CachedImage from "./CachedImage";
import { useChannelStore, useUIStore } from "../stores";

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
}

const CHANNELS_PER_PAGE = 200; // Reasonable number for performance

export default function ChannelList({ channels }: ChannelListProps) {
  const [currentPage, setCurrentPage] = useState(1);
  const channelListRef = useRef<HTMLUListElement>(null);

  // Get state and actions from stores
  const { selectedChannel, favorites, setSelectedChannel, toggleFavorite } =
    useChannelStore();

  const { focusedIndex, selectedGroup, clearGroupFilter, setFocusedIndex } =
    useUIStore();

  // Reset to first page when channels change
  useEffect(() => {
    setCurrentPage(1);
  }, [channels.length, selectedGroup]);

  // Handle pagination and scrolling when focusedIndex changes
  useEffect(() => {
    if (channels.length === 0) return;

    const requiredPage = Math.ceil((focusedIndex + 1) / CHANNELS_PER_PAGE);

    // Change page if focused item is on a different page
    if (requiredPage !== currentPage) {
      setCurrentPage(requiredPage);
    }

    // Scroll focused item into view
    const focusedElement = channelListRef.current?.querySelector(
      ".channel-item.focused",
    );
    if (focusedElement) {
      focusedElement.scrollIntoView({
        behavior: "smooth",
        block: "center",
        inline: "nearest",
      });
    }
  }, [focusedIndex, channels.length, currentPage]);

  const totalPages = Math.ceil(channels.length / CHANNELS_PER_PAGE);
  const startIndex = (currentPage - 1) * CHANNELS_PER_PAGE;
  const endIndex = startIndex + CHANNELS_PER_PAGE;
  const currentChannels = channels.slice(startIndex, endIndex);

  const isFavorite = (channel: Channel) => {
    return favorites.some((fav) => fav.name === channel.name);
  };

  const handlePageChange = (page: number) => {
    setCurrentPage(page);
  };

  const handleToggleFavorite = async (channel: Channel) => {
    await toggleFavorite(channel);
  };

  const getPageNumbers = () => {
    const pages = [];
    const maxVisiblePages = 5;

    let startPage = Math.max(1, currentPage - Math.floor(maxVisiblePages / 2));
    let endPage = Math.min(totalPages, startPage + maxVisiblePages - 1);

    // Adjust startPage if we're near the end
    if (endPage - startPage < maxVisiblePages - 1) {
      startPage = Math.max(1, endPage - maxVisiblePages + 1);
    }

    for (let i = startPage; i <= endPage; i++) {
      pages.push(i);
    }

    return pages;
  };

  return (
    <div className="channel-list-container">
      {/* Group Filter Indicator */}
      {selectedGroup && (
        <div className="group-filter-indicator">
          <div className="filter-info">
            <svg
              className="folder-icon"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            <span className="filter-label">Group:</span>
            <span className="filter-value">{selectedGroup}</span>
          </div>
          <button
            className="clear-filter-btn"
            onClick={clearGroupFilter}
            title="Clear group filter"
          >
            <svg
              className="close-icon"
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="currentColor"
              stroke="none"
            >
              <path d="M18.3 5.71a.996.996 0 0 0-1.41 0L12 10.59 7.11 5.7A.996.996 0 1 0 5.7 7.11L10.59 12 5.7 16.89a.996.996 0 1 0 1.41 1.41L12 13.41l4.89 4.89a.996.996 0 1 0 1.41-1.41L13.41 12l4.89-4.89c.38-.38.38-1.02 0-1.4z" />
            </svg>
          </button>
        </div>
      )}

      {/* Pagination Info */}
      <div className="pagination-info">
        <span className="channel-count">
          Showing {startIndex + 1}-{Math.min(endIndex, channels.length)} of{" "}
          {channels.length} channels
          {totalPages > 1 && ` (Page ${currentPage} of ${totalPages})`}
        </span>
      </div>

      <ul className="channel-list" ref={channelListRef}>
        {currentChannels.map((channel, index) => {
          const globalIndex = startIndex + index;
          return (
            <li
              key={`${channel.name}-${globalIndex}`}
              className={`channel-item ${selectedChannel?.name === channel.name ? "selected" : ""} ${focusedIndex === globalIndex ? "focused" : ""}`}
              onClick={() => {
                setSelectedChannel(channel);
                // Sync focusedIndex with clicked channel
                setFocusedIndex(globalIndex);
              }}
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
                    <span className="channel-number">{globalIndex + 1}</span>
                    <span className="channel-name">{channel.name}</span>
                  </div>
                  <div className="channel-badges">
                    <span className="badge badge-category">
                      {channel.group_title}
                    </span>
                    <span className="badge badge-quality">
                      {channel.resolution || "HD"}
                    </span>
                  </div>
                  <div className="channel-group">{channel.extra_info}</div>
                </div>
                <div className="channel-actions">
                  <button
                    className={`action-button ${isFavorite(channel) ? "favorite" : ""}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleToggleFavorite(channel);
                    }}
                  >
                    {isFavorite(channel) ? "★" : "☆"}
                  </button>
                </div>
              </div>
            </li>
          );
        })}
      </ul>

      {/* Pagination Controls */}
      {totalPages > 1 && (
        <div className="pagination-controls">
          <button
            className="pagination-btn"
            onClick={() => handlePageChange(1)}
            disabled={currentPage === 1}
            title="First page"
          >
            ««
          </button>
          <button
            className="pagination-btn"
            onClick={() => handlePageChange(currentPage - 1)}
            disabled={currentPage === 1}
            title="Previous page"
          >
            ‹
          </button>

          {getPageNumbers().map((page) => (
            <button
              key={page}
              className={`pagination-btn ${page === currentPage ? "active" : ""}`}
              onClick={() => handlePageChange(page)}
            >
              {page}
            </button>
          ))}

          <button
            className="pagination-btn"
            onClick={() => handlePageChange(currentPage + 1)}
            disabled={currentPage === totalPages}
            title="Next page"
          >
            ›
          </button>
          <button
            className="pagination-btn"
            onClick={() => handlePageChange(totalPages)}
            disabled={currentPage === totalPages}
            title="Last page"
          >
            »»
          </button>
        </div>
      )}
    </div>
  );
}
