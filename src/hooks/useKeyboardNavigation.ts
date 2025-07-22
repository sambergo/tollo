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
  setSelectedChannel: (channel: Channel | null) => void;
  setActiveTab: (tab: Tab) => void;
  handleSelectGroup: (group: string | null) => void;
  handleToggleFavorite: (channel: Channel) => void;
  handlePlayInExternalPlayer: (channel: Channel) => void;
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
  clearGroupSearch: () => void;
  clearAllFilters: () => void;

  // Channel list management
  refreshCurrentChannelList: () => void;

  // Group management
  selectAllGroups: () => void;
  unselectAllGroups: () => void;
  toggleGroupDisplayMode: () => void;
  toggleCurrentGroupSelection: () => void;

  // Video controls
  toggleMute: () => void;
  toggleFullscreen: () => void;
  togglePlayPause: () => void;
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
  handlePlayInExternalPlayer,
  savedFilters,
  onSaveFilter,
  onApplyFilter,
  clearSearch,
  clearGroupSearch,
  clearAllFilters,
  refreshCurrentChannelList,
  selectAllGroups,
  unselectAllGroups,
  toggleGroupDisplayMode,
  toggleCurrentGroupSelection,
  toggleMute,
  toggleFullscreen,
  togglePlayPause,
}: UseKeyboardNavigationProps) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const focusedElement = document.activeElement;

      // Handle escape key when input fields are focused
      if (focusedElement && focusedElement.tagName === "INPUT") {
        if (e.key === "Escape") {
          e.preventDefault();
          (focusedElement as HTMLInputElement).blur();
          return;
        } else {
          // Let input handle the key normally
          return;
        }
      }

      // Handle escape key when input fields are NOT focused
      if (e.key === "Escape") {
        // If search input is not focused, clear the search based on current tab
        if (activeTab === "channels") {
          clearSearch();
        } else if (activeTab === "groups") {
          clearGroupSearch();
        }
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

      // Handle Tab key navigation separately to prevent conflicts
      if (e.key === "Tab" && !e.ctrlKey && !e.altKey && !e.shiftKey) {
        e.preventDefault();
        const tabs: Tab[] = [
          "channels",
          "favorites",
          "groups",
          "history",
          "help",
          "settings",
        ];
        const currentIndex = tabs.indexOf(activeTab);

        // Tab - Next tab
        const nextIndex = (currentIndex + 1) % tabs.length;
        setActiveTab(tabs[nextIndex]);
        setFocusedIndex(0);
        setSelectedChannel(null);
        return;
      }

      // Tab navigation with Ctrl+J and Ctrl+K keys
      if (e.ctrlKey && e.key === "j" && !e.altKey && !e.shiftKey) {
        e.preventDefault(); // Prevent default tab behavior
        const tabs: Tab[] = [
          "channels",
          "favorites",
          "groups",
          "history",
          "help",
          "settings",
        ];
        const currentIndex = tabs.indexOf(activeTab);
        const nextIndex = (currentIndex + 1) % tabs.length;
        setActiveTab(tabs[nextIndex]);
        setFocusedIndex(0);
        setSelectedChannel(null);
        return;
      }

      if (e.ctrlKey && e.key === "k" && !e.altKey && !e.shiftKey) {
        e.preventDefault(); // Prevent default tab behavior
        const tabs: Tab[] = [
          "channels",
          "favorites",
          "groups",
          "history",
          "help",
          "settings",
        ];
        const currentIndex = tabs.indexOf(activeTab);
        const prevIndex = (currentIndex - 1 + tabs.length) % tabs.length;
        setActiveTab(tabs[prevIndex]);
        setFocusedIndex(0);
        setSelectedChannel(null);
        return;
      }

      // Navigation within lists
      if (e.key === "j" || e.key === "ArrowDown") {
        setFocusedIndex((prev) => {
          const newIndex = Math.min(prev + 1, listItems.length - 1);
          // Auto-select channel when navigating in channel-related tabs
          if (
            (activeTab === "channels" ||
              activeTab === "favorites" ||
              activeTab === "history") &&
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
            (activeTab === "channels" ||
              activeTab === "favorites" ||
              activeTab === "history") &&
            listItems[newIndex]
          ) {
            setSelectedChannel(listItems[newIndex] as Channel);
          }
          return newIndex;
        });
      }

      // Selection and interaction
      else if (e.key === "l" || e.key === "ArrowRight") {
        // Set selected channel if null, then play/pause video preview
        if (
          !selectedChannel &&
          (activeTab === "channels" ||
            activeTab === "favorites" ||
            activeTab === "history") &&
          listItems[focusedIndex]
        ) {
          setSelectedChannel(listItems[focusedIndex] as Channel);
        }
        togglePlayPause();
      } else if (e.key === "Enter" || e.key === "o") {
        if (
          activeTab === "channels" ||
          activeTab === "favorites" ||
          activeTab === "history"
        ) {
          handlePlayInExternalPlayer(listItems[focusedIndex] as Channel);
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
          (activeTab === "channels" ||
            activeTab === "favorites" ||
            activeTab === "history") &&
          listItems[firstVisibleIndex]
        ) {
          setSelectedChannel(listItems[firstVisibleIndex] as Channel);
        }
      } else if (e.key === "G" && !e.ctrlKey && !e.altKey) {
        // Go to last item in current view
        const currentPage = Math.floor(focusedIndex / 200);
        const lastVisibleIndex = Math.min(
          (currentPage + 1) * 200 - 1,
          listItems.length - 1,
        );
        setFocusedIndex(lastVisibleIndex);
        if (
          (activeTab === "channels" ||
            activeTab === "favorites" ||
            activeTab === "history") &&
          listItems[lastVisibleIndex]
        ) {
          setSelectedChannel(listItems[lastVisibleIndex] as Channel);
        }
      } else if (e.key === "Home") {
        // Go to first item in current view
        const firstVisibleIndex = Math.floor(focusedIndex / 200) * 200;
        setFocusedIndex(firstVisibleIndex);
        if (
          (activeTab === "channels" ||
            activeTab === "favorites" ||
            activeTab === "history") &&
          listItems[firstVisibleIndex]
        ) {
          setSelectedChannel(listItems[firstVisibleIndex] as Channel);
        }
      } else if (e.key === "End") {
        // Go to last item in current view
        const currentPage = Math.floor(focusedIndex / 200);
        const lastVisibleIndex = Math.min(
          (currentPage + 1) * 200 - 1,
          listItems.length - 1,
        );
        setFocusedIndex(lastVisibleIndex);
        if (
          (activeTab === "channels" ||
            activeTab === "favorites" ||
            activeTab === "history") &&
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
            (activeTab === "channels" ||
              activeTab === "favorites" ||
              activeTab === "history") &&
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
            (activeTab === "channels" ||
              activeTab === "favorites" ||
              activeTab === "history") &&
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
            (activeTab === "channels" ||
              activeTab === "favorites" ||
              activeTab === "history") &&
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
            (activeTab === "channels" ||
              activeTab === "favorites" ||
              activeTab === "history") &&
            listItems[newIndex]
          ) {
            setSelectedChannel(listItems[newIndex] as Channel);
          }
          return newIndex;
        });
      }

      // Pagination controls - H/L for previous/next page
      else if (e.key === "H") {
        // Previous page
        e.preventDefault();
        const ITEMS_PER_PAGE = 200;
        const currentPage = Math.floor(focusedIndex / ITEMS_PER_PAGE);
        if (currentPage > 0) {
          const newIndex = (currentPage - 1) * ITEMS_PER_PAGE;
          setFocusedIndex(newIndex);
          if (
            (activeTab === "channels" ||
              activeTab === "favorites" ||
              activeTab === "history") &&
            listItems[newIndex]
          ) {
            setSelectedChannel(listItems[newIndex] as Channel);
          }
        }
      } else if (e.key === "L") {
        // Next page
        e.preventDefault();
        const ITEMS_PER_PAGE = 200;
        const currentPage = Math.floor(focusedIndex / ITEMS_PER_PAGE);
        const totalPages = Math.ceil(listItems.length / ITEMS_PER_PAGE);
        if (currentPage < totalPages - 1) {
          const newIndex = (currentPage + 1) * ITEMS_PER_PAGE;
          const maxIndex = Math.min(newIndex, listItems.length - 1);
          setFocusedIndex(maxIndex);
          if (
            (activeTab === "channels" ||
              activeTab === "favorites" ||
              activeTab === "history") &&
            listItems[maxIndex]
          ) {
            setSelectedChannel(listItems[maxIndex] as Channel);
          }
        }
      }

      // Search and filtering
      else if (e.key === "/" || e.key === "i") {
        // Focus search input
        e.preventDefault(); // Prevent the key from being inserted
        const searchInput = document.querySelector(
          ".search-input",
        ) as HTMLInputElement;
        if (searchInput) {
          searchInput.focus();
        }
      } else if (e.key === "c" && !e.ctrlKey && !e.altKey && !e.shiftKey) {
        // Clear search and focus search input (combination of d and i)
        e.preventDefault();
        if (activeTab === "channels") {
          clearSearch();
        } else if (activeTab === "groups") {
          clearGroupSearch();
        }
        const searchInput = document.querySelector(
          ".search-input",
        ) as HTMLInputElement;
        if (searchInput) {
          searchInput.focus();
        }
      } else if (e.key === "d" && !e.ctrlKey && !e.altKey && !e.shiftKey) {
        // Clear search based on current tab
        if (activeTab === "channels") {
          clearSearch();
        } else if (activeTab === "groups") {
          clearGroupSearch();
        }
      } else if (e.key === "D") {
        // Clear all filters based on current tab
        e.preventDefault();
        if (activeTab === "channels") {
          clearSearch();
          clearAllFilters();
        } else if (activeTab === "groups") {
          clearGroupSearch();
        }
      }

      // Channel actions
      else if (e.key === "F") {
        if (activeTab === "channels") {
          handleToggleFavorite(listItems[focusedIndex] as Channel);
        }
      }

      // Channel list management
      else if (e.key === "R") {
        // Refresh current channel list
        refreshCurrentChannelList();
      }

      // Group management
      else if (e.key === "A") {
        // Select all groups
        selectAllGroups();
      } else if (e.key === "U") {
        // Unselect all groups
        unselectAllGroups();
      } else if (e.key === "t") {
        // Toggle group display mode
        toggleGroupDisplayMode();
      } else if (e.key === " ") {
        // Toggle current group selection or play/pause video preview
        e.preventDefault(); // Prevent page scroll
        if (activeTab === "groups") {
          toggleCurrentGroupSelection();
        } else if (
          activeTab === "channels" ||
          activeTab === "favorites" ||
          activeTab === "history"
        ) {
          // Set selected channel if null, then play/pause video preview
          if (!selectedChannel && listItems[focusedIndex]) {
            setSelectedChannel(listItems[focusedIndex] as Channel);
          }
          togglePlayPause();
        }
      }

      // Video controls
      else if (e.key === "m") {
        // Toggle mute
        toggleMute();
      } else if (e.key === "f") {
        // Toggle fullscreen
        toggleFullscreen();
      }

      // Clear selected channel
      else if (
        e.key === "h" ||
        e.key === "ArrowLeft" ||
        e.key === "Backspace"
      ) {
        // Clear selected channel to return to "Select a channel to start watching" state
        setSelectedChannel(null);
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
    clearGroupSearch,
    clearAllFilters,
    refreshCurrentChannelList,
    selectAllGroups,
    unselectAllGroups,
    toggleGroupDisplayMode,
    toggleCurrentGroupSelection,
    toggleMute,
    toggleFullscreen,
    togglePlayPause,
    setSelectedChannel,
  ]);
}
