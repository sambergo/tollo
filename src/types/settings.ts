import type { SavedFilter } from "../hooks/useSavedFilters";

export interface ChannelList {
  id: number;
  name: string;
  source: string; // url or file path
  is_default: boolean;
  last_fetched: number | null;
}

export interface ChannelListWithFilters extends ChannelList {
  savedFilters: SavedFilter[];
}

export interface SettingsProps {
  onSelectList: (id: number) => void;
  onFiltersChanged?: () => Promise<void>;
} 