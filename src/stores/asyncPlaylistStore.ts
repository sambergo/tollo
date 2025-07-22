import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface PlaylistFetchStatus {
  id: number;
  status: string; // "starting", "fetching", "processing", "saving", "completed", "error"
  progress: number; // 0.0 to 1.0
  message: string;
  channel_count?: number;
  error?: string;
}

export class AsyncPlaylistStore {
  private fetchStatuses = new Map<number, PlaylistFetchStatus>();
  private listeners = new Set<(status: PlaylistFetchStatus) => void>();

  constructor() {
    // Listen for playlist fetch status updates
    listen<PlaylistFetchStatus>("playlist_fetch_status", (event) => {
      const status = event.payload;
      this.fetchStatuses.set(status.id, status);

      // Notify all listeners
      this.listeners.forEach((listener) => listener(status));
    });
  }

  // Add a new playlist with async fetching
  async addPlaylistAsync(name: string, source: string): Promise<number> {
    try {
      const id = await invoke<number>("validate_and_add_channel_list_async", {
        name,
        source,
      });
      console.log(`Started async playlist addition with ID: ${id}`);
      return id;
    } catch (error) {
      console.error("Failed to add playlist:", error);
      throw error;
    }
  }

  // Refresh an existing playlist with async fetching
  async refreshPlaylistAsync(id: number): Promise<void> {
    try {
      await invoke("refresh_channel_list_async", { id });
      console.log(`Started async playlist refresh for ID: ${id}`);
    } catch (error) {
      console.error("Failed to refresh playlist:", error);
      throw error;
    }
  }

  // Get the current fetch status for a playlist
  async getFetchStatus(id: number): Promise<PlaylistFetchStatus | null> {
    try {
      return await invoke<PlaylistFetchStatus | null>(
        "get_playlist_fetch_status",
        { id },
      );
    } catch (error) {
      console.error("Failed to get fetch status:", error);
      return null;
    }
  }

  // Get all active fetch statuses
  async getAllFetchStatuses(): Promise<PlaylistFetchStatus[]> {
    try {
      return await invoke<PlaylistFetchStatus[]>(
        "get_all_playlist_fetch_status",
      );
    } catch (error) {
      console.error("Failed to get all fetch statuses:", error);
      return [];
    }
  }

  // Get cached status (from event updates)
  getCachedStatus(id: number): PlaylistFetchStatus | undefined {
    return this.fetchStatuses.get(id);
  }

  // Subscribe to status updates
  onStatusUpdate(listener: (status: PlaylistFetchStatus) => void): () => void {
    this.listeners.add(listener);

    // Return unsubscribe function
    return () => {
      this.listeners.delete(listener);
    };
  }

  // Check if a playlist is currently being fetched
  isPlaylistFetching(id: number): boolean {
    const status = this.fetchStatuses.get(id);
    return (
      status?.status === "fetching" ||
      status?.status === "processing" ||
      status?.status === "saving"
    );
  }

  // Get progress percentage for display
  getProgressPercentage(id: number): number {
    const status = this.fetchStatuses.get(id);
    return status ? Math.round(status.progress * 100) : 0;
  }

  // Usage example methods:

  // Example: Add playlist with progress monitoring
  async addPlaylistWithProgress(
    name: string,
    source: string,
    onProgress: (status: PlaylistFetchStatus) => void,
  ): Promise<number> {
    const id = await this.addPlaylistAsync(name, source);

    // Subscribe to updates for this specific playlist
    const unsubscribe = this.onStatusUpdate((status) => {
      if (status.id === id) {
        onProgress(status);

        // Unsubscribe when completed or errored
        if (status.status === "completed" || status.status === "error") {
          unsubscribe();
        }
      }
    });

    return id;
  }

  // Example: Refresh with promise that resolves when complete
  async refreshPlaylistAndWait(id: number): Promise<PlaylistFetchStatus> {
    await this.refreshPlaylistAsync(id);

    return new Promise((resolve, reject) => {
      const unsubscribe = this.onStatusUpdate((status) => {
        if (status.id === id) {
          if (status.status === "completed") {
            unsubscribe();
            resolve(status);
          } else if (status.status === "error") {
            unsubscribe();
            reject(new Error(status.error || "Unknown error"));
          }
        }
      });
    });
  }
}

// Create and export a singleton instance
export const asyncPlaylistStore = new AsyncPlaylistStore();
