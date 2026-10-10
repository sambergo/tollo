export interface BackupSettings {
  player_command: string;
  clipboard_command: string;
  cache_duration_hours: number;
  enable_preview: boolean;
  mute_on_start: boolean;
  show_controls: boolean;
  autoplay: boolean;
}

export interface ImportPreview {
  token: string;
  exported_at: string;
  app_version: string;
  settings: BackupSettings;
  list_names: string[];
  default_list: string | null;
  favorite_count: number;
  filter_count: number;
  group_count: number;
  local_playlist_count: number;
}

export type ExportResult = string | null;
