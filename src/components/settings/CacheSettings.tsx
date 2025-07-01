import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ClockIcon } from "./SettingsIcons";

export function CacheSettings() {
  const [cacheDuration, setCacheDuration] = useState(24);

  useEffect(() => {
    fetchCacheDuration();
  }, []);

  async function fetchCacheDuration() {
    const duration = await invoke<number>("get_cache_duration");
    setCacheDuration(duration);
  }

  const handleSaveCacheDuration = async () => {
    await invoke("set_cache_duration", { hours: cacheDuration });
  };

  return (
    <div className="settings-card">
      <div className="card-header">
        <ClockIcon />
        <h3>Cache Settings</h3>
      </div>
      <div className="card-content">
        <div className="form-group">
          <label className="form-label">Cache Duration (hours)</label>
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
          <p className="form-help">How long to cache channel data before refreshing</p>
        </div>
      </div>
    </div>
  );
} 