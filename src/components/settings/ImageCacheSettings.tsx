import { useState, useEffect } from "react";
import { useImageCache } from "../../hooks/useImageCache";
import { formatBytes } from "../../utils/format";
import { ImageIcon } from "./SettingsIcons";

export function ImageCacheSettings() {
  const [imageCacheSize, setImageCacheSize] = useState<number>(0);
  const { clearCache, getCacheSize } = useImageCache();

  useEffect(() => {
    fetchImageCacheSize();
  }, []);

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

  return (
    <div className="settings-card">
      <div className="card-header">
        <ImageIcon />
        <h3>Image Cache</h3>
      </div>
      <div className="card-content">
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
  );
} 