interface GroupListProps {
  groups: string[];
  selectedGroup: string | null;
  focusedIndex: number;
  onSelectGroup: (group: string | null) => void;
}

export default function GroupList({ 
  groups, 
  selectedGroup, 
  focusedIndex, 
  onSelectGroup 
}: GroupListProps) {
  return (
    <ul className="group-list">
      <li
        className={`group-item ${selectedGroup === null ? "selected" : ""} ${focusedIndex === 0 ? "focused" : ""}`}
        onClick={() => onSelectGroup(null)}
      >
        All Groups
      </li>
      {groups.map((group, index) => (
        <li
          key={group}
          className={`group-item ${selectedGroup === group ? "selected" : ""} ${focusedIndex === index + 1 ? "focused" : ""}`}
          onClick={() => onSelectGroup(group)}
        >
          {group}
        </li>
      ))}
    </ul>
  );
} 