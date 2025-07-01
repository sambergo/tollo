import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ChannelList, SettingsProps } from "../types/settings";
import { ChannelListsSettings } from "./settings/ChannelListsSettings";
import { PlayerSettings } from "./settings/PlayerSettings";
import { CacheSettings } from "./settings/CacheSettings";
import { ImageCacheSettings } from "./settings/ImageCacheSettings";
import { SavedFiltersSettings } from "./settings/SavedFiltersSettings";

function Settings({ onSelectList, onFiltersChanged }: SettingsProps) {
  const [channelLists, setChannelLists] = useState<ChannelList[]>([]);
  const [defaultChannelList, setDefaultChannelList] = useState<number | null>(null);
  const [loadingLists, setLoadingLists] = useState<Set<number>>(new Set());

  async function fetchChannelLists() {
    const fetchedLists = await invoke<ChannelList[]>("get_channel_lists");
    setChannelLists(fetchedLists);
    const defaultList = fetchedLists.find((list) => list.is_default);
    if (defaultList) {
      setDefaultChannelList(defaultList.id);
    }
  }

  useEffect(() => {
    fetchChannelLists();
  }, []);

  const handleRefreshLists = async () => {
    await fetchChannelLists();
  };

  const handleSelectList = async (id: number) => {
    try {
      setLoadingLists(prev => new Set([...prev, id]));
      onSelectList(id);
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
        channelLists={channelLists}
        defaultChannelList={defaultChannelList}
        loadingLists={loadingLists}
        onSelectList={handleSelectList}
        onRefreshLists={handleRefreshLists}
      />

      <PlayerSettings />

      <CacheSettings />

      <ImageCacheSettings />

      <SavedFiltersSettings
        channelLists={channelLists}
        onFiltersChanged={onFiltersChanged}
      />
    </div>
  );
}

export default Settings;

