import { useState, useEffect, useRef } from "react";
import { useUIStore, useChannelStore } from "../stores";
import { GroupDisplayMode } from "../stores/uiStore";

const GROUPS_PER_PAGE = 200; // Match channels paging for consistency

export default function GroupList() {
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const groupListRef = useRef<HTMLUListElement>(null);

  // Get state and actions from stores
  const {
    selectedGroup,
    focusedIndex,
    enabledGroups,
    groupDisplayMode,
    groupSearchTerm,
    setSelectedGroup,
    setFocusedIndex,
    toggleGroupEnabled,
    setGroupDisplayMode,
    setGroupSearchTerm,
    selectAllGroups,
    unselectAllGroups,
    setActiveTab,
  } = useUIStore();

  const { groups, selectedChannelListId } = useChannelStore();

  // Filter groups based on search term
  const filteredGroups = groups.filter((group: string) =>
    group.toLowerCase().includes(groupSearchTerm.toLowerCase()),
  );

  // Reset to first page when search term or mode changes
  useEffect(() => {
    setCurrentPage(1);
  }, [groupSearchTerm, groupDisplayMode]);

  // Auto-change page if focused item is on a different page
  useEffect(() => {
    if (filteredGroups.length === 0) return;

    // Calculate total items including "All Groups" option if in AllGroups mode
    const totalItems =
      groupDisplayMode === GroupDisplayMode.AllGroups
        ? filteredGroups.length + 1
        : filteredGroups.length;

    const requiredPage = Math.ceil((focusedIndex + 1) / GROUPS_PER_PAGE);

    // Change page if focused item is on a different page
    if (
      requiredPage !== currentPage &&
      requiredPage <= Math.ceil(totalItems / GROUPS_PER_PAGE)
    ) {
      setCurrentPage(requiredPage);
    }
  }, [focusedIndex, filteredGroups.length, currentPage, groupDisplayMode]);

  // Calculate pagination
  const totalPages = Math.ceil(
    (groupDisplayMode === GroupDisplayMode.AllGroups
      ? filteredGroups.length + 1
      : filteredGroups.length) / GROUPS_PER_PAGE,
  );
  const startIndex = (currentPage - 1) * GROUPS_PER_PAGE;
  const endIndex = startIndex + GROUPS_PER_PAGE;

  // Slice groups for current page, handling "All Groups" option
  const getPagedGroups = () => {
    if (groupDisplayMode === GroupDisplayMode.AllGroups) {
      // Include "All Groups" as first item, then filtered groups
      const allItems = [null, ...filteredGroups]; // null represents "All Groups"
      return allItems.slice(startIndex, endIndex);
    } else {
      return filteredGroups.slice(startIndex, endIndex);
    }
  };

  const currentGroups = getPagedGroups();

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setDropdownOpen(false);
      }
    };

    if (dropdownOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [dropdownOpen]);

  // Scroll focused item into view
  useEffect(() => {
    const focusedElement = groupListRef.current?.querySelector(
      ".group-item.focused",
    );
    if (focusedElement) {
      focusedElement.scrollIntoView({
        behavior: "smooth",
        block: "center",
        inline: "nearest",
      });
    }
  }, [focusedIndex]);

  const handleClearSearch = () => {
    setGroupSearchTerm("");
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.ctrlKey) {
      switch (e.key) {
        case "w":
          e.preventDefault();
          // Remove last word
          const input = e.currentTarget;
          const value = input.value;
          const cursorPos = input.selectionStart || 0;
          const beforeCursor = value.substring(0, cursorPos);
          const afterCursor = value.substring(cursorPos);

          // Find the start of the last word before cursor
          const words = beforeCursor.trimEnd();
          const lastSpaceIndex = words.lastIndexOf(" ");
          const newBeforeCursor =
            lastSpaceIndex >= 0 ? words.substring(0, lastSpaceIndex + 1) : "";

          const newValue = newBeforeCursor + afterCursor;
          setGroupSearchTerm(newValue);

          // Set cursor position after the removed word
          setTimeout(() => {
            input.setSelectionRange(
              newBeforeCursor.length,
              newBeforeCursor.length,
            );
          }, 0);
          break;

        case "u":
          e.preventDefault();
          // Clear entire input
          setGroupSearchTerm("");
          break;

        case "c":
          e.preventDefault();
          // Unfocus the input
          e.currentTarget.blur();
          break;
      }
    }
  };

  const handleSelectAllGroups = () => {
    if (selectedChannelListId) {
      selectAllGroups(groups, selectedChannelListId);
    }
    setDropdownOpen(false);
  };

  const handleUnselectAllGroups = () => {
    if (selectedChannelListId) {
      unselectAllGroups(groups, selectedChannelListId);
    }
    setDropdownOpen(false);
  };

  const handleToggleGroupEnabled = (group: string) => {
    if (selectedChannelListId) {
      toggleGroupEnabled(group, selectedChannelListId);
    }
  };

  // Pagination handlers
  const handlePageChange = (page: number) => {
    setCurrentPage(page);
    // Update focused index to first item of new page to maintain consistency
    const newFocusedIndex = (page - 1) * GROUPS_PER_PAGE;
    setFocusedIndex(newFocusedIndex);
  };

  const getPageNumbers = () => {
    const pages = [];
    const maxVisible = 5;
    let start = Math.max(1, currentPage - Math.floor(maxVisible / 2));
    let end = Math.min(totalPages, start + maxVisible - 1);

    if (end - start + 1 < maxVisible) {
      start = Math.max(1, end - maxVisible + 1);
    }

    for (let i = start; i <= end; i++) {
      pages.push(i);
    }
    return pages;
  };

  return (
    <div className="group-list-container">
      {/* Search Input */}
      <div className="search-container">
        <div className="search-input-wrapper">
          <input
            type="text"
            className="search-input"
            placeholder="Search groups..."
            value={groupSearchTerm}
            onChange={(e) => setGroupSearchTerm(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          {groupSearchTerm && (
            <button
              className="clear-search-btn"
              onClick={handleClearSearch}
              type="button"
            >
              ×
            </button>
          )}
        </div>
        {groupSearchTerm && (
          <div className="search-results-count">
            Showing {filteredGroups.length} of {groups.length} groups
          </div>
        )}
      </div>

      {/* Pagination Info */}
      <div className="pagination-info">
        <span className="group-count">
          Showing {startIndex + 1}-
          {Math.min(
            endIndex,
            groupDisplayMode === GroupDisplayMode.AllGroups
              ? filteredGroups.length + 1
              : filteredGroups.length,
          )}{" "}
          of{" "}
          {groupDisplayMode === GroupDisplayMode.AllGroups
            ? filteredGroups.length + 1
            : filteredGroups.length}{" "}
          groups
          {totalPages > 1 && ` (Page ${currentPage} of ${totalPages})`}
        </span>
      </div>

      {/* Mode Toggle Buttons */}
      <div className="group-mode-controls">
        <button
          className={`mode-btn ${groupDisplayMode === GroupDisplayMode.EnabledGroups ? "active" : ""}`}
          onClick={() => setGroupDisplayMode(GroupDisplayMode.EnabledGroups)}
        >
          Enabled Groups
        </button>
        <button
          className={`mode-btn ${groupDisplayMode === GroupDisplayMode.AllGroups ? "active" : ""}`}
          onClick={() => setGroupDisplayMode(GroupDisplayMode.AllGroups)}
        >
          Select group
        </button>

        {/* Bulk Actions Dropdown - Only show in Enabled Groups mode */}
        {groupDisplayMode === GroupDisplayMode.EnabledGroups && (
          <div className="bulk-actions-dropdown" ref={dropdownRef}>
            <button
              className="dropdown-toggle"
              onClick={() => setDropdownOpen(!dropdownOpen)}
            >
              ⋮
            </button>
            {dropdownOpen && (
              <div className="dropdown-menu">
                <button
                  className="dropdown-item"
                  onClick={handleSelectAllGroups}
                >
                  Select All
                </button>
                <button
                  className="dropdown-item"
                  onClick={handleUnselectAllGroups}
                >
                  Unselect All
                </button>
              </div>
            )}
          </div>
        )}
      </div>

      <ul className="group-list" ref={groupListRef}>
        {currentGroups.map((group: string | null, index: number) => {
          if (group === null) {
            // "All Groups" option
            const globalIndex = startIndex + index;
            return (
              <li
                key="all-groups"
                className={`group-item ${selectedGroup === null ? "selected" : ""} ${focusedIndex === globalIndex ? "focused" : ""}`}
                onClick={() => {
                  setSelectedGroup(null);
                  setActiveTab("channels");
                }}
              >
                All Groups
              </li>
            );
          }

          // Regular group
          const globalIndex = startIndex + index;
          const isSelected = selectedGroup === group;
          const isFocused = focusedIndex === globalIndex;
          const isEnabled = enabledGroups.has(group);

          return (
            <li
              key={group}
              className={`group-item ${isSelected ? "selected" : ""} ${isFocused ? "focused" : ""}`}
            >
              <div className="group-item-content">
                {/* Checkbox for enabling/disabling groups (only in Enabled Groups mode) */}
                {groupDisplayMode === GroupDisplayMode.EnabledGroups && (
                  <input
                    type="checkbox"
                    className="group-checkbox"
                    checked={isEnabled}
                    onChange={() => handleToggleGroupEnabled(group)}
                  />
                )}

                {/* Group name - different click behavior based on mode */}
                <span
                  className="group-name"
                  onClick={() => {
                    if (groupDisplayMode === GroupDisplayMode.EnabledGroups) {
                      // In enabled groups mode, toggle the checkbox
                      handleToggleGroupEnabled(group);
                    } else {
                      // In all groups mode, select the group and navigate back to channels
                      setSelectedGroup(group);
                      setActiveTab("channels");
                    }
                  }}
                >
                  {group}
                </span>
              </div>
            </li>
          );
        })}
      </ul>

      {/* Pagination Controls */}
      {totalPages > 1 && (
        <div className="pagination-controls">
          <button
            className="pagination-btn"
            onClick={() => handlePageChange(1)}
            disabled={currentPage === 1}
            title="First page"
          >
            ««
          </button>
          <button
            className="pagination-btn"
            onClick={() => handlePageChange(currentPage - 1)}
            disabled={currentPage === 1}
            title="Previous page"
          >
            ‹
          </button>
          {getPageNumbers().map((page) => (
            <button
              key={page}
              className={`pagination-btn ${page === currentPage ? "active" : ""}`}
              onClick={() => handlePageChange(page)}
            >
              {page}
            </button>
          ))}
          <button
            className="pagination-btn"
            onClick={() => handlePageChange(currentPage + 1)}
            disabled={currentPage === totalPages}
            title="Next page"
          >
            ›
          </button>
          <button
            className="pagination-btn"
            onClick={() => handlePageChange(totalPages)}
            disabled={currentPage === totalPages}
            title="Last page"
          >
            »»
          </button>
        </div>
      )}
    </div>
  );
}
