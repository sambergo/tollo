// Icon components (using SVG for better styling)
export const TvIcon = () => (
  <svg
    className="nav-icon"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <rect x="2" y="3" width="20" height="14" rx="2" ry="2" />
    <line x1="8" y1="21" x2="16" y2="21" />
    <line x1="12" y1="17" x2="12" y2="21" />
  </svg>
);

export const HeartIcon = () => (
  <svg
    className="nav-icon"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    viewBox="0 0 24 24"
  >
    <path d="M12 21s-6.2-5.05-8.2-7.05A5.5 5.5 0 0 1 12 5.5a5.5 5.5 0 0 1 8.2 8.45C18.2 15.95 12 21 12 21z" />
  </svg>
);

export const UsersIcon = () => (
  <svg
    className="nav-icon"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    viewBox="0 0 24 24"
  >
    <circle cx="12" cy="8" r="4" />
    <path d="M2 20c0-3.31 3.58-6 8-6s8 2.69 8 6" />
  </svg>
);

export const HistoryIcon = () => (
  <svg
    className="nav-icon"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    viewBox="0 0 24 24"
  >
    <path d="M3 12a9 9 0 1 1 9 9" />
    <polyline points="3 12 7 16 11 12" />
    <line x1="12" y1="7" x2="12" y2="12" />
  </svg>
);

export const HelpIcon = () => (
  <svg
    className="nav-icon"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    viewBox="0 0 24 24"
  >
    <rect x="2" y="6" width="20" height="12" rx="2" />
    <path d="M6 10h.01" />
    <path d="M10 10h.01" />
    <path d="M14 10h.01" />
    <path d="M18 10h.01" />
    <path d="M6 14h.01" />
    <path d="M10 14h.01" />
    <path d="M14 14h.01" />
    <path d="M18 14h.01" />
  </svg>
);

export const SettingsIcon = () => (
  <svg
    className="nav-icon"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    viewBox="0 0 24 24"
  >
    <circle cx="12" cy="12" r="3" />
    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1 1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
  </svg>
);

export const PlayIcon = () => (
  <svg
    className="video-placeholder-icon"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <polygon points="5,3 19,12 5,21" />
  </svg>
);

export const SignalIcon = () => (
  <svg
    className="meta-icon"
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path d="2 20h.01" />
    <path d="7 20v-4" />
    <path d="12 20v-8" />
    <path d="17 20V8" />
    <path d="22 4v16" />
  </svg>
);

export const StarIcon = () => (
  <svg
    className="meta-icon rating-star"
    fill="currentColor"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <polygon points="12,2 15.09,8.26 22,9.27 17,14.14 18.18,21.02 12,17.77 5.82,21.02 7,14.14 2,9.27 8.91,8.26" />
  </svg>
);
