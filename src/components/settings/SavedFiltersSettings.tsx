import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  useSettingsStore,
  useFilterStore,
  type SavedFilter,
} from "../../stores";
import type { ChannelListWithFilters } from "../../types/settings";
import { FilterIcon, LoadingIcon, TrashIcon } from "./SettingsIcons";

export function SavedFiltersSettings() {
  const [channelListsWithFilters, setChannelListsWithFilters] = useState<
    ChannelListWithFilters[]
  >([]);
  const [loadingSavedFilters, setLoadingSavedFilters] = useState(false);

  const { channelLists } = useSettingsStore();
  const { refreshFilters } = useFilterStore();

  useEffect(() => {
    if (channelLists.length > 0) {
      fetchSavedFilters();
    }
  }, [channelLists]);

  async function fetchSavedFilters() {
    setLoadingSavedFilters(true);
    try {
      const listsWithFilters: ChannelListWithFilters[] = [];

      for (const list of channelLists) {
        try {
          const filters = await invoke<SavedFilter[]>("get_saved_filters", {
            channelListId: list.id,
          });
          listsWithFilters.push({ ...list, savedFilters: filters });
        } catch (error) {
          console.error(
            `Failed to load saved filters for list ${list.id}:`,
            error,
          );
          listsWithFilters.push({ ...list, savedFilters: [] });
        }
      }

      setChannelListsWithFilters(listsWithFilters);
    } catch (error) {
      console.error("Failed to fetch saved filters:", error);
    } finally {
      setLoadingSavedFilters(false);
    }
  }

  const handleDeleteSavedFilter = async (
    channelListId: number,
    slotNumber: number,
  ) => {
    try {
      await invoke("delete_saved_filter", {
        channelListId,
        slotNumber,
      });

      // Refresh saved filters
      await fetchSavedFilters();

      // Refresh the global filter store if it's the current channel list
      await refreshFilters(channelListId);
    } catch (error) {
      console.error("Failed to delete saved filter:", error);
      alert("Failed to delete saved filter: " + error);
    }
  };

  return (
    <div className="settings-card">
      <div className="card-header">
        <FilterIcon />
        <h3>Saved Filters</h3>
      </div>
      <div className="card-content">
        {loadingSavedFilters ? (
          <div className="loading-indicator">
            <LoadingIcon />
            <span className="loading-text">Loading saved filters...</span>
          </div>
        ) : (
          <div className="saved-filters-management">
            {channelListsWithFilters.length === 0 ? (
              <p className="form-help">No channel lists available.</p>
            ) : (
              channelListsWithFilters.map((list) => (
                <div key={list.id} className="channel-list-filters">
                  <h4 className="list-name">{list.name}</h4>
                  {list.savedFilters.length === 0 ? (
                    <p className="form-help">No saved filters for this list.</p>
                  ) : (
                    <div className="filters-list">
                      {list.savedFilters.map((filter) => (
                        <div key={filter.slot_number} className="filter-item">
                          <div className="filter-details">
                            <div className="filter-slot">
                              Slot {filter.slot_number}
                            </div>
                            {filter.selected_group && (
                              <div className="filter-group">
                                Group: {filter.selected_group}
                              </div>
                            )}
                            {filter.search_query && (
                              <div className="filter-query">
                                Search: {filter.search_query}
                              </div>
                            )}
                          </div>
                          <button
                            className="btn-icon btn-danger"
                            onClick={() =>
                              handleDeleteSavedFilter(
                                list.id,
                                filter.slot_number,
                              )
                            }
                            title="Delete this saved filter"
                          >
                            <TrashIcon />
                          </button>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
}
