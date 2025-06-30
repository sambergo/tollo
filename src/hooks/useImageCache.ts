import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';

interface ImageCacheHook {
  getCachedImageUrl: (originalUrl: string) => Promise<string>;
  clearCache: () => Promise<void>;
  getCacheSize: () => Promise<number>;
}

export const useImageCache = (): ImageCacheHook => {
  const getCachedImageUrl = async (originalUrl: string): Promise<string> => {
    try {
      if (!originalUrl || originalUrl.trim() === '') {
        return '';
      }
      
      // Only cache remote URLs (http/https)
      if (!originalUrl.startsWith('http')) {
        return originalUrl;
      }
      
      const cachedPath = await invoke<string>('get_cached_image', { url: originalUrl });
      // Convert the file path to a URL that the webview can display
      return convertFileSrc(cachedPath);
    } catch (error) {
      console.warn('Failed to get cached image, using original URL:', error);
      return originalUrl;
    }
  };

  const clearCache = async (): Promise<void> => {
    try {
      await invoke('clear_image_cache');
    } catch (error) {
      console.error('Failed to clear image cache:', error);
      throw error;
    }
  };

  const getCacheSize = async (): Promise<number> => {
    try {
      const size = await invoke<number>('get_image_cache_size');
      return size;
    } catch (error) {
      console.error('Failed to get cache size:', error);
      return 0;
    }
  };

  return {
    getCachedImageUrl,
    clearCache,
    getCacheSize,
  };
};

// Helper hook for caching a single image URL
export const useCachedImage = (originalUrl: string) => {
  const [cachedUrl, setCachedUrl] = useState<string>(originalUrl);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const { getCachedImageUrl } = useImageCache();

  useEffect(() => {
    if (!originalUrl) {
      setCachedUrl('');
      return;
    }

    const loadCachedImage = async () => {
      setLoading(true);
      setError(null);
      
      try {
        const cached = await getCachedImageUrl(originalUrl);
        setCachedUrl(cached);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load image');
        setCachedUrl(originalUrl); // Fallback to original URL
      } finally {
        setLoading(false);
      }
    };

    loadCachedImage();
  }, [originalUrl, getCachedImageUrl]);

  return { cachedUrl, loading, error };
}; 