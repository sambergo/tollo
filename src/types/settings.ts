import type { SavedFilter } from "../stores";

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
