import { useState, useEffect, useRef } from "react";

enum GroupDisplayMode {
  EnabledGroups = 'enabled',
  AllGroups = 'all'
}

interface GroupListProps {
  groups: string[];
  selectedGroup: string | null;
  focusedIndex: number;
  enabledGroups: Set<string>;
  groupDisplayMode: GroupDisplayMode;
  onSelectGroup: (group: string | null) => void;
  onToggleGroupEnabled: (group: string) => void;
  onChangeDisplayMode: (mode: GroupDisplayMode) => void;
  onSelectAllGroups: () => void;
  onUnselectAllGroups: () => void;
}

export default function GroupList({ 
  groups, 
  selectedGroup, 
  focusedIndex, 
  enabledGroups,
  groupDisplayMode,
  onSelectGroup,
  onToggleGroupEnabled,
  onChangeDisplayMode,
  onSelectAllGroups,
  onUnselectAllGroups
}: GroupListProps) {
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Filter groups based on search term
  const filteredGroups = groups.filter(group =>
    group.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setDropdownOpen(false);
      }
    };

    if (dropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [dropdownOpen]);

  const handleClearSearch = () => {
    setSearchTerm("");
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
          onClick={() => onChangeDisplayMode(GroupDisplayMode.EnabledGroups)}
        >
          Enabled Groups
        </button>
        <button 
          className={`mode-btn ${groupDisplayMode === GroupDisplayMode.AllGroups ? "active" : ""}`}
          onClick={() => onChangeDisplayMode(GroupDisplayMode.AllGroups)}
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
                  onClick={() => {
                    onSelectAllGroups();
                    setDropdownOpen(false);
                  }}
                >
                  Select All
                </button>
                <button 
                  className="dropdown-item"
                  onClick={() => {
                    onUnselectAllGroups();
                    setDropdownOpen(false);
                  }}
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
            onClick={() => onSelectGroup(null)}
          >
            All Groups
          </li>
        )}

        {filteredGroups.map((group, index) => {
          const isSelected = selectedGroup === group;
          const adjustedIndex = (groupDisplayMode === GroupDisplayMode.EnabledGroups) ? index : index + 1;
          const isFocused = focusedIndex === adjustedIndex;
          const isEnabled = enabledGroups.has(group);

          return (
            <li
              key={group}
              className={`group-item ${isSelected ? "selected" : ""} ${isFocused ? "focused" : ""}`}
            >
              <div className="group-item-content">
                {/* Checkbox for enabling/disabling groups (only in Enabled Groups mode) */}
                {(groupDisplayMode === GroupDisplayMode.EnabledGroups) && (
                  <input
                    type="checkbox"
                    className="group-checkbox"
                    checked={isEnabled}
                    onChange={() => onToggleGroupEnabled(group)}
                  />
                )}
                
                {/* Group name - different click behavior based on mode */}
                <span 
                  className="group-name"
                  onClick={() => {
                    if (groupDisplayMode === GroupDisplayMode.EnabledGroups) {
                      // In enabled groups mode, toggle the checkbox
                      onToggleGroupEnabled(group);
                    } else {
                      // In all groups mode, select the group
                      onSelectGroup(group);
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