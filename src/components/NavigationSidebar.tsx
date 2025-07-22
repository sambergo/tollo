import {
  TvIcon,
  HeartIcon,
  UsersIcon,
  HistoryIcon,
  HelpIcon,
  SettingsIcon,
} from "./Icons";
import SavedFilters from "./SavedFilters";
import { useUIStore, useChannelStore } from "../stores";

type Tab =
  | "channels"
  | "favorites"
  | "groups"
  | "history"
  | "help"
  | "settings";

export default function NavigationSidebar() {
  const { activeTab, setActiveTab } = useUIStore();
  const { selectedChannelListId } = useChannelStore();

  return (
    <div className="nav-sidebar">
      <div className="app-header">
        <div className="app-logo">
          <img className="app-logo-icon" src="/logo.png" alt="Tollo logo" />
        </div>
        <nav className="nav-menu">
          <button
            className={`nav-button ${activeTab === "channels" ? "active" : ""}`}
            onClick={() => setActiveTab("channels")}
          >
            <TvIcon />
            Channels
          </button>
          <button
            className={`nav-button ${activeTab === "favorites" ? "active" : ""}`}
            onClick={() => setActiveTab("favorites")}
          >
            <HeartIcon />
            Favorites
          </button>
          <button
            className={`nav-button ${activeTab === "groups" ? "active" : ""}`}
            onClick={() => setActiveTab("groups")}
          >
            <UsersIcon />
            Groups
          </button>
          <button
            className={`nav-button ${activeTab === "history" ? "active" : ""}`}
            onClick={() => setActiveTab("history")}
          >
            <HistoryIcon />
            History
          </button>
          <button
            className={`nav-button ${activeTab === "help" ? "active" : ""}`}
            onClick={() => setActiveTab("help")}
          >
            <HelpIcon />
            Help
          </button>
          <button
            className={`nav-button ${activeTab === "settings" ? "active" : ""}`}
            onClick={() => setActiveTab("settings")}
          >
            <SettingsIcon />
            Settings
          </button>
        </nav>
      </div>
      {selectedChannelListId && <SavedFilters />}
    </div>
  );
}

export type { Tab };
