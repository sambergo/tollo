import { TvIcon, HeartIcon, UsersIcon, HistoryIcon, SettingsIcon } from "./Icons";

type Tab = "channels" | "favorites" | "groups" | "history" | "settings";

interface NavigationSidebarProps {
  activeTab: Tab;
  onTabChange: (tab: Tab) => void;
}

export default function NavigationSidebar({ activeTab, onTabChange }: NavigationSidebarProps) {
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
            onClick={() => onTabChange("channels")}
          >
            <TvIcon />
            Channels
          </button>
          <button 
            className={`nav-button ${activeTab === "favorites" ? "active" : ""}`}
            onClick={() => onTabChange("favorites")}
          >
            <HeartIcon />
            Favorites
          </button>
          <button 
            className={`nav-button ${activeTab === "groups" ? "active" : ""}`}
            onClick={() => onTabChange("groups")}
          >
            <UsersIcon />
            Groups
          </button>
          <button 
            className={`nav-button ${activeTab === "history" ? "active" : ""}`}
            onClick={() => onTabChange("history")}
          >
            <HistoryIcon />
            History
          </button>
          <button 
            className={`nav-button ${activeTab === "settings" ? "active" : ""}`}
            onClick={() => onTabChange("settings")}
          >
            <SettingsIcon />
            Settings
          </button>
        </nav>
      </div>
    </div>
  );
}

export type { Tab }; 