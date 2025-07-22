import { useState, useEffect } from "react";
import { useImageCache } from "../../hooks/useImageCache";
import { useSettingsStore } from "../../stores";
import { formatBytes } from "../../utils/format";
import { ImageIcon, ClockIcon } from "./SettingsIcons";

export function ImageCacheSettings() {
  const [imageCacheSize, setImageCacheSize] = useState<number>(0);
  const [isLoadingCacheSize, setIsLoadingCacheSize] = useState<boolean>(false);
  const [isClearingCache, setIsClearingCache] = useState<boolean>(false);
  const { clearCacheAsync, getCacheSizeAsync } = useImageCache();

  // Cache duration settings
  const {
    cacheDuration,
    setCacheDuration,
    saveCacheDuration,
    fetchCacheDuration,
  } = useSettingsStore();

  useEffect(() => {
    fetchImageCacheSize();
    fetchCacheDuration();
  }, [fetchCacheDuration]);

  async function fetchImageCacheSize() {
    setIsLoadingCacheSize(true);
    try {
      const size = await getCacheSizeAsync();
      setImageCacheSize(size);
    } catch (error) {
      console.error("Failed to fetch image cache size:", error);
    } finally {
      setIsLoadingCacheSize(false);
    }
  }

  const handleClearImageCache = async () => {
    setIsClearingCache(true);
    try {
      await clearCacheAsync();
      await fetchImageCacheSize(); // Refresh cache size
      alert("Image cache cleared successfully!");
    } catch (error) {
      alert("Failed to clear image cache: " + error);
    } finally {
      setIsClearingCache(false);
    }
  };

  const handleSaveCacheDuration = async () => {
    await saveCacheDuration();
  };

  return (
    <div className="settings-card">
      <div className="card-header">
        <ImageIcon />
        <h3>Cache Settings</h3>
      </div>
      <div className="card-content">
        {/* Cache Duration Settings */}
        <div className="form-group">
          <div className="form-label-with-icon">
            <ClockIcon />
            <label className="form-label">Cache Duration (hours)</label>
          </div>
          <div className="form-row">
            <input
              type="number"
              className="form-input"
              value={cacheDuration}
              onChange={(e) => setCacheDuration(parseInt(e.target.value))}
              min="1"
              max="168"
            />
            <button className="btn-primary" onClick={handleSaveCacheDuration}>
              Save
            </button>
          </div>
          <p className="form-help">
            How long to cache channel data before refreshing
          </p>
        </div>

        <hr className="settings-divider" />

        {/* Image Cache Management */}
        <div className="form-group">
          <div className="form-label-with-icon">
            <ImageIcon />
            <label className="form-label">Image Cache</label>
          </div>
          <div className="cache-info">
            <div className="cache-stat">
              <span className="stat-label">Cache Size:</span>
              <span className="stat-value">
                {isLoadingCacheSize
                  ? "Loading..."
                  : formatBytes(imageCacheSize)}
              </span>
            </div>
          </div>
          <div className="cache-actions">
            <button
              className="btn-secondary"
              onClick={fetchImageCacheSize}
              disabled={isLoadingCacheSize}
            >
              {isLoadingCacheSize ? "Loading..." : "Refresh Size"}
            </button>
            <button
              className="btn-danger"
              onClick={handleClearImageCache}
              disabled={isClearingCache}
            >
              {isClearingCache ? "Clearing..." : "Clear Cache"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
