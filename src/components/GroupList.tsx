import { useState, useEffect, useRef } from "react";
import { useUIStore, useChannelStore } from "../stores";
import { GroupDisplayMode } from "../stores/uiStore";

export default function GroupList() {
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Get state and actions from stores
  const {
    selectedGroup,
    focusedIndex,
    enabledGroups,
    groupDisplayMode,
    setSelectedGroup,
    toggleGroupEnabled,
    setGroupDisplayMode,
    selectAllGroups,
    unselectAllGroups,
    setActiveTab,
  } = useUIStore();

  const { groups, selectedChannelListId } = useChannelStore();

  // Filter groups based on search term
  const filteredGroups = groups.filter((group: string) =>
    group.toLowerCase().includes(searchTerm.toLowerCase()),
  );

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

  const handleClearSearch = () => {
    setSearchTerm("");
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

  return (
    <div className="group-list-container">
      {/* Search Input */}
      <div className="search-container">
        <div className="search-input-wrapper">
          <input
            type="text"
            className="search-input"
            placeholder="Search groups..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
          {searchTerm && (
            <button
              className="clear-search-btn"
              onClick={handleClearSearch}
              type="button"
            >
              ×
            </button>
          )}
        </div>
        {searchTerm && (
          <div className="search-results-count">
            Showing {filteredGroups.length} of {groups.length} groups
          </div>
        )}
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

      <ul className="group-list">
        {/* All Groups option for AllGroups mode */}
        {groupDisplayMode === GroupDisplayMode.AllGroups && (
          <li
            className={`group-item ${selectedGroup === null ? "selected" : ""} ${focusedIndex === 0 ? "focused" : ""}`}
            onClick={() => {
              setSelectedGroup(null);
              setActiveTab("channels");
            }}
          >
            All Groups
          </li>
        )}

        {filteredGroups.map((group: string, index: number) => {
          const isSelected = selectedGroup === group;
          const adjustedIndex =
            groupDisplayMode === GroupDisplayMode.EnabledGroups
              ? index
              : index + 1;
          const isFocused = focusedIndex === adjustedIndex;
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
    </div>
  );
}
