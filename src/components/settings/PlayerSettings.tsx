import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
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
    fetchEnablePreview,
    muteOnStart,
    setMuteOnStart,
    saveMuteOnStart,
    fetchMuteOnStart,
    showControls,
    setShowControls,
    saveShowControls,
    fetchShowControls,
    autoplay,
    setAutoplay,
    saveAutoplay,
    fetchAutoplay,
  } = useSettingsStore();

  useEffect(() => {
    fetchPlayerCommand();
    fetchEnablePreview();
    fetchMuteOnStart();
    fetchShowControls();
    fetchAutoplay();
  }, [
    fetchPlayerCommand,
    fetchEnablePreview,
    fetchMuteOnStart,
    fetchShowControls,
    fetchAutoplay,
  ]);

  const handleSavePlayerCommand = async () => {
    await savePlayerCommand();
  };

  const handleTogglePreview = async () => {
    const newValue = !enablePreview;
    setEnablePreview(newValue);
    await invoke("set_enable_preview", { enabled: newValue });
  };

  const handleToggleMute = async () => {
    const newValue = !muteOnStart;
    setMuteOnStart(newValue);
    await saveMuteOnStart();
  };

  const handleToggleControls = async () => {
    const newValue = !showControls;
    setShowControls(newValue);
    await saveShowControls();
  };

  const handleToggleAutoplay = async () => {
    const newValue = !autoplay;
    setAutoplay(newValue);
    await saveAutoplay();
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
              placeholder={`e.g., ${playerCommand || "mpv"}`}
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
              <div className="setting-description">
                Enable or disable channel preview functionality
              </div>
            </div>
            <button
              className={`toggle-button ${enablePreview ? "active" : ""}`}
              onClick={handleTogglePreview}
              type="button"
            />
          </div>
        </div>
        <div className="form-group">
          <div className="toggle-setting">
            <div className="setting-info">
              <div className="setting-label">Mute on Start</div>
              <div className="setting-description">
                Start video preview muted
              </div>
            </div>
            <button
              className={`toggle-button ${muteOnStart ? "active" : ""}`}
              onClick={handleToggleMute}
              type="button"
            />
          </div>
        </div>
        <div className="form-group">
          <div className="toggle-setting">
            <div className="setting-info">
              <div className="setting-label">Show Controls</div>
              <div className="setting-description">
                Show video player controls
              </div>
            </div>
            <button
              className={`toggle-button ${showControls ? "active" : ""}`}
              onClick={handleToggleControls}
              type="button"
            />
          </div>
        </div>
        <div className="form-group">
          <div className="toggle-setting">
            <div className="setting-info">
              <div className="setting-label">Autoplay</div>
              <div className="setting-description">
                Start playing video automatically
              </div>
            </div>
            <button
              className={`toggle-button ${autoplay ? "active" : ""}`}
              onClick={handleToggleAutoplay}
              type="button"
            />
          </div>
        </div>
      </div>
    </div>
  );
}
