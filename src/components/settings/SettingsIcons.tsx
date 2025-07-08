// Settings-specific icon components
export const PlayIcon = () => (
  <svg
    className="settings-icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <polygon points="5,3 19,12 5,21" stroke="currentColor" fill="none" />
  </svg>
);

export const ClockIcon = () => (
  <svg
    className="settings-icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <circle cx="12" cy="12" r="10" stroke="currentColor" fill="none" />
    <polyline points="12,6 12,12 16,14" stroke="currentColor" fill="none" />
  </svg>
);

export const ListIcon = () => (
  <svg
    className="settings-icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <line x1="8" y1="6" x2="21" y2="6" stroke="currentColor" />
    <line x1="8" y1="12" x2="21" y2="12" stroke="currentColor" />
    <line x1="8" y1="18" x2="21" y2="18" stroke="currentColor" />
    <line x1="3" y1="6" x2="3.01" y2="6" stroke="currentColor" />
    <line x1="3" y1="12" x2="3.01" y2="12" stroke="currentColor" />
    <line x1="3" y1="18" x2="3.01" y2="18" stroke="currentColor" />
  </svg>
);

export const ImageIcon = () => (
  <svg
    className="settings-icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <rect
      x="3"
      y="3"
      width="18"
      height="18"
      rx="2"
      ry="2"
      stroke="currentColor"
      fill="none"
    />
    <circle cx="8.5" cy="8.5" r="1.5" stroke="currentColor" fill="none" />
    <polyline points="21,15 16,10 5,21" stroke="currentColor" fill="none" />
  </svg>
);

export const EditIcon = () => (
  <svg
    className="action-icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path
      d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
      stroke="currentColor"
      fill="none"
    />
    <path
      d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
      stroke="currentColor"
      fill="none"
    />
  </svg>
);

export const TrashIcon = () => (
  <svg
    className="action-icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <polyline points="3,6 5,6 21,6" stroke="currentColor" fill="none" />
    <path
      d="M19,6v14a2,2,0,0,1-2,2H7a2,2,0,0,1-2-2V6m3,0V4a2,2,0,0,1,2-2h4a2,2,0,0,1,2,2V6"
      stroke="currentColor"
      fill="none"
    />
    <line x1="10" y1="11" x2="10" y2="17" stroke="currentColor" />
    <line x1="14" y1="11" x2="14" y2="17" stroke="currentColor" />
  </svg>
);

export const RefreshIcon = () => (
  <svg
    className="action-icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M23 4v6h-6" stroke="currentColor" fill="none" />
    <path
      d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"
      stroke="currentColor"
      fill="none"
    />
  </svg>
);

export const CheckIcon = () => (
  <svg
    className="action-icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <polyline points="20,6 9,17 4,12" stroke="currentColor" fill="none" />
  </svg>
);

export const XIcon = () => (
  <svg
    className="action-icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <line x1="18" y1="6" x2="6" y2="18" stroke="currentColor" />
    <line x1="6" y1="6" x2="18" y2="18" stroke="currentColor" />
  </svg>
);

export const StarIcon = ({ filled }: { filled: boolean }) => (
  <svg
    className="action-icon"
    viewBox="0 0 24 24"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
    fill={filled ? "currentColor" : "none"}
  >
    <polygon
      points="12,2 15.09,8.26 22,9.27 17,14.14 18.18,21.02 12,17.77 5.82,21.02 7,14.14 2,9.27 8.91,8.26"
      stroke="currentColor"
    />
  </svg>
);

export const LoadingIcon = () => (
  <svg
    className="action-icon animate-spin"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M21 12a9 9 0 11-6.219-8.56" stroke="currentColor" fill="none" />
  </svg>
);

export const FilterIcon = () => (
  <svg
    className="settings-icon"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <polygon
      points="22,3 2,3 10,12.46 10,19 14,21 14,12.46"
      stroke="currentColor"
      fill="none"
    />
  </svg>
);
