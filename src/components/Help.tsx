import { HelpIcon } from "./Icons";

export default function Help() {
  return (
    <div className="help-layout">
      <div className="help-grid">
        <div className="help-left-column">
          <div className="settings-card">
            <div className="card-header">
              <HelpIcon />
              <h3>General</h3>
            </div>
            <div className="card-content">
              <div className="keybinding">
                <div className="keys">
                  <span className="key">Tab</span>
                  <span className="key">Ctrl+j</span>
                </div>
                <span className="description">Next tab</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">Ctrl+k</span>
                </div>
                <span className="description">Previous tab</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">j</span>
                  <span className="key">↓</span>
                </div>
                <span className="description">Navigate down in lists</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">k</span>
                  <span className="key">↑</span>
                </div>
                <span className="description">Navigate up in lists</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">g</span>
                  <span className="key">Home</span>
                </div>
                <span className="description">
                  Go to first item in current page
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">G</span>
                  <span className="key">End</span>
                </div>
                <span className="description">
                  Go to last item in current page
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">Ctrl+u</span>
                  <span className="key">PageUp</span>
                </div>
                <span className="description">
                  Page up (scroll up by 10 items)
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">Ctrl+d</span>
                  <span className="key">PageDown</span>
                </div>
                <span className="description">
                  Page down (scroll down by 10 items)
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">H</span>
                </div>
                <span className="description">Previous page</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">L</span>
                </div>
                <span className="description">Next page</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">/</span>
                  <span className="key">i</span>
                </div>
                <span className="description">Focus search input</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">Escape</span>
                </div>
                <span className="description">
                  Unfocus search field (if focused) or clear search
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">d</span>
                </div>
                <span className="description">Clear search</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">D</span>
                </div>
                <span className="description">
                  Clear all filters (search + group)
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">c</span>
                </div>
                <span className="description">
                  Clear search and focus search input
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">Ctrl+r</span>
                </div>
                <span className="description">
                  Refresh current channel list
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">F</span>
                </div>
                <span className="description">
                  Toggle favorite (in channels tab)
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">h</span>
                  <span className="key">←</span>
                  <span className="key">Backspace</span>
                </div>
                <span className="description">Clear selected channel</span>
              </div>
            </div>
          </div>
        </div>

        <div className="help-right-column">
          <div className="settings-card">
            <div className="card-header">
              <HelpIcon />
              <h3>Video Playback</h3>
            </div>
            <div className="card-content">
              <div className="keybinding">
                <div className="keys">
                  <span className="key">l</span>
                  <span className="key">→</span>
                  <span className="key">Space</span>
                </div>
                <span className="description">Play/pause video preview</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">Enter</span>
                  <span className="key">o</span>
                </div>
                <span className="description">Play in external player</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">m</span>
                </div>
                <span className="description">Toggle mute</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">f</span>
                </div>
                <span className="description">Toggle fullscreen</span>
              </div>
            </div>
          </div>

          <div className="settings-card">
            <div className="card-header">
              <HelpIcon />
              <h3>Saved Filters</h3>
            </div>
            <div className="card-content">
              <div className="keybinding">
                <div className="keys">
                  <span className="key">Alt+0-9</span>
                </div>
                <span className="description">
                  Save current search + group filter to slot (0-9)
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">0-9</span>
                </div>
                <span className="description">
                  Apply saved filter from slot (0-9)
                </span>
              </div>
            </div>
          </div>

          <div className="settings-card">
            <div className="card-header">
              <HelpIcon />
              <h3>Group Management</h3>
            </div>
            <div className="card-content">
              <div className="keybinding">
                <div className="keys">
                  <span className="key">A</span>
                </div>
                <span className="description">Select all groups</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">U</span>
                </div>
                <span className="description">Unselect all groups</span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">t</span>
                </div>
                <span className="description">
                  Toggle group display mode (enabled/all)
                </span>
              </div>
              <div className="keybinding">
                <div className="keys">
                  <span className="key">Space</span>
                </div>
                <span className="description">
                  Toggle current group selection (in groups tab) / Play/pause
                  video preview (in channels tab)
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
