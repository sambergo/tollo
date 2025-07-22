import ChannelList, { type Channel } from "./ChannelList";
import GroupList from "./GroupList";
import ChannelLoadingProgress from "./ChannelLoadingProgress";
import {
  useChannelStore,
  useUIStore,
  useSearchStore,
  useSettingsStore,
} from "../stores";
import { useEffect } from "react";

interface MainContentProps {
  filteredChannels: Channel[];
}

// Loading indicator component
const LoadingChannelList = () => (
  <div className="loading-channel-list">
    <div className="loading-content">
      <div className="loading-spinner-large">
        <div className="spinner-large"></div>
      </div>
      <h3>Loading Channel List</h3>
      <p>Setting up channels and groups...</p>
    </div>
  </div>
);

export default function MainContent({ filteredChannels }: MainContentProps) {
  // Get state from stores
  const {
    favorites,
    groups,
    history,
    isLoadingChannelList,
    selectedChannelListId,
    loadingProgress,
    isAsyncLoading,
  } = useChannelStore();

  const { activeTab } = useUIStore();

  const { searchQuery, isSearching, setSearchQuery } = useSearchStore();

  const { channelListName, getChannelListName } = useSettingsStore();

  useEffect(() => {
    if (selectedChannelListId !== null) {
      getChannelListName(selectedChannelListId);
    }
  }, [selectedChannelListId, getChannelListName]);

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchQuery(e.target.value);
  };

  const handleClearSearch = () => {
    setSearchQuery("");
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.ctrlKey) {
      switch (e.key) {
        case "w":
          e.preventDefault();
          // Remove last word
          const input = e.currentTarget;
          const value = input.value;
          const cursorPos = input.selectionStart || 0;
          const beforeCursor = value.substring(0, cursorPos);
          const afterCursor = value.substring(cursorPos);

          // Find the start of the last word before cursor
          const words = beforeCursor.trimEnd();
          const lastSpaceIndex = words.lastIndexOf(" ");
          const newBeforeCursor =
            lastSpaceIndex >= 0 ? words.substring(0, lastSpaceIndex + 1) : "";

          const newValue = newBeforeCursor + afterCursor;
          setSearchQuery(newValue);

          // Set cursor position after the removed word
          setTimeout(() => {
            input.setSelectionRange(
              newBeforeCursor.length,
              newBeforeCursor.length,
            );
          }, 0);
          break;

        case "u":
          e.preventDefault();
          // Clear entire input
          setSearchQuery("");
          break;

        case "c":
          e.preventDefault();
          // Unfocus the input
          e.currentTarget.blur();
          break;
      }
    }
  };

  const getTabTitle = () => {
    switch (activeTab) {
      case "channels":
        return channelListName ? `Channels (${channelListName})` : "Channels";
      case "favorites":
        return "Favorites";
      case "groups":
        return channelListName ? `Groups (${channelListName})` : "Groups";
      case "history":
        return "History";
      default:
        return "IPTV Player";
    }
  };

  const getTabSubtitle = () => {
    switch (activeTab) {
      case "channels":
        return `${filteredChannels.length} channels available`;
      case "favorites":
        return `${favorites.length} favorite channels`;
      case "groups":
        return `${groups.length} groups available`;
      case "history":
        return `${history.length} recently watched`;
      default:
        return "";
    }
  };

  const renderContent = () => {
    switch (activeTab) {
      case "channels":
        // Show loading progress for async operations
        if (isAsyncLoading || loadingProgress) {
          return (
            <>
              <ChannelLoadingProgress />
              {/* Still show the old loading screen if no channels are loaded yet */}
              {filteredChannels.length === 0 && <LoadingChannelList />}
            </>
          );
        }

        // Show legacy loading screen for non-async operations
        if (isLoadingChannelList) {
          return <LoadingChannelList />;
        }

        return (
          <>
            <div className="search-container">
              <div className="search-input-wrapper">
                <input
                  type="text"
                  className="search-input"
                  placeholder="Search channels (min 3 characters)..."
                  value={searchQuery}
                  onChange={handleSearch}
                  onKeyDown={handleKeyDown}
                />
                {searchQuery && (
                  <button
                    className="clear-search-btn"
                    onClick={handleClearSearch}
                    type="button"
                    title="Clear search"
                  >
                    ×
                  </button>
                )}
              </div>
            </div>
            {searchQuery.length > 0 && searchQuery.length < 3 && (
              <div className="search-status">
                Type at least 3 characters to search...
              </div>
            )}
            {isSearching && <div className="search-status">Searching...</div>}
            <div className="content-list">
              <ChannelList channels={filteredChannels} />
            </div>
          </>
        );
      case "favorites":
        return (
          <div className="content-list">
            <ChannelList channels={favorites} />
          </div>
        );
      case "groups":
        return (
          <div className="content-list">
            <GroupList />
          </div>
        );
      case "history":
        return (
          <div className="content-list">
            <ChannelList channels={history} />
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className="channels-section">
      <div className="section-header">
        <h2 className="section-title">{getTabTitle()}</h2>
        <p className="section-subtitle">{getTabSubtitle()}</p>
      </div>
      {renderContent()}
    </div>
  );
}
