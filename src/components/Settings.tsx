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
    <div className="settings">
      <div>
        <label>Player Command</label>
        <input
          type="text"
          value={playerCommand}
          onChange={(e) => setPlayerCommand(e.target.value)}
        />
      </div>
      <hr />
      <div>
        <label>Cache Duration (hours)</label>
        <input
          type="number"
          value={cacheDuration}
          onChange={(e) => setCacheDuration(parseInt(e.target.value))}
        />
      </div>
      <hr />
      <div>
        <h3>Channel Lists</h3>
        <ul>
          {channelLists.map((list) => (
            <li key={list.id}>
              {editingList && editingList.id === list.id ? (
                <div>
                  <input
                    type="text"
                    value={editingList.name}
                    onChange={(e) =>
                      setEditingList({ ...editingList, name: e.target.value })
                    }
                  />
                  <input
                    type="text"
                    value={editingList.source}
                    onChange={(e) =>
                      setEditingList({ ...editingList, source: e.target.value })
                    }
                  />
                  <button onClick={handleUpdateChannelList}>Save</button>
                  <button onClick={() => setEditingList(null)}>Cancel</button>
                </div>
              ) : (
                <div>
                  <span>{list.name} ({list.source})</span>
                  {list.last_fetched && (
                    <span>
                      {" "}
                      - Last fetched:{" "}
                      {new Date(list.last_fetched * 1000).toLocaleString()}
                    </span>
                  )}
                  <button onClick={() => onSelectList(list.id)}>Select</button>
                  <button onClick={() => handleSetDefault(list.id)}>
                    {defaultChannelList === list.id
                      ? "Default"
                      : "Set as Default"}
                  </button>
                  <button onClick={() => handleRefreshChannelList(list.id)}>
                    Refresh
                  </button>
                  <button onClick={() => handleEditClick(list)}>Edit</button>
                  <button onClick={() => handleDeleteChannelList(list.id)}>
                    Delete
                  </button>
                </div>
              )}
            </li>
          ))}
        </ul>
        <div>
          <input
            type="text"
            placeholder="List Name"
            value={newListName}
            onChange={(e) => setNewListName(e.target.value)}
          />
          <input
            type="text"
            placeholder="URL or File Path"
            value={newListSource}
            onChange={(e) => setNewListSource(e.target.value)}
          />
          <button onClick={handleAddChannelList}>Add List</button>
        </div>
      </div>
      <hr />
      <div>
        <h3>Image Cache</h3>
        <p>Cache Size: {formatBytes(imageCacheSize)}</p>
        <button onClick={handleClearImageCache}>Clear Image Cache</button>
        <button onClick={fetchImageCacheSize}>Refresh Cache Size</button>
      </div>
      <hr />
      <button onClick={handleSaveSettings}>Save Settings</button>
    </div>
  );
}

export default Settings;

