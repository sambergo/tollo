import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import type { RemoteInfo, RemotePlayback } from "../types/remote";

interface RemoteState {
  info: RemoteInfo | null;
  playback: RemotePlayback | null;
  previewSuspended: boolean;
  previewPending: boolean;
  setInfo: (info: RemoteInfo) => void;
  setPlayback: (playback: RemotePlayback) => void;
  suspendPreview: () => void;
  refresh: () => Promise<void>;
}

export const useRemoteStore = create<RemoteState>((set) => ({
  info: null,
  playback: null,
  previewSuspended: false,
  previewPending: false,
  setInfo: (info) =>
    set({
      info,
      ...(!info.enabled
        ? { previewSuspended: false, previewPending: false }
        : {}),
    }),
  setPlayback: (playback) =>
    set({
      playback,
      previewPending: false,
      previewSuspended: playback.channel !== null,
    }),
  suspendPreview: () => set({ previewSuspended: true, previewPending: true }),
  refresh: async () => {
    const [info, playback] = await Promise.all([
      invoke<RemoteInfo>("get_remote_info"),
      invoke<RemotePlayback>("get_remote_playback"),
    ]);
    // A status poll may arrive before mpv starts. Keep the preview detached
    // until the playback event confirms the outcome of the pending action.
    set((current) => ({
      info,
      playback,
      previewPending: info.enabled && current.previewPending,
      previewSuspended:
        info.enabled && (current.previewPending || playback.channel !== null),
    }));
  },
}));
