import { useState, useEffect, useRef } from "react";
import {
  DndContext,
  closestCenter,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { restrictToVerticalAxis } from "@dnd-kit/modifiers";
import { CSS } from "@dnd-kit/utilities";
import CachedImage from "./CachedImage";
import { useChannelStore, useUIStore } from "../stores";
import type { Channel } from "./ChannelList";

const CHANNELS_PER_PAGE = 200;

interface SortableItemProps {
  channel: Channel;
  globalIndex: number;
  isSelected: boolean;
  isFocused: boolean;
  isFavorite: boolean;
  onSelect: () => void;
  onToggleFavorite: () => void;
  onMoveUp: () => void;
  onMoveDown: () => void;
  isFirst: boolean;
  isLast: boolean;
}

function SortableItem({
  channel,
  globalIndex,
  isSelected,
  isFocused,
  isFavorite,
  onSelect,
  onToggleFavorite,
  onMoveUp,
  onMoveDown,
  isFirst,
  isLast,
}: SortableItemProps) {
  const {
    attributes,
    listeners,
    setNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: channel.name });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <li
      ref={setNodeRef}
      style={style}
      className={`channel-item ${isSelected ? "selected" : ""} ${isFocused ? "focused" : ""} ${isDragging ? "dragging" : ""}`}
      onClick={onSelect}
    >
      <div className="channel-content">
        <div
          className="drag-handle"
          {...attributes}
          {...listeners}
          title="Drag to reorder"
        >
          ⠿
        </div>
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
            <span className="badge badge-category">{channel.group_title}</span>
            <span className="badge badge-quality">
              {channel.resolution || "HD"}
            </span>
          </div>
          <div className="channel-group">{channel.extra_info}</div>
        </div>
        <div className="channel-actions">
          <button
            className="move-btn"
            onClick={(e) => {
              e.stopPropagation();
              onMoveUp();
            }}
            disabled={isFirst}
            title="Move up"
          >
            ▲
          </button>
          <button
            className="move-btn"
            onClick={(e) => {
              e.stopPropagation();
              onMoveDown();
            }}
            disabled={isLast}
            title="Move down"
          >
            ▼
          </button>
          <button
            className={`action-button ${isFavorite ? "favorite" : ""}`}
            onClick={(e) => {
              e.stopPropagation();
              onToggleFavorite();
            }}
          >
            {isFavorite ? "★" : "☆"}
          </button>
        </div>
      </div>
    </li>
  );
}

export default function FavoritesChannelList() {
  const [currentPage, setCurrentPage] = useState(1);
  const channelListRef = useRef<HTMLUListElement>(null);

  const {
    favorites,
    selectedChannel,
    setSelectedChannel,
    toggleFavorite,
    reorderFavorites,
    moveFavorite,
  } = useChannelStore();

  const { focusedIndex, setFocusedIndex } = useUIStore();

  useEffect(() => {
    setCurrentPage(1);
  }, [favorites.length]);

  useEffect(() => {
    if (favorites.length === 0) return;

    const requiredPage = Math.ceil((focusedIndex + 1) / CHANNELS_PER_PAGE);
    if (requiredPage !== currentPage) {
      setCurrentPage(requiredPage);
    }

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
  }, [focusedIndex, favorites.length, currentPage]);

  const totalPages = Math.ceil(favorites.length / CHANNELS_PER_PAGE);
  const startIndex = (currentPage - 1) * CHANNELS_PER_PAGE;
  const endIndex = startIndex + CHANNELS_PER_PAGE;
  const currentFavorites = favorites.slice(startIndex, endIndex);

  const handlePageChange = (page: number) => {
    setCurrentPage(page);
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const oldIndex = favorites.findIndex((ch) => ch.name === active.id);
    const newIndex = favorites.findIndex((ch) => ch.name === over.id);
    if (oldIndex === -1 || newIndex === -1) return;

    const updated = [...favorites];
    const [moved] = updated.splice(oldIndex, 1);
    updated.splice(newIndex, 0, moved);
    reorderFavorites(updated);
  };

  const getPageNumbers = () => {
    const pages = [];
    const maxVisiblePages = 5;

    let startPage = Math.max(1, currentPage - Math.floor(maxVisiblePages / 2));
    let endPage = Math.min(totalPages, startPage + maxVisiblePages - 1);

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
      <div className="pagination-info">
        <span className="channel-count">
          Showing {startIndex + 1}-{Math.min(endIndex, favorites.length)} of{" "}
          {favorites.length} favorites
          {totalPages > 1 && ` (Page ${currentPage} of ${totalPages})`}
        </span>
      </div>

      <DndContext
        collisionDetection={closestCenter}
        modifiers={[restrictToVerticalAxis]}
        onDragEnd={handleDragEnd}
      >
        <SortableContext
          items={currentFavorites.map((ch) => ch.name)}
          strategy={verticalListSortingStrategy}
        >
          <ul className="channel-list" ref={channelListRef}>
            {currentFavorites.map((channel, index) => {
              const globalIndex = startIndex + index;
              return (
                <SortableItem
                  key={channel.name}
                  channel={channel}
                  globalIndex={globalIndex}
                  isSelected={selectedChannel?.name === channel.name}
                  isFocused={focusedIndex === globalIndex}
                  isFavorite={true}
                  onSelect={() => {
                    setSelectedChannel(channel);
                    setFocusedIndex(globalIndex);
                  }}
                  onToggleFavorite={() => toggleFavorite(channel)}
                  onMoveUp={() => {
                    if (globalIndex > 0) {
                      moveFavorite(globalIndex, globalIndex - 1);
                      setFocusedIndex(globalIndex - 1);
                    }
                  }}
                  onMoveDown={() => {
                    if (globalIndex < favorites.length - 1) {
                      moveFavorite(globalIndex, globalIndex + 1);
                      setFocusedIndex(globalIndex + 1);
                    }
                  }}
                  isFirst={globalIndex === 0}
                  isLast={globalIndex === favorites.length - 1}
                />
              );
            })}
          </ul>
        </SortableContext>
      </DndContext>

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
