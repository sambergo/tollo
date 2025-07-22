import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { convertFileSrc } from "@tauri-apps/api/core";

interface ImageCacheHook {
  getCachedImageUrl: (originalUrl: string) => Promise<string>;
  clearCache: () => Promise<void>;
  getCacheSize: () => Promise<number>;
  getCachedImageUrlAsync: (originalUrl: string) => Promise<string>;
  clearCacheAsync: () => Promise<void>;
  getCacheSizeAsync: () => Promise<number>;
  getDownloadStatus: (url: string) => Promise<string>;
  preloadImages: (urls: string[]) => Promise<string[]>;
}

export const useImageCache = (): ImageCacheHook => {
  const getCachedImageUrl = async (originalUrl: string): Promise<string> => {
    try {
      if (!originalUrl || originalUrl.trim() === "") {
        return "";
      }

      // Only cache remote URLs (http/https)
      if (!originalUrl.startsWith("http")) {
        return originalUrl;
      }

      const cachedPath = await invoke<string>("get_cached_image", {
        url: originalUrl,
      });
      // Convert the file path to a URL that the webview can display
      return convertFileSrc(cachedPath);
    } catch (error) {
      console.warn("Failed to get cached image, using original URL:", error);
      return originalUrl;
    }
  };

  const clearCache = async (): Promise<void> => {
    try {
      await invoke("clear_image_cache");
    } catch (error) {
      console.error("Failed to clear image cache:", error);
      throw error;
    }
  };

  const getCacheSize = async (): Promise<number> => {
    try {
      const size = await invoke<number>("get_image_cache_size");
      return size;
    } catch (error) {
      console.error("Failed to get cache size:", error);
      return 0;
    }
  };

  const getCachedImageUrlAsync = async (
    originalUrl: string,
  ): Promise<string> => {
    try {
      if (!originalUrl || originalUrl.trim() === "") {
        return "";
      }

      // Only cache remote URLs (http/https)
      if (!originalUrl.startsWith("http")) {
        return originalUrl;
      }

      const cachedPath = await invoke<string>("get_cached_image_async", {
        url: originalUrl,
      });
      // Convert the file path to a URL that the webview can display
      return convertFileSrc(cachedPath);
    } catch (error) {
      console.warn(
        "Failed to get cached image async, using original URL:",
        error,
      );
      return originalUrl;
    }
  };

  const clearCacheAsync = async (): Promise<void> => {
    try {
      await invoke("clear_image_cache_async");
    } catch (error) {
      console.error("Failed to clear image cache async:", error);
      throw error;
    }
  };

  const getCacheSizeAsync = async (): Promise<number> => {
    try {
      const size = await invoke<number>("get_image_cache_size_async");
      return size;
    } catch (error) {
      console.error("Failed to get cache size async:", error);
      return 0;
    }
  };

  const getDownloadStatus = async (url: string): Promise<string> => {
    try {
      const status = await invoke<string>("get_image_download_status", { url });
      return status;
    } catch (error) {
      console.error("Failed to get download status:", error);
      return "not_cached";
    }
  };

  const preloadImages = async (urls: string[]): Promise<string[]> => {
    try {
      const results = await invoke<string[]>("preload_images", { urls });
      return results.map((path) => convertFileSrc(path));
    } catch (error) {
      console.error("Failed to preload images:", error);
      return urls; // Return original URLs as fallback
    }
  };

  return {
    getCachedImageUrl,
    clearCache,
    getCacheSize,
    getCachedImageUrlAsync,
    clearCacheAsync,
    getCacheSizeAsync,
    getDownloadStatus,
    preloadImages,
  };
};

// Enhanced hook for caching a single image URL with async support
export const useCachedImageAsync = (
  originalUrl: string,
  shouldLoad: boolean = true,
) => {
  const [cachedUrl, setCachedUrl] = useState<string>(originalUrl);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [hasLoaded, setHasLoaded] = useState<boolean>(false);
  const { getCachedImageUrlAsync } = useImageCache();

  useEffect(() => {
    if (!originalUrl || !shouldLoad || hasLoaded) {
      if (!originalUrl) {
        setCachedUrl("");
      }
      return;
    }

    const loadCachedImage = async () => {
      setLoading(true);
      setError(null);

      try {
        const cached = await getCachedImageUrlAsync(originalUrl);
        setCachedUrl(cached);
        setHasLoaded(true);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load image");
        setCachedUrl(originalUrl); // Fallback to original URL
        setHasLoaded(true);
      } finally {
        setLoading(false);
      }
    };

    loadCachedImage();
  }, [originalUrl, shouldLoad, hasLoaded, getCachedImageUrlAsync]);

  return { cachedUrl, loading, error, hasLoaded };
};

// EXISTING HOOK (for backward compatibility)
export const useCachedImage = (
  originalUrl: string,
  shouldLoad: boolean = true,
) => {
  const [cachedUrl, setCachedUrl] = useState<string>(originalUrl);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [hasLoaded, setHasLoaded] = useState<boolean>(false);
  const { getCachedImageUrl } = useImageCache();

  useEffect(() => {
    if (!originalUrl || !shouldLoad || hasLoaded) {
      if (!originalUrl) {
        setCachedUrl("");
      }
      return;
    }

    const loadCachedImage = async () => {
      setLoading(true);
      setError(null);

      try {
        const cached = await getCachedImageUrl(originalUrl);
        setCachedUrl(cached);
        setHasLoaded(true);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load image");
        setCachedUrl(originalUrl); // Fallback to original URL
        setHasLoaded(true);
      } finally {
        setLoading(false);
      }
    };

    loadCachedImage();
  }, [originalUrl, shouldLoad, hasLoaded, getCachedImageUrl]);

  return { cachedUrl, loading, error, hasLoaded };
};
