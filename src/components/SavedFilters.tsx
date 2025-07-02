import { useState, useEffect } from "react";
import {
  useSearchStore,
  useUIStore,
  useFilterStore,
  useChannelStore,
  GroupDisplayMode,
  type SavedFilter,
} from "../stores";

export default function SavedFilters() {
  const [currentPage, setCurrentPage] = useState(0);
  const { setSearchQuery } = useSearchStore();
  const {
    setSelectedGroup,
    setGroupDisplayMode,
    setActiveTab,
    setFocusedIndex,
  } = useUIStore();
  const { savedFilters, loadSavedFilters } = useFilterStore();
  const { selectedChannelListId } = useChannelStore();

  // Load saved filters when channel list changes
  useEffect(() => {
    loadSavedFilters(selectedChannelListId);
  }, [selectedChannelListId, loadSavedFilters]);

  // Reset to first page when filters change
  useEffect(() => {
    setCurrentPage(0);
  }, [savedFilters]);

  const handleApplyFilter = (filter: SavedFilter) => {
    // Apply the search query
    setSearchQuery(filter.search_query);

    // Apply the group selection and set appropriate display mode
    setSelectedGroup(filter.selected_group);

    // If the filter has a selected group, switch to AllGroups mode to make the group filter active
    // If no group is selected, use EnabledGroups mode
    if (filter.selected_group) {
      setGroupDisplayMode(GroupDisplayMode.AllGroups);
    } else {
      setGroupDisplayMode(GroupDisplayMode.EnabledGroups);
    }

    // Switch to channels tab to see the results
    setActiveTab("channels");
    setFocusedIndex(0);
  };

  if (savedFilters.length === 0) {
    return (
      <div className="saved-filters">
        <div className="saved-filters-header">
          <h3 className="saved-filters-title">Saved Filters</h3>
        </div>
        <div className="saved-filters-help">
          <p>No saved filters yet</p>
          <p>Press Alt+number to save current filter</p>
        </div>
      </div>
    );
  }

  // Sort filters by slot number, treating 0 as 10
  const sortedFilters = [...savedFilters].sort((a, b) => {
    const slotA = a.slot_number === 0 ? 10 : a.slot_number;
    const slotB = b.slot_number === 0 ? 10 : b.slot_number;
    return slotA - slotB;
  });

  // Paginate filters - 5 per page
  const filtersPerPage = 5;
  const totalPages = Math.ceil(sortedFilters.length / filtersPerPage);
  const startIndex = currentPage * filtersPerPage;
  const endIndex = startIndex + filtersPerPage;
  const currentFilters = sortedFilters.slice(startIndex, endIndex);

  const goToNextPage = () => {
    setCurrentPage((prev) => (prev + 1) % totalPages);
  };

  const goToPrevPage = () => {
    setCurrentPage((prev) => (prev - 1 + totalPages) % totalPages);
  };

  return (
    <div className="saved-filters">
      <div className="saved-filters-header">
        <h3 className="saved-filters-title">Saved Filters</h3>
      </div>
      <div className="saved-filters-list">
        {currentFilters.map((filter) => (
          <div key={filter.slot_number} className="saved-filter-item">
            <button
              className="saved-filter-button"
              onClick={() => handleApplyFilter(filter)}
              title={`Press ${filter.slot_number === 0 ? "0" : filter.slot_number} to apply this filter`}
            >
              <div className="filter-first-line">
                <span className="filter-key">
                  {filter.slot_number === 0 ? "0" : filter.slot_number}
                </span>
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
      {totalPages > 1 && (
        <div className="saved-filters-nav">
          <button
            className="nav-arrow"
            onClick={goToPrevPage}
            title="Previous page"
          >
            ‹
          </button>
          <span className="page-indicator">
            {currentPage + 1}/{totalPages}
          </span>
          <button
            className="nav-arrow"
            onClick={goToNextPage}
            title="Next page"
          >
            ›
          </button>
        </div>
      )}
      <div className="saved-filters-help">
        <p>Press number keys (0-9) to apply</p>
        <p>Press Alt+number to save current filter</p>
      </div>
    </div>
  );
}
