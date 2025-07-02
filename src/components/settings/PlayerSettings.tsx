import { useEffect } from "react";
import { useSettingsStore } from "../../stores";
import { PlayIcon } from "./SettingsIcons";

export function PlayerSettings() {
  const { 
    playerCommand, 
    setPlayerCommand, 
    savePlayerCommand, 
    fetchPlayerCommand 
  } = useSettingsStore();

  useEffect(() => {
    fetchPlayerCommand();
  }, [fetchPlayerCommand]);

  const handleSavePlayerCommand = async () => {
    await savePlayerCommand();
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
      </div>
    </div>
  );
} 