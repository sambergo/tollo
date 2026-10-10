export interface RemoteInfo {
  supported: boolean;
  enabled: boolean;
  running: boolean;
  urls: string[];
  token: string;
  port: number;
  error: string | null;
}

export interface RemotePlayback {
  channel: string | null;
  state: "stopped" | "loading" | "playing" | "paused" | "failed";
  paused: boolean;
  muted: boolean;
  volume: number;
  error: string | null;
}
