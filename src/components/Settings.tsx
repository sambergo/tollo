import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useImageCache } from "../hooks/useImageCache";

interface ChannelList {
  id: number;
  name: string;
  source: string; // url or file path
  is_default: boolean;
  last_fetched: number | null;
}

interface SettingsProps {
  onSelectList: (id: number) => void;
}

// Icon components
const PlayIcon = () => (
  <svg className="settings-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <polygon points="5,3 19,12 5,21"/>
  </svg>
);

const ClockIcon = () => (
  <svg className="settings-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <circle cx="12" cy="12" r="10"/>
    <polyline points="12,6 12,12 16,14"/>
  </svg>
);

const ListIcon = () => (
  <svg className="settings-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <line x1="8" y1="6" x2="21" y2="6"/>
    <line x1="8" y1="12" x2="21" y2="12"/>
    <line x1="8" y1="18" x2="21" y2="18"/>
    <line x1="3" y1="6" x2="3.01" y2="6"/>
    <line x1="3" y1="12" x2="3.01" y2="12"/>
    <line x1="3" y1="18" x2="3.01" y2="18"/>
  </svg>
);

const ImageIcon = () => (
  <svg className="settings-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <rect x="3" y="3" width="18" height="18" rx="2" ry="2"/>
    <circle cx="8.5" cy="8.5" r="1.5"/>
    <polyline points="21,15 16,10 5,21"/>
  </svg>
);

const EditIcon = () => (
  <svg className="action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path d="11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
    <path d="18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
  </svg>
);

const TrashIcon = () => (
  <svg className="action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <polyline points="3,6 5,6 21,6"/>
    <path d="19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2V6"/>
    <line x1="10" y1="11" x2="10" y2="17"/>
    <line x1="14" y1="11" x2="14" y2="17"/>
  </svg>
);

const RefreshIcon = () => (
  <svg className="action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <path d="23 4v6h-6"/>
    <path d="20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
  </svg>
);

const CheckIcon = () => (
  <svg className="action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <polyline points="20,6 9,17 4,12"/>
  </svg>
);

const XIcon = () => (
  <svg className="action-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
    <line x1="18" y1="6" x2="6" y2="18"/>
    <line x1="6" y1="6" x2="18" y2="18"/>
  </svg>
);

const StarIcon = ({ filled }: { filled: boolean }) => (
  <svg className="action-icon" fill={filled ? "currentColor" : "none"} stroke="currentColor" viewBox="0 0 24 24">
    <polygon points="12,2 15.09,8.26 22,9.27 17,14.14 18.18,21.02 12,17.77 5.82,21.02 7,14.14 2,9.27 8.91,8.26"/>
  </svg>
);

function Settings({ onSelectList }: SettingsProps) {
  const [playerCommand, setPlayerCommand] = useState("");
  const [channelLists, setChannelLists] = useState<ChannelList[]>([]);
  const [newListName, setNewListName] = useState("");
  const [newListSource, setNewListSource] = useState("");
  const [defaultChannelList, setDefaultChannelList] = useState<number | null>(null);
  const [cacheDuration, setCacheDuration] = useState(24);
  const [editingList, setEditingList] = useState<ChannelList | null>(null);
  const [imageCacheSize, setImageCacheSize] = useState<number>(0);
  const { clearCache, getCacheSize } = useImageCache();

  async function fetchPlayerCommand() {
    const fetchedCommand = await invoke<string>("get_player_command");
    setPlayerCommand(fetchedCommand);
  }

  async function fetchChannelLists() {
    const fetchedLists = await invoke<ChannelList[]>("get_channel_lists");
    setChannelLists(fetchedLists);
    const defaultList = fetchedLists.find((list) => list.is_default);
    if (defaultList) {
      setDefaultChannelList(defaultList.id);
    }
  }

  async function fetchCacheDuration() {
    const duration = await invoke<number>("get_cache_duration");
    setCacheDuration(duration);
  }

  async function fetchImageCacheSize() {
    const size = await getCacheSize();
    setImageCacheSize(size);
  }

  useEffect(() => {
    fetchPlayerCommand();
    fetchChannelLists();
    fetchCacheDuration();
    fetchImageCacheSize();
  }, []);

  const handleSaveSettings = async () => {
    await invoke("set_player_command", { command: playerCommand });
    if (defaultChannelList !== null) {
      await invoke("set_default_channel_list", { id: defaultChannelList });
    }
    await invoke("set_cache_duration", { hours: cacheDuration });
  };

  const handleAddChannelList = async () => {
    if (newListName && newListSource) {
      await invoke("add_channel_list", { name: newListName, source: newListSource });
      setNewListName("");
      setNewListSource("");
      fetchChannelLists();
    }
  };

  const handleSetDefault = (id: number) => {
    setDefaultChannelList(id);
  };

  const handleRefreshChannelList = async (id: number) => {
    await invoke("refresh_channel_list", { id });
    fetchChannelLists();
  };

  const handleDeleteChannelList = async (id: number) => {
    await invoke("delete_channel_list", { id });
    fetchChannelLists();
  };

  const handleUpdateChannelList = async () => {
    if (editingList) {
      await invoke("update_channel_list", {
        id: editingList.id,
        name: editingList.name,
        source: editingList.source,
      });
      setEditingList(null);
      fetchChannelLists();
    }
  };

  const handleEditClick = (list: ChannelList) => {
    setEditingList({ ...list });
  };

  const handleClearImageCache = async () => {
    try {
      await clearCache();
      await fetchImageCacheSize(); // Refresh cache size
      alert("Image cache cleared successfully!");
    } catch (error) {
      alert("Failed to clear image cache: " + error);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div className="settings-layout">
      {/* Channel Lists Card */}
      <div className="settings-card">
        <div className="card-header">
          <ListIcon />
          <h3>Channel Lists</h3>
        </div>
        <div className="card-content">
          {/* Add New List Form */}
          <div className="add-list-form">
            <div className="form-row">
              <input
                type="text"
                className="form-input"
                placeholder="List Name"
                value={newListName}
                onChange={(e) => setNewListName(e.target.value)}
              />
              <input
                type="text"
                className="form-input"
                placeholder="URL or File Path"
                value={newListSource}
                onChange={(e) => setNewListSource(e.target.value)}
              />
              <button 
                className="btn-primary"
                onClick={handleAddChannelList}
                disabled={!newListName || !newListSource}
              >
                Add List
              </button>
            </div>
          </div>

          {/* Channel Lists */}
          <div className="channel-lists">
            {channelLists.map((list) => (
              <div key={list.id} className="channel-list-item">
                {editingList && editingList.id === list.id ? (
                  /* Edit Mode */
                  <div className="edit-form">
                    <div className="form-row">
                      <input
                        type="text"
                        className="form-input"
                        value={editingList.name}
                        onChange={(e) =>
                          setEditingList({ ...editingList, name: e.target.value })
                        }
                      />
                      <input
                        type="text"
                        className="form-input"
                        value={editingList.source}
                        onChange={(e) =>
                          setEditingList({ ...editingList, source: e.target.value })
                        }
                      />
                      <div className="edit-actions">
                        <button className="btn-success" onClick={handleUpdateChannelList}>
                          <CheckIcon />
                        </button>
                        <button className="btn-secondary" onClick={() => setEditingList(null)}>
                          <XIcon />
                        </button>
                      </div>
                    </div>
                  </div>
                ) : (
                  /* View Mode */
                  <div className="list-info">
                    <div className="list-details">
                      <div className="list-header">
                        <h4 className="list-name">{list.name}</h4>
                        {defaultChannelList === list.id && (
                          <span className="default-badge">Default</span>
                        )}
                      </div>
                      <p className="list-source">{list.source}</p>
                      {list.last_fetched && (
                        <p className="list-meta">
                          Last updated: {new Date(list.last_fetched * 1000).toLocaleString()}
                        </p>
                      )}
                    </div>
                    <div className="list-actions">
                      <button 
                        className="btn-primary btn-sm"
                        onClick={() => onSelectList(list.id)}
                      >
                        Select
                      </button>
                      <button 
                        className={`btn-icon ${defaultChannelList === list.id ? 'active' : ''}`}
                        onClick={() => handleSetDefault(list.id)}
                        title={defaultChannelList === list.id ? "Default" : "Set as Default"}
                      >
                        <StarIcon filled={defaultChannelList === list.id} />
                      </button>
                      <button 
                        className="btn-icon"
                        onClick={() => handleRefreshChannelList(list.id)}
                        title="Refresh"
                      >
                        <RefreshIcon />
                      </button>
                      <button 
                        className="btn-icon"
                        onClick={() => handleEditClick(list)}
                        title="Edit"
                      >
                        <EditIcon />
                      </button>
                      <button 
                        className="btn-icon btn-danger"
                        onClick={() => handleDeleteChannelList(list.id)}
                        title="Delete"
                      >
                        <TrashIcon />
                      </button>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Player Settings Card */}
      <div className="settings-card">
        <div className="card-header">
          <PlayIcon />
          <h3>Player Settings</h3>
        </div>
        <div className="card-content">
          <div className="form-group">
            <label className="form-label">Player Command</label>
            <input
              type="text"
              className="form-input"
              value={playerCommand}
              onChange={(e) => setPlayerCommand(e.target.value)}
              placeholder="e.g., mpv"
            />
            <p className="form-help">Command to launch external video player</p>
          </div>
        </div>
      </div>

      {/* Cache Settings Card */}
      <div className="settings-card">
        <div className="card-header">
          <ClockIcon />
          <h3>Cache Settings</h3>
        </div>
        <div className="card-content">
          <div className="form-group">
            <label className="form-label">Cache Duration (hours)</label>
            <input
              type="number"
              className="form-input"
              value={cacheDuration}
              onChange={(e) => setCacheDuration(parseInt(e.target.value))}
              min="1"
              max="168"
            />
            <p className="form-help">How long to cache channel data before refreshing</p>
          </div>
        </div>
      </div>

      {/* Image Cache Card */}
      <div className="settings-card">
        <div className="card-header">
          <ImageIcon />
          <h3>Image Cache</h3>
        </div>
        <div className="card-content">
          <div className="cache-info">
            <div className="cache-stat">
              <span className="stat-label">Cache Size:</span>
              <span className="stat-value">{formatBytes(imageCacheSize)}</span>
            </div>
          </div>
          <div className="cache-actions">
            <button className="btn-secondary" onClick={fetchImageCacheSize}>
              Refresh Size
            </button>
            <button className="btn-danger" onClick={handleClearImageCache}>
              Clear Cache
            </button>
          </div>
        </div>
      </div>

      {/* Save Settings */}
      <div className="settings-actions">
        <button className="btn-primary btn-large" onClick={handleSaveSettings}>
          Save All Settings
        </button>
      </div>
    </div>
  );
}

export default Settings;

