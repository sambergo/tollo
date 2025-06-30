import type { SavedFilter } from "../hooks/useSavedFilters";

interface SavedFiltersProps {
  savedFilters: SavedFilter[];
  onApplyFilter: (filter: SavedFilter) => void;
}

export default function SavedFilters({ savedFilters, onApplyFilter }: SavedFiltersProps) {
  if (savedFilters.length === 0) {
    return null;
  }

  return (
    <div className="saved-filters">
      <h3 className="saved-filters-title">Saved Filters</h3>
      <div className="saved-filters-list">
        {savedFilters.map((filter) => (
          <div key={filter.slot_number} className="saved-filter-item">
            <button
              className="saved-filter-button"
              onClick={() => onApplyFilter(filter)}
              title={`Press ${filter.slot_number} to apply this filter`}
            >
              <div className="filter-first-line">
                <span className="filter-key">{filter.slot_number}</span>
                {filter.search_query && (
                  <span className="filter-query">{filter.search_query}</span>
                )}
              </div>
              {filter.selected_group && (
                <div className="filter-group">{filter.selected_group}</div>
              )}
            </button>
          </div>
        ))}
      </div>
      <div className="saved-filters-help">
        <p>Press number keys (0-9) to apply</p>
        <p>Press Alt+number to save current filter</p>
      </div>
    </div>
  );
} 