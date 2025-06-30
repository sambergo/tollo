import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface SavedFilter {
  slot_number: number;
  search_query: string;
  selected_group: string | null;
  name: string;
}

export function useSavedFilters(selectedChannelListId: number | null) {
  const [savedFilters, setSavedFilters] = useState<SavedFilter[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  // Load saved filters when channel list changes
  useEffect(() => {
    const loadSavedFilters = async () => {
      if (selectedChannelListId === null) {
        setSavedFilters([]);
        return;
      }

      setIsLoading(true);
      try {
        const filters = await invoke<SavedFilter[]>("get_saved_filters", { 
          channelListId: selectedChannelListId 
        });
        setSavedFilters(filters);
      } catch (error) {
        console.error("Failed to load saved filters:", error);
        setSavedFilters([]);
      } finally {
        setIsLoading(false);
      }
    };

    loadSavedFilters();
  }, [selectedChannelListId]);

  const saveFilter = async (
    slotNumber: number, 
    searchQuery: string, 
    selectedGroup: string | null,
    name: string
  ): Promise<boolean> => {
    if (selectedChannelListId === null) return false;

    try {
      await invoke("save_filter", {
        channelListId: selectedChannelListId,
        slotNumber,
        searchQuery,
        selectedGroup,
        name
      });

      // Reload saved filters
      const filters = await invoke<SavedFilter[]>("get_saved_filters", { 
        channelListId: selectedChannelListId 
      });
      setSavedFilters(filters);
      return true;
    } catch (error) {
      console.error("Failed to save filter:", error);
      return false;
    }
  };

  const deleteFilter = async (slotNumber: number): Promise<boolean> => {
    if (selectedChannelListId === null) return false;

    try {
      await invoke("delete_saved_filter", {
        channelListId: selectedChannelListId,
        slotNumber
      });

      // Reload saved filters
      const filters = await invoke<SavedFilter[]>("get_saved_filters", { 
        channelListId: selectedChannelListId 
      });
      setSavedFilters(filters);
      return true;
    } catch (error) {
      console.error("Failed to delete filter:", error);
      return false;
    }
  };

  const getFilter = (slotNumber: number): SavedFilter | undefined => {
    return savedFilters.find(filter => filter.slot_number === slotNumber);
  };

  const refreshFilters = async (): Promise<void> => {
    if (selectedChannelListId === null) {
      setSavedFilters([]);
      return;
    }

    setIsLoading(true);
    try {
      const filters = await invoke<SavedFilter[]>("get_saved_filters", { 
        channelListId: selectedChannelListId 
      });
      setSavedFilters(filters);
    } catch (error) {
      console.error("Failed to refresh saved filters:", error);
      setSavedFilters([]);
    } finally {
      setIsLoading(false);
    }
  };

  return {
    savedFilters,
    isLoading,
    saveFilter,
    deleteFilter,
    getFilter,
    refreshFilters
  };
} 