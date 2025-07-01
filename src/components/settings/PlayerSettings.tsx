import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { PlayIcon } from "./SettingsIcons";

export function PlayerSettings() {
  const [playerCommand, setPlayerCommand] = useState("");

  useEffect(() => {
    fetchPlayerCommand();
  }, []);

  async function fetchPlayerCommand() {
    const fetchedCommand = await invoke<string>("get_player_command");
    setPlayerCommand(fetchedCommand);
  }

  const handleSavePlayerCommand = async () => {
    await invoke("set_player_command", { command: playerCommand });
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