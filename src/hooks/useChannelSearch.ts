import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Channel } from "../components/ChannelList";

export function useChannelSearch(selectedChannelListId: number | null) {
  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedSearchQuery, setDebouncedSearchQuery] = useState("");
  const [isSearching, setIsSearching] = useState(false);

  // Debounce search query
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedSearchQuery(searchQuery);
    }, 400);

    return () => clearTimeout(timer);
  }, [searchQuery]);

  const searchChannels = async (query: string): Promise<Channel[]> => {
    if (query === "" || query.length < 3) {
      const fetchedChannels = await invoke<Channel[]>("get_channels", { id: selectedChannelListId });
      return fetchedChannels;
    } else {
      setIsSearching(true);
      try {
        const searchedChannels = await invoke<Channel[]>("search_channels", { 
          query, 
          id: selectedChannelListId 
        });
        return searchedChannels;
      } catch (error) {
        console.error("Search failed:", error);
        const fetchedChannels = await invoke<Channel[]>("get_channels", { id: selectedChannelListId });
        return fetchedChannels;
      } finally {
        setIsSearching(false);
      }
    }
  };

  return {
    searchQuery,
    setSearchQuery,
    debouncedSearchQuery,
    isSearching,
    searchChannels
  };
} 