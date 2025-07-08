import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  useChannelStore,
  useSettingsStore,
  useUIStore,
  useSearchStore,
} from "../stores";
import type { ChannelList } from "../types/settings";
import { ChannelListsSettings } from "./settings/ChannelListsSettings";
import { PlayerSettings } from "./settings/PlayerSettings";

import { ImageCacheSettings } from "./settings/ImageCacheSettings";
import { SavedFiltersSettings } from "./settings/SavedFiltersSettings";

function Settings() {
  const [defaultChannelList, setDefaultChannelList] = useState<number | null>(
    null,
  );
  const [loadingLists, setLoadingLists] = useState<Set<number>>(new Set());
  const [isSelectingChannelList, setIsSelectingChannelList] = useState(false);
  const [selectingListName, setSelectingListName] = useState<string>("");

  // Get state and actions from stores
  const {
    selectedChannelListId,
    setSelectedChannelListId,
    setIsLoadingChannelList,
    setChannels,
    setSelectedChannel,
  } = useChannelStore();
  const { setChannelLists } = useSettingsStore();
  const { setActiveTab, clearGroupFilter } = useUIStore();
  const { clearSearch } = useSearchStore();

  // Clear all UI state to provide a clean starting point
  const resetToDefaultState = () => {
    setSelectedChannel(null);
    clearSearch();
    clearGroupFilter();
  };

  async function fetchChannelListsData() {
    const fetchedLists = await invoke<ChannelList[]>("get_channel_lists");
    setChannelLists(fetchedLists);
    const defaultList = fetchedLists.find((list) => list.is_default);
    if (defaultList) {
      setDefaultChannelList(defaultList.id);
    }
  }

  useEffect(() => {
    fetchChannelListsData();
  }, []);

  const handleRefreshLists = async () => {
    await fetchChannelListsData();
  };

  const handleSelectingChange = (isSelecting: boolean, listName?: string) => {
    setIsSelectingChannelList(isSelecting);
    setSelectingListName(listName || "");
  };

  const handleSelectList = async (id: number) => {
    // Don't proceed if this list is already selected
    if (id === selectedChannelListId) {
      return;
    }

    try {
      setLoadingLists((prev) => new Set([...prev, id]));

      // Set loading state immediately when user clicks Select
      setIsLoadingChannelList(true);

      // Clear current data to show loading state immediately
      setChannels([]);

      // Clear selected channel to get a clean starting point
      resetToDefaultState();

      // Update selected channel list
      setSelectedChannelListId(id);

      // Navigate back to channels tab to see the loaded data
      setActiveTab("channels");
    } finally {
      setLoadingLists((prev) => {
        const newSet = new Set(prev);
        newSet.delete(id);
        return newSet;
      });
    }
  };

  return (
    <div
      className={`settings-layout ${isSelectingChannelList ? "selecting" : ""}`}
    >
      {isSelectingChannelList && (
        <div className="settings-loading-overlay">
          <div className="loading-content">
            <div className="loading-spinner-large"></div>
            <h3>Switching to "{selectingListName}"</h3>
            <p>Preparing channel list and switching to channels tab...</p>
          </div>
        </div>
      )}

      <ChannelListsSettings
        defaultChannelList={defaultChannelList}
        loadingLists={loadingLists}
        onSelectList={handleSelectList}
        onRefreshLists={handleRefreshLists}
        onSelectingChange={handleSelectingChange}
      />

      <PlayerSettings />

      <ImageCacheSettings />

      <SavedFiltersSettings />
    </div>
  );
}

export default Settings;
