import ChannelList, { type Channel } from "./ChannelList";
import GroupList from "./GroupList";
import type { Tab } from "./NavigationSidebar";

interface MainContentProps {
  activeTab: Tab;
  channelListName: string;
  searchQuery: string;
  isSearching: boolean;
  filteredChannels: Channel[];
  favorites: Channel[];
  groups: string[];
  history: Channel[];
  selectedGroup: string | null;
  selectedChannel: Channel | null;
  focusedIndex: number;
  onSearch: (e: React.ChangeEvent<HTMLInputElement>) => void;
  onSelectChannel: (channel: Channel) => void;
  onToggleFavorite: (channel: Channel) => void;
  onSelectGroup: (group: string | null) => void;
}

export default function MainContent({
  activeTab,
  channelListName,
  searchQuery,
  isSearching,
  filteredChannels,
  favorites,
  groups,
  history,
  selectedGroup,
  selectedChannel,
  focusedIndex,
  onSearch,
  onSelectChannel,
  onToggleFavorite,
  onSelectGroup
}: MainContentProps) {
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
        return (
          <>
            {channelListName && (
              <div className="channel-list-info">
                <strong>Channel List:</strong> {channelListName}
              </div>
            )}
            <div className="search-container">
              <input
                type="text"
                className="search-input"
                placeholder="Search channels (min 3 characters)..."
                value={searchQuery}
                onChange={onSearch}
              />
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
                onSelectChannel={onSelectChannel}
                onToggleFavorite={onToggleFavorite}
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
              onSelectChannel={onSelectChannel}
              onToggleFavorite={onToggleFavorite}
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
              onSelectGroup={onSelectGroup}
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
              onSelectChannel={onSelectChannel}
              onToggleFavorite={onToggleFavorite}
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