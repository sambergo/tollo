import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useChannelListName(selectedChannelListId: number | null) {
  const [channelListName, setChannelListName] = useState<string>("");

  useEffect(() => {
    async function fetchChannelLists() {
      const lists = await invoke<any[]>("get_channel_lists");
      const found = lists.find((l) => l.id === selectedChannelListId);
      setChannelListName(found ? found.name : "");
    }
    if (selectedChannelListId !== null) {
      fetchChannelLists();
    } else {
      setChannelListName("");
    }
  }, [selectedChannelListId]);

  return channelListName;
} 