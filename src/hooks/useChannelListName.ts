import { useEffect } from "react";
import { useSettingsStore } from "../stores";

export function useChannelListName(selectedChannelListId: number | null) {
  const { channelListName, getChannelListName } = useSettingsStore();

  useEffect(() => {
    getChannelListName(selectedChannelListId);
  }, [selectedChannelListId, getChannelListName]);

  return channelListName;
}
