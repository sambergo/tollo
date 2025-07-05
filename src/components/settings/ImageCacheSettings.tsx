import { useState, useEffect } from "react";
import { useImageCache } from "../../hooks/useImageCache";
import { useSettingsStore } from "../../stores";
import { formatBytes } from "../../utils/format";
import { ImageIcon, ClockIcon } from "./SettingsIcons";

export function ImageCacheSettings() {
  const [imageCacheSize, setImageCacheSize] = useState<number>(0);
  const { clearCache, getCacheSize } = useImageCache();

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
    const size = await getCacheSize();
    setImageCacheSize(size);
  }

  const handleClearImageCache = async () => {
    try {
      await clearCache();
      await fetchImageCacheSize(); // Refresh cache size
      alert("Image cache cleared successfully!");
    } catch (error) {
      alert("Failed to clear image cache: " + error);
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
              <span className="stat-value">{formatBytes(imageCacheSize)}</span>
            </div>
          </div>
          <div className="cache-actions">
            <button className="btn-secondary" onClick={fetchImageCacheSize}>
              Refresh Size
            </button>
            <button className="btn-danger" onClick={handleClearImageCache}>
              Clear Cache
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
