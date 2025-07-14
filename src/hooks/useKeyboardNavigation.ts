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
  // Search and filter actions
  clearSearch: () => void;
  clearAllFilters: () => void;
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
  clearSearch,
  clearAllFilters,
}: UseKeyboardNavigationProps) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Handle escape key even when input fields are focused
      if (e.key === "Escape") {
        const focusedElement = document.activeElement;
        if (focusedElement && focusedElement.tagName === 'INPUT') {
          // If search input is focused, just unfocus it without clearing
          (focusedElement as HTMLInputElement).blur();
        } else {
          // If search input is not focused, clear the search
          clearSearch();
        }
        return;
      }

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
      
      // Handle Tab key separately to prevent conflicts
      if (e.key === "Tab" && !e.shiftKey && !e.ctrlKey && !e.altKey) {
        e.preventDefault();
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
        return;
      }

      // Navigation within lists
      if (e.key === "j" || e.key === "ArrowDown") {
        setFocusedIndex((prev) => {
          const newIndex = Math.min(prev + 1, listItems.length - 1);
          // Auto-select channel when navigating in channel-related tabs
          if (
            (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") &&
            listItems[newIndex]
          ) {
            setSelectedChannel(listItems[newIndex] as Channel);
          }
          return newIndex;
        });
      } else if (e.key === "k" || e.key === "ArrowUp") {
        setFocusedIndex((prev) => {
          const newIndex = Math.max(prev - 1, 0);
          // Auto-select channel when navigating in channel-related tabs
          if (
            (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") &&
            listItems[newIndex]
          ) {
            setSelectedChannel(listItems[newIndex] as Channel);
          }
          return newIndex;
        });
      }
      
      // Tab navigation
      else if (e.key === "J" || e.key === "Tab") {
        e.preventDefault(); // Prevent default tab behavior
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
      } else if (e.key === "K") {
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
      }
      
      // Selection and interaction
      else if (e.key === "l" || e.key === "ArrowRight") {
        if (
          activeTab === "channels" ||
          activeTab === "favorites" ||
          activeTab === "history"
        ) {
          setSelectedChannel(listItems[focusedIndex] as Channel);
        } else if (activeTab === "groups") {
          handleSelectGroup(listItems[focusedIndex] as string);
        }
      } else if (e.key === "Enter" || e.key === "o") {
        if (
          activeTab === "channels" ||
          activeTab === "favorites" ||
          activeTab === "history"
        ) {
          handlePlayInMpv(listItems[focusedIndex] as Channel);
        } else if (activeTab === "groups") {
          handleSelectGroup(listItems[focusedIndex] as string);
        }
      }
      
      // Enhanced Navigation
      else if (e.key === "g" && !e.shiftKey && !e.ctrlKey && !e.altKey) {
        // Go to first item in current view
        const firstVisibleIndex = Math.floor(focusedIndex / 200) * 200;
        setFocusedIndex(firstVisibleIndex);
        if (
          (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") &&
          listItems[firstVisibleIndex]
        ) {
          setSelectedChannel(listItems[firstVisibleIndex] as Channel);
        }
      } else if (e.key === "G" && !e.ctrlKey && !e.altKey) {
        // Go to last item in current view
        const currentPage = Math.floor(focusedIndex / 200);
        const lastVisibleIndex = Math.min((currentPage + 1) * 200 - 1, listItems.length - 1);
        setFocusedIndex(lastVisibleIndex);
        if (
          (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") &&
          listItems[lastVisibleIndex]
        ) {
          setSelectedChannel(listItems[lastVisibleIndex] as Channel);
        }
      } else if (e.key === "Home") {
        // Go to first item in current view
        const firstVisibleIndex = Math.floor(focusedIndex / 200) * 200;
        setFocusedIndex(firstVisibleIndex);
        if (
          (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") &&
          listItems[firstVisibleIndex]
        ) {
          setSelectedChannel(listItems[firstVisibleIndex] as Channel);
        }
      } else if (e.key === "End") {
        // Go to last item in current view
        const currentPage = Math.floor(focusedIndex / 200);
        const lastVisibleIndex = Math.min((currentPage + 1) * 200 - 1, listItems.length - 1);
        setFocusedIndex(lastVisibleIndex);
        if (
          (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") &&
          listItems[lastVisibleIndex]
        ) {
          setSelectedChannel(listItems[lastVisibleIndex] as Channel);
        }
      } else if (e.ctrlKey && e.key === "u") {
        // Page up - scroll up by 10 items
        e.preventDefault();
        setFocusedIndex((prev) => {
          const newIndex = Math.max(prev - 10, 0);
          if (
            (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") &&
            listItems[newIndex]
          ) {
            setSelectedChannel(listItems[newIndex] as Channel);
          }
          return newIndex;
        });
      } else if (e.ctrlKey && e.key === "d") {
        // Page down - scroll down by 10 items
        e.preventDefault();
        setFocusedIndex((prev) => {
          const newIndex = Math.min(prev + 10, listItems.length - 1);
          if (
            (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") &&
            listItems[newIndex]
          ) {
            setSelectedChannel(listItems[newIndex] as Channel);
          }
          return newIndex;
        });
      } else if (e.key === "PageUp") {
        // Page up - scroll up by 10 items (same as Ctrl+u)
        e.preventDefault();
        setFocusedIndex((prev) => {
          const newIndex = Math.max(prev - 10, 0);
          if (
            (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") &&
            listItems[newIndex]
          ) {
            setSelectedChannel(listItems[newIndex] as Channel);
          }
          return newIndex;
        });
      } else if (e.key === "PageDown") {
        // Page down - scroll down by 10 items (same as Ctrl+d)
        e.preventDefault();
        setFocusedIndex((prev) => {
          const newIndex = Math.min(prev + 10, listItems.length - 1);
          if (
            (activeTab === "channels" || activeTab === "favorites" || activeTab === "history") &&
            listItems[newIndex]
          ) {
            setSelectedChannel(listItems[newIndex] as Channel);
          }
          return newIndex;
        });
      }
      
      // Search and filtering
      else if (e.key === "/" || e.key === "i") {
        // Focus search input
        e.preventDefault(); // Prevent the key from being inserted
        const searchInput = document.querySelector('.search-input') as HTMLInputElement;
        if (searchInput) {
          searchInput.focus();
        }
      } else if (e.key === "d" && !e.ctrlKey && !e.altKey && !e.shiftKey) {
        // Clear search
        clearSearch();
      } else if (e.key === "D") {
        // Clear all filters (search + group)
        e.preventDefault();
        clearAllFilters();
      }
      
      // Channel actions
      else if (e.key === "f") {
        if (activeTab === "channels") {
          handleToggleFavorite(listItems[focusedIndex] as Channel);
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
    clearSearch,
    clearAllFilters,
  ]);
}
