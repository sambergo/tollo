import { TvIcon, HeartIcon, UsersIcon, HistoryIcon, SettingsIcon } from "./Icons";
import SavedFilters from "./SavedFilters";
import { useUIStore, useGroupStore, GroupDisplayMode } from "../stores";
import { useSavedFilters, type SavedFilter } from "../hooks/useSavedFilters";
import { useChannelStore } from "../stores";

type Tab = "channels" | "favorites" | "groups" | "history" | "settings";

export default function NavigationSidebar() {
  const { activeTab, setActiveTab, setFocusedIndex, setSearchQuery } = useUIStore();
  const { setSelectedGroup, setGroupDisplayMode } = useGroupStore();
  const { selectedChannelListId } = useChannelStore();
  const { savedFilters } = useSavedFilters(selectedChannelListId);

  const handleTabChange = (tab: Tab) => {
    setActiveTab(tab);
  };

  const handleApplyFilter = (filter: SavedFilter) => {
    // Apply the search query
    setSearchQuery(filter.search_query);
    
    // Apply the group selection and set appropriate display mode
    setSelectedGroup(filter.selected_group);
    
    // If the filter has a selected group, switch to AllGroups mode to make the group filter active
    // If no group is selected, use EnabledGroups mode
    if (filter.selected_group) {
      setGroupDisplayMode(GroupDisplayMode.AllGroups);
    } else {
      setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
    }
    
    // Switch to channels tab to see the results
    setActiveTab("channels");
    setFocusedIndex(0);
  };

  return (
    <div className="nav-sidebar">
      <div className="app-header">
        <div className="app-logo">
          <div className="app-logo-icon">TV</div>
          <h1 className="app-title">IPTV Pro</h1>
        </div>
        <nav className="nav-menu">
          <button 
            className={`nav-button ${activeTab === "channels" ? "active" : ""}`}
            onClick={() => handleTabChange("channels")}
          >
            <TvIcon />
            Channels
          </button>
          <button 
            className={`nav-button ${activeTab === "favorites" ? "active" : ""}`}
            onClick={() => handleTabChange("favorites")}
          >
            <HeartIcon />
            Favorites
          </button>
          <button 
            className={`nav-button ${activeTab === "groups" ? "active" : ""}`}
            onClick={() => handleTabChange("groups")}
          >
            <UsersIcon />
            Groups
          </button>
          <button 
            className={`nav-button ${activeTab === "history" ? "active" : ""}`}
            onClick={() => handleTabChange("history")}
          >
            <HistoryIcon />
            History
          </button>
          <button 
            className={`nav-button ${activeTab === "settings" ? "active" : ""}`}
            onClick={() => handleTabChange("settings")}
          >
            <SettingsIcon />
            Settings
          </button>
        </nav>
      </div>
      <SavedFilters 
        savedFilters={savedFilters}
        onApplyFilter={handleApplyFilter}
      />
    </div>
  );
}

export type { Tab }; 