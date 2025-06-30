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
}

export default function GroupList({ 
  groups, 
  selectedGroup, 
  focusedIndex, 
  enabledGroups,
  groupDisplayMode,
  onSelectGroup,
  onToggleGroupEnabled,
  onChangeDisplayMode
}: GroupListProps) {
  return (
    <div className="group-list-container">
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
          All Groups
        </button>
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

        {groups.map((group, index) => {
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
                
                {/* Group name (clickable in all modes) */}
                <span 
                  className="group-name"
                  onClick={() => onSelectGroup(group)}
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