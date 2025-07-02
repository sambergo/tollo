import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useChannelStore, useSettingsStore, useUIStore } from "../stores";
import type { ChannelList } from "../types/settings";
import { ChannelListsSettings } from "./settings/ChannelListsSettings";
import { PlayerSettings } from "./settings/PlayerSettings";
import { CacheSettings } from "./settings/CacheSettings";
import { ImageCacheSettings } from "./settings/ImageCacheSettings";
import { SavedFiltersSettings } from "./settings/SavedFiltersSettings";

function Settings() {
  const [defaultChannelList, setDefaultChannelList] = useState<number | null>(null);
  const [loadingLists, setLoadingLists] = useState<Set<number>>(new Set());
  
  // Get state and actions from stores
  const { selectedChannelListId, setSelectedChannelListId, setIsLoadingChannelList, setChannels } = useChannelStore();
  const { setChannelLists } = useSettingsStore();
  const { setActiveTab } = useUIStore();

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

  const handleSelectList = async (id: number) => {
    // Don't proceed if this list is already selected
    if (id === selectedChannelListId) {
      return;
    }
    
    try {
      setLoadingLists(prev => new Set([...prev, id]));
      
      // Set loading state immediately when user clicks Select
      setIsLoadingChannelList(true);
      
      // Clear current data to show loading state immediately
      setChannels([]);
      
      // Update selected channel list
      setSelectedChannelListId(id);
      
      // Navigate back to channels tab to see the loaded data
      setActiveTab("channels");
      
    } finally {
      setLoadingLists(prev => {
        const newSet = new Set(prev);
        newSet.delete(id);
        return newSet;
      });
    }
  };



  return (
    <div className="settings-layout">
      <ChannelListsSettings
        defaultChannelList={defaultChannelList}
        loadingLists={loadingLists}
        onSelectList={handleSelectList}
        onRefreshLists={handleRefreshLists}
      />

      <PlayerSettings />

      <CacheSettings />

      <ImageCacheSettings />

      <SavedFiltersSettings />
    </div>
  );
}

export default Settings;

