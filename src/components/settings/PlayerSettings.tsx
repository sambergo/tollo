import { useEffect } from "react";
import { invoke } from '@tauri-apps/api/core';
import { useSettingsStore } from "../../stores";
import { PlayIcon } from "./SettingsIcons";

export function PlayerSettings() {
  const { 
    playerCommand, 
    setPlayerCommand, 
    savePlayerCommand, 
    fetchPlayerCommand,
    enablePreview,
    setEnablePreview,
    fetchEnablePreview
  } = useSettingsStore();

  useEffect(() => {
    fetchPlayerCommand();
    fetchEnablePreview();
  }, [fetchPlayerCommand, fetchEnablePreview]);

  const handleSavePlayerCommand = async () => {
    await savePlayerCommand();
  };

  const handleTogglePreview = async () => {
    const newValue = !enablePreview;
    setEnablePreview(newValue);
    // Save the new value directly
    await invoke("set_enable_preview", { enabled: newValue });
  };

  return (
    <div className="settings-card">
      <div className="card-header">
        <PlayIcon />
        <h3>Player Settings</h3>
      </div>
      <div className="card-content">
        <div className="form-group">
          <label className="form-label">Player Command</label>
          <div className="form-row">
            <input
              type="text"
              className="form-input"
              value={playerCommand}
              onChange={(e) => setPlayerCommand(e.target.value)}
              placeholder="e.g., mpv"
            />
            <button className="btn-primary" onClick={handleSavePlayerCommand}>
              Save
            </button>
          </div>
          <p className="form-help">Command to launch external video player</p>
        </div>
        <div className="form-group">
          <div className="toggle-setting">
            <div className="setting-info">
              <div className="setting-label">Enable Preview</div>
              <div className="setting-description">Enable or disable channel preview functionality</div>
            </div>
            <button
              className={`toggle-button ${enablePreview ? 'active' : ''}`}
              onClick={handleTogglePreview}
              type="button"
            />
          </div>
        </div>
      </div>
    </div>
  );
} 