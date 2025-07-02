import ChannelList from "./ChannelList";
import GroupList from "./GroupList";
import { useChannelListName } from "../hooks/useChannelListName";
import { 
  useChannelStore,
  useUIStore,
  useGroupStore,
  useUserDataStore
} from "../stores";

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

export default function MainContent() {
  // Get state from stores
  const { 
    filteredChannels, 
    selectedChannel, 
    selectedChannelListId,
    setSelectedChannel 
  } = useChannelStore();
  
  const {
    activeTab,
    focusedIndex,
    isLoadingChannelList,
    searchQuery,
    isSearching,
    setSearchQuery
  } = useUIStore();
  
  const {
    groups,
    selectedGroup,
    enabledGroups,
    groupDisplayMode,
    setSelectedGroup,
    setGroupDisplayMode,
    toggleGroupEnabled,
    selectAllGroups,
    unselectAllGroups,
    clearGroupFilter
  } = useGroupStore();
  
  const {
    favorites,
    history,
    toggleFavorite
  } = useUserDataStore();
  
  // Hooks
  const channelListName = useChannelListName(selectedChannelListId);

  const handleSearch = (e: React.ChangeEvent<HTMLInputElement>) => {
    setSearchQuery(e.target.value);
  };

  const handleClearSearch = () => {
    setSearchQuery("");
  };

  const handleToggleGroupEnabled = (group: string) => {
    toggleGroupEnabled(group);
  };

  const handleSelectAllGroups = () => {
    selectAllGroups();
  };

  const handleUnselectAllGroups = () => {
    unselectAllGroups();
  };

  const handleClearGroupFilter = () => {
    clearGroupFilter();
  };

  const getTabTitle = () => {
    switch (activeTab) {
      case "channels":
        return "Channels";
      case "favorites":
        return "Favorites";
      case "groups":
        return "Groups";
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
        if (isLoadingChannelList) {
          return <LoadingChannelList />;
        }
        
        return (
          <>
            {channelListName && (
              <div className="channel-list-info">
                <strong>Channel List:</strong> {channelListName}
              </div>
            )}
            <div className="search-container">
              <div className="search-input-wrapper">
                <input
                  type="text"
                  className="search-input"
                  placeholder="Search channels (min 3 characters)..."
                  value={searchQuery}
                  onChange={handleSearch}
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
            {isSearching && (
              <div className="search-status">
                Searching...
              </div>
            )}
            <div className="content-list">
              <ChannelList
                channels={filteredChannels}
                selectedChannel={selectedChannel}
                focusedIndex={focusedIndex}
                favorites={favorites}
                selectedGroup={selectedGroup}
                onSelectChannel={(channel) => setSelectedChannel(channel)}
                onToggleFavorite={(channel) => toggleFavorite(channel)}
                onClearGroupFilter={handleClearGroupFilter}
              />
            </div>
          </>
        );
      case "favorites":
        return (
          <div className="content-list">
            <ChannelList
              channels={favorites}
              selectedChannel={selectedChannel}
              focusedIndex={focusedIndex}
              favorites={favorites}
              selectedGroup={selectedGroup}
              onSelectChannel={(channel) => setSelectedChannel(channel)}
              onToggleFavorite={(channel) => toggleFavorite(channel)}
              onClearGroupFilter={handleClearGroupFilter}
            />
          </div>
        );
      case "groups":
        return (
          <div className="content-list">
            <GroupList
              groups={groups}
              selectedGroup={selectedGroup}
              focusedIndex={focusedIndex}
              enabledGroups={enabledGroups}
              groupDisplayMode={groupDisplayMode}
              onSelectGroup={(group) => setSelectedGroup(group)}
              onToggleGroupEnabled={(group) => handleToggleGroupEnabled(group)}
              onChangeDisplayMode={(mode) => setGroupDisplayMode(mode)}
              onSelectAllGroups={handleSelectAllGroups}
              onUnselectAllGroups={handleUnselectAllGroups}
            />
          </div>
        );
      case "history":
        return (
          <div className="content-list">
            <ChannelList
              channels={history}
              selectedChannel={selectedChannel}
              focusedIndex={focusedIndex}
              favorites={favorites}
              selectedGroup={selectedGroup}
              onSelectChannel={(channel) => setSelectedChannel(channel)}
              onToggleFavorite={(channel) => toggleFavorite(channel)}
              onClearGroupFilter={handleClearGroupFilter}
            />
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