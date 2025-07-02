import { useEffect } from "react";
import { useFilterStore, type SavedFilter } from "../stores";

export type { SavedFilter };

export function useSavedFilters(selectedChannelListId: number | null) {
  const {
    savedFilters,
    isLoading,
    loadSavedFilters,
    saveFilter: storeSaveFilter,
    deleteFilter: storeDeleteFilter,
    getFilter,
    refreshFilters: storeRefreshFilters,
  } = useFilterStore();

  // Load saved filters when channel list changes
  useEffect(() => {
    loadSavedFilters(selectedChannelListId);
  }, [selectedChannelListId, loadSavedFilters]);

  // Wrapper functions to include selectedChannelListId
  const saveFilter = async (
    slotNumber: number,
    searchQuery: string,
    selectedGroup: string | null,
    name: string,
  ): Promise<boolean> => {
    if (selectedChannelListId === null) return false;
    return storeSaveFilter(
      selectedChannelListId,
      slotNumber,
      searchQuery,
      selectedGroup,
      name,
    );
  };

  const deleteFilter = async (slotNumber: number): Promise<boolean> => {
    if (selectedChannelListId === null) return false;
    return storeDeleteFilter(selectedChannelListId, slotNumber);
  };

  const refreshFilters = async (): Promise<void> => {
    return storeRefreshFilters(selectedChannelListId);
  };

  return {
    savedFilters,
    isLoading,
    saveFilter,
    deleteFilter,
    getFilter,
    refreshFilters,
  };
}
