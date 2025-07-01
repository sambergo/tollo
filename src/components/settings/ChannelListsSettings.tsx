import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ChannelList } from "../../types/settings";
import { 
  ListIcon, 
  EditIcon, 
  TrashIcon, 
  RefreshIcon, 
  CheckIcon, 
  XIcon, 
  StarIcon, 
  LoadingIcon 
} from "./SettingsIcons";

interface ChannelListsSettingsProps {
  channelLists: ChannelList[];
  defaultChannelList: number | null;
  loadingLists: Set<number>;
  onSelectList: (id: number) => void;
  onRefreshLists: () => Promise<void>;
}

export function ChannelListsSettings({
  channelLists,
  defaultChannelList,
  loadingLists,
  onSelectList,
  onRefreshLists
}: ChannelListsSettingsProps) {
  const [newListName, setNewListName] = useState("");
  const [newListSource, setNewListSource] = useState("");
  const [editingList, setEditingList] = useState<ChannelList | null>(null);
  const [isAddingList, setIsAddingList] = useState(false);

  const handleAddChannelList = async () => {
    if (newListName && newListSource) {
      setIsAddingList(true);
      try {
        const listId = await invoke<number>("validate_and_add_channel_list", { 
          name: newListName, 
          source: newListSource 
        });
        
        console.log("Successfully added and fetched channel list with ID:", listId);
        
        setNewListName("");
        setNewListSource("");
        await onRefreshLists();
        
      } catch (error) {
        console.error("Failed to add channel list:", error);
        const errorMsg = error instanceof Error ? error.message : String(error);
        alert(`Failed to add channel list "${newListName}".\n\nError: ${errorMsg}`);
      } finally {
        setIsAddingList(false);
      }
    }
  };

  const handleSetDefault = async (id: number) => {
    await invoke("set_default_channel_list", { id });
    await onRefreshLists();
  };

  const handleRefreshChannelList = async (id: number) => {
    try {
      await invoke("refresh_channel_list", { id });
      await onRefreshLists();
    } catch (error) {
      console.error("Failed to refresh channel list:", error);
      alert("Failed to refresh channel list: " + error);
    }
  };

  const handleDeleteChannelList = async (id: number) => {
    await invoke("delete_channel_list", { id });
    await onRefreshLists();
  };

  const handleUpdateChannelList = async () => {
    if (editingList) {
      await invoke("update_channel_list", {
        id: editingList.id,
        name: editingList.name,
        source: editingList.source,
      });
      setEditingList(null);
      await onRefreshLists();
    }
  };

  const handleEditClick = (list: ChannelList) => {
    setEditingList({ ...list });
  };

  return (
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
              disabled={!newListName || !newListSource || isAddingList}
            >
              {isAddingList ? "Adding..." : "Add List"}
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
                      <div className="list-status">
                        {loadingLists.has(list.id) && (
                          <span className="loading-indicator">
                            <LoadingIcon />
                            <span className="loading-text">Fetching...</span>
                          </span>
                        )}
                        {defaultChannelList === list.id && (
                          <span className="default-badge">Default</span>
                        )}
                      </div>
                    </div>
                    <p className="list-source">{list.source}</p>
                    {list.last_fetched && (
                      <p className="list-meta">
                        Last updated: {new Date(list.last_fetched * 1000).toLocaleString()}
                      </p>
                    )}
                    {loadingLists.has(list.id) && (
                      <p className="list-meta loading-status">
                        Downloading channel data...
                      </p>
                    )}
                  </div>
                  <div className="list-actions">
                    <button 
                      className="btn-primary btn-sm"
                      onClick={() => onSelectList(list.id)}
                      disabled={loadingLists.has(list.id)}
                      title={loadingLists.has(list.id) ? "Fetching channels..." : "Select this list"}
                    >
                      {loadingLists.has(list.id) ? "Fetching..." : "Select"}
                    </button>
                    <button 
                      className={`btn-icon ${defaultChannelList === list.id ? 'active' : ''}`}
                      onClick={() => handleSetDefault(list.id)}
                      title={defaultChannelList === list.id ? "Default" : "Set as Default"}
                      disabled={loadingLists.has(list.id)}
                    >
                      <StarIcon filled={defaultChannelList === list.id} />
                    </button>
                    <button 
                      className="btn-icon"
                      onClick={() => handleRefreshChannelList(list.id)}
                      title="Refresh"
                      disabled={loadingLists.has(list.id)}
                    >
                      <RefreshIcon />
                    </button>
                    <button 
                      className="btn-icon"
                      onClick={() => handleEditClick(list)}
                      title="Edit"
                      disabled={loadingLists.has(list.id)}
                    >
                      <EditIcon />
                    </button>
                    <button 
                      className="btn-icon btn-danger"
                      onClick={() => handleDeleteChannelList(list.id)}
                      title="Delete"
                      disabled={loadingLists.has(list.id)}
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
  );
} 