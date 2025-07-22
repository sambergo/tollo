import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useChannelStore, useSettingsStore } from "../../stores";
import {
  asyncPlaylistStore,
  PlaylistFetchStatus,
} from "../../stores/asyncPlaylistStore";
import type { ChannelList } from "../../types/settings";
import {
  ListIcon,
  EditIcon,
  TrashIcon,
  RefreshIcon,
  CheckIcon,
  XIcon,
  StarIcon,
  LoadingIcon,
} from "./SettingsIcons";

interface ChannelListsSettingsProps {
  defaultChannelList: number | null;
  loadingLists: Set<number>;
  onSelectList: (id: number) => void;
  onRefreshLists: () => Promise<void>;
  onSelectingChange?: (isSelecting: boolean, listName?: string) => void;
}

export function ChannelListsSettings({
  defaultChannelList,
  loadingLists,
  onSelectList,
  onRefreshLists,
  onSelectingChange,
}: ChannelListsSettingsProps) {
  const [newListName, setNewListName] = useState("");
  const [newListSource, setNewListSource] = useState("");
  const [editingList, setEditingList] = useState<ChannelList | null>(null);
  const [isAddingList, setIsAddingList] = useState(false);
  const [selectingList, setSelectingList] = useState<number | null>(null);
  const [refreshingList, setRefreshingList] = useState<number | null>(null);

  // Async operation tracking
  const [asyncStatuses, setAsyncStatuses] = useState<
    Map<number, PlaylistFetchStatus>
  >(new Map());

  // Get data from stores
  const { channelLists } = useSettingsStore();
  const { selectedChannelListId } = useChannelStore();

  // Subscribe to async playlist status updates
  useEffect(() => {
    const unsubscribe = asyncPlaylistStore.onStatusUpdate((status) => {
      setAsyncStatuses((prev) => {
        const newMap = new Map(prev);
        newMap.set(status.id, status);

        // Auto-refresh lists when operations complete
        if (status.status === "completed") {
          onRefreshLists();
          // Clean up after a delay
          setTimeout(() => {
            setAsyncStatuses((curr) => {
              const updated = new Map(curr);
              updated.delete(status.id);
              return updated;
            });
          }, 3000);
        }

        return newMap;
      });
    });

    return unsubscribe;
  }, [onRefreshLists]);

  const handleAddChannelList = async () => {
    if (newListName && newListSource) {
      setIsAddingList(true);
      try {
        const listId = await asyncPlaylistStore.addPlaylistAsync(
          newListName,
          newListSource,
        );
        console.log("Started async playlist addition with ID:", listId);
        setNewListName("");
        setNewListSource("");
      } catch (error) {
        console.error("Failed to add channel list:", error);
        const errorMsg = error instanceof Error ? error.message : String(error);
        alert(
          `Failed to add channel list "${newListName}".\n\nError: ${errorMsg}`,
        );
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
    setRefreshingList(id);
    try {
      await asyncPlaylistStore.refreshPlaylistAsync(id);
      console.log(`Started async playlist refresh for ID: ${id}`);
    } catch (error) {
      console.error("Failed to refresh channel list:", error);
      alert("Failed to refresh channel list: " + error);
    } finally {
      setRefreshingList(null);
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

  const handleSelectList = (id: number) => {
    const selectedList = channelLists.find((list) => list.id === id);
    const listName = selectedList?.name || "Unknown List";

    setSelectingList(id);
    onSelectingChange?.(true, listName);

    // Use setTimeout to ensure the UI updates before starting the operation
    setTimeout(async () => {
      try {
        // First call the async backend command to properly download the playlist
        await invoke("start_channel_list_selection_async", { id });

        // Only after download completes, set the selected list ID and navigate to channels
        onSelectList(id);
      } catch (error) {
        console.error("Failed to select channel list:", error);
        alert(`Failed to select channel list: ${error}`);
      } finally {
        setSelectingList(null);
        onSelectingChange?.(false);
      }
    }, 50); // Small delay to ensure UI renders
  };

  const getStatusColor = (status: string): string => {
    switch (status) {
      case "completed":
        return "#4CAF50";
      case "error":
        return "#F44336";
      case "fetching":
        return "#2196F3";
      case "processing":
        return "#FF9800";
      case "saving":
        return "#9C27B0";
      default:
        return "#757575";
    }
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
          {channelLists.map((list) => {
            const asyncStatus = asyncStatuses.get(list.id);
            const isAsyncOperation =
              asyncStatus &&
              (asyncStatus.status === "starting" ||
                asyncStatus.status === "fetching" ||
                asyncStatus.status === "processing" ||
                asyncStatus.status === "saving");

            return (
              <div
                key={list.id}
                className={`channel-list-item ${refreshingList === list.id ? "refreshing" : ""} ${isAsyncOperation ? "async-operation" : ""}`}
              >
                {/* Progress overlay for async operations */}
                {isAsyncOperation && (
                  <div className="async-progress-overlay">
                    <div className="async-progress-content">
                      <div className="async-progress-info">
                        <span className="async-progress-message">
                          {asyncStatus.message}
                        </span>
                        <span className="async-progress-percentage">
                          {Math.round(asyncStatus.progress * 100)}%
                        </span>
                      </div>
                      <div className="async-progress-bar">
                        <div
                          className="async-progress-fill"
                          style={{
                            width: `${asyncStatus.progress * 100}%`,
                            backgroundColor: getStatusColor(asyncStatus.status),
                          }}
                        />
                      </div>
                      <div
                        className="async-status-badge"
                        style={{
                          backgroundColor: getStatusColor(asyncStatus.status),
                        }}
                      >
                        {asyncStatus.status.toUpperCase()}
                      </div>
                    </div>
                  </div>
                )}

                {/* Legacy loading overlay */}
                {refreshingList === list.id && !isAsyncOperation && (
                  <div className="channel-list-loading-overlay">
                    <div className="loading-content">
                      <div className="loading-spinner"></div>
                      <span>Refreshing channel list...</span>
                    </div>
                  </div>
                )}

                {editingList && editingList.id === list.id ? (
                  /* Edit Mode */
                  <div className="edit-form">
                    <div className="form-row">
                      <input
                        type="text"
                        className="form-input"
                        value={editingList.name}
                        onChange={(e) =>
                          setEditingList({
                            ...editingList,
                            name: e.target.value,
                          })
                        }
                      />
                      <input
                        type="text"
                        className="form-input"
                        value={editingList.source}
                        onChange={(e) =>
                          setEditingList({
                            ...editingList,
                            source: e.target.value,
                          })
                        }
                      />
                      <div className="edit-actions">
                        <button
                          className="btn-success"
                          onClick={handleUpdateChannelList}
                        >
                          <CheckIcon />
                        </button>
                        <button
                          className="btn-secondary"
                          onClick={() => setEditingList(null)}
                        >
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
                          {loadingLists.has(list.id) && !isAsyncOperation && (
                            <span className="loading-indicator">
                              <LoadingIcon />
                              <span className="loading-text">Fetching...</span>
                            </span>
                          )}
                          {asyncStatus &&
                            asyncStatus.status === "completed" && (
                              <span className="async-success-badge">
                                ✓{" "}
                                {asyncStatus.channel_count
                                  ? `${asyncStatus.channel_count} channels`
                                  : "Completed"}
                              </span>
                            )}
                          {asyncStatus && asyncStatus.status === "error" && (
                            <span
                              className="async-error-badge"
                              title={asyncStatus.error}
                            >
                              ✗ Failed
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
                          Last updated:{" "}
                          {new Date(list.last_fetched * 1000).toLocaleString()}
                        </p>
                      )}
                      {loadingLists.has(list.id) && !isAsyncOperation && (
                        <p className="list-meta loading-status">
                          Downloading channel data...
                        </p>
                      )}
                    </div>
                    <div className="list-actions">
                      <button
                        className="btn-primary btn-sm"
                        onClick={() => handleSelectList(list.id)}
                        disabled={
                          loadingLists.has(list.id) ||
                          selectedChannelListId === list.id ||
                          selectingList === list.id ||
                          isAsyncOperation
                        }
                      >
                        {selectingList === list.id ? "Selecting..." : "Select"}
                      </button>

                      <button
                        className="btn-icon btn-secondary"
                        onClick={() => handleRefreshChannelList(list.id)}
                        disabled={
                          loadingLists.has(list.id) ||
                          refreshingList === list.id ||
                          isAsyncOperation
                        }
                        title={
                          isAsyncOperation
                            ? "Operation in progress..."
                            : refreshingList === list.id
                              ? "Refreshing..."
                              : "Refresh channel list data"
                        }
                      >
                        <RefreshIcon />
                      </button>

                      <button
                        className="btn-icon btn-secondary"
                        onClick={() => handleEditClick(list)}
                        disabled={loadingLists.has(list.id) || isAsyncOperation}
                        title="Edit channel list"
                      >
                        <EditIcon />
                      </button>

                      <button
                        className="btn-icon btn-secondary"
                        onClick={() => handleSetDefault(list.id)}
                        disabled={
                          loadingLists.has(list.id) ||
                          defaultChannelList === list.id ||
                          isAsyncOperation
                        }
                        title="Set as default channel list"
                      >
                        <StarIcon filled={defaultChannelList === list.id} />
                      </button>

                      <button
                        className="btn-icon btn-danger"
                        onClick={() => handleDeleteChannelList(list.id)}
                        disabled={loadingLists.has(list.id) || isAsyncOperation}
                        title="Delete channel list"
                      >
                        <TrashIcon />
                      </button>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>

        {channelLists.length === 0 && (
          <p className="form-help">
            No channel lists found. Add one above to get started.
          </p>
        )}
      </div>
    </div>
  );
}
