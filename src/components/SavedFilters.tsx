import { useState, useEffect } from "react";
import type { SavedFilter } from "../hooks/useSavedFilters";

interface SavedFiltersProps {
  savedFilters: SavedFilter[];
  onApplyFilter: (filter: SavedFilter) => void;
}

export default function SavedFilters({ savedFilters, onApplyFilter }: SavedFiltersProps) {
  const [currentPage, setCurrentPage] = useState(0);
  
  // Reset to first page when filters change
  useEffect(() => {
    setCurrentPage(0);
  }, [savedFilters]);
  
  if (savedFilters.length === 0) {
    return null;
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
              onClick={() => onApplyFilter(filter)}
              title={`Press ${filter.slot_number === 0 ? '0' : filter.slot_number} to apply this filter`}
            >
              <div className="filter-first-line">
                <span className="filter-key">{filter.slot_number === 0 ? '0' : filter.slot_number}</span>
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