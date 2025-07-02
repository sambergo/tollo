import { useEffect } from "react";
import type { Tab } from "../components/NavigationSidebar";
import type { Channel } from "../components/ChannelList";
import type { SavedFilter } from "../stores";

interface UseKeyboardNavigationProps {
  activeTab: Tab;
  channels: Channel[];
  favorites: Channel[];
  groups: string[];
  history: Channel[];
  selectedGroup: string | null;
  selectedChannel: Channel | null;
  focusedIndex: number;
  listItems: any[];
  searchQuery: string;
  setFocusedIndex: (value: number | ((prev: number) => number)) => void;
  setSelectedChannel: (channel: Channel) => void;
  setActiveTab: (tab: Tab) => void;
  handleSelectGroup: (group: string | null) => void;
  handleToggleFavorite: (channel: Channel) => void;
  handlePlayInMpv: (channel: Channel) => void;
  // Saved filters functionality
  savedFilters: SavedFilter[];
  onSaveFilter: (
    slotNumber: number,
    searchQuery: string,
    selectedGroup: string | null,
    name: string,
  ) => Promise<boolean>;
  onApplyFilter: (filter: SavedFilter) => void;
}

export function useKeyboardNavigation({
  activeTab,
  channels,
  favorites,
  groups,
  history,
  selectedGroup,
  selectedChannel,
  focusedIndex,
  listItems,
  searchQuery,
  setFocusedIndex,
  setSelectedChannel,
  setActiveTab,
  handleSelectGroup,
  handleToggleFavorite,
  handlePlayInMpv,
  savedFilters,
  onSaveFilter,
  onApplyFilter,
}: UseKeyboardNavigationProps) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement) {
        return;
      }

      // Handle number keys (0-9) for applying saved filters
      if (
        e.key >= "0" &&
        e.key <= "9" &&
        !e.altKey &&
        !e.ctrlKey &&
        !e.shiftKey
      ) {
        const slotNumber = parseInt(e.key);
        const filter = savedFilters.find((f) => f.slot_number === slotNumber);
        if (filter) {
          onApplyFilter(filter);
          return;
        }
      }

      // Handle Alt+number keys (Alt+0-9) for saving current filter
      if (
        e.altKey &&
        e.key >= "0" &&
        e.key <= "9" &&
        !e.ctrlKey &&
        !e.shiftKey
      ) {
        e.preventDefault();
        const slotNumber = parseInt(e.key);

        // Generate a name for the filter
        const groupPart = selectedGroup ? `${selectedGroup}` : "All";
        const searchPart = searchQuery ? `"${searchQuery}"` : "No search";
        const filterName = `${groupPart} + ${searchPart}`;

        onSaveFilter(slotNumber, searchQuery, selectedGroup, filterName);
        return;
      }

      if (e.key === "j" || e.key === "ArrowDown") {
        setFocusedIndex((prev) => Math.min(prev + 1, listItems.length - 1));
      } else if (e.key === "k" || e.key === "ArrowUp") {
        setFocusedIndex((prev) => Math.max(prev - 1, 0));
      } else if (e.key === "Enter") {
        if (
          activeTab === "channels" ||
          activeTab === "favorites" ||
          activeTab === "history"
        ) {
          setSelectedChannel(listItems[focusedIndex] as Channel);
        } else if (activeTab === "groups") {
          handleSelectGroup(listItems[focusedIndex] as string);
        }
      } else if (e.key === "l" || e.key === "ArrowRight") {
        const tabs: Tab[] = [
          "channels",
          "favorites",
          "groups",
          "history",
          "settings",
        ];
        const currentIndex = tabs.indexOf(activeTab);
        const nextIndex = (currentIndex + 1) % tabs.length;
        setActiveTab(tabs[nextIndex]);
        setFocusedIndex(0);
      } else if (e.key === "h" || e.key === "ArrowLeft") {
        const tabs: Tab[] = [
          "channels",
          "favorites",
          "groups",
          "history",
          "settings",
        ];
        const currentIndex = tabs.indexOf(activeTab);
        const prevIndex = (currentIndex - 1 + tabs.length) % tabs.length;
        setActiveTab(tabs[prevIndex]);
        setFocusedIndex(0);
      } else if (e.key === "f") {
        if (activeTab === "channels") {
          handleToggleFavorite(listItems[focusedIndex] as Channel);
        }
      } else if (e.key === "p") {
        if (
          activeTab === "channels" ||
          activeTab === "favorites" ||
          activeTab === "history"
        ) {
          handlePlayInMpv(listItems[focusedIndex] as Channel);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [
    activeTab,
    channels,
    favorites,
    groups,
    history,
    selectedGroup,
    selectedChannel,
    focusedIndex,
    listItems,
    searchQuery,
    savedFilters,
    onSaveFilter,
    onApplyFilter,
  ]);
}
