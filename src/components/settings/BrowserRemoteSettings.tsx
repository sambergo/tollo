import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useRemoteStore } from "../../stores/remoteStore";
import type { RemoteInfo } from "../../types/remote";
import { RemoteConnectionAddress } from "./RemoteConnectionAddress";

export function BrowserRemoteSettings() {
  const { info, setInfo } = useRemoteStore();
  const [port, setPort] = useState("8790");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    invoke<RemoteInfo>("get_remote_info")
      .then((value) => {
        setInfo(value);
        setPort(String(value.port));
      })
      .catch((err) => setError(String(err)));
  }, [setInfo]);

  async function configure(enabled: boolean) {
    const value = Number(port);
    if (!Number.isInteger(value) || value < 1 || value > 65535) {
      setError("Choose a port between 1 and 65535.");
      return;
    }
    setBusy(true);
    setError("");
    setMessage("");
    try {
      setInfo(
        await invoke<RemoteInfo>("set_remote_config", { enabled, port: value }),
      );
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function revoke() {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      setInfo(await invoke<RemoteInfo>("revoke_remote_access"));
      setMessage("Access revoked. Share the new link to reconnect browsers.");
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function copy(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      setMessage("Copied.");
    } catch {
      setError("Could not copy. Select and copy the text manually.");
    }
  }

  return (
    <section className="settings-card" aria-busy={busy}>
      <div className="card-header">
        <h3>Browser Remote</h3>
      </div>
      <div className="card-content browser-remote-settings">
        <p>
          Browse this computer’s channels and favourites from a laptop or phone,
          and play them on its TV. Keep Tollo open in the TV desktop session and
          install mpv on this computer.
        </p>
        {info && !info.supported ? (
          <p role="status">
            Browser Remote hosting is currently supported on Linux only.
            Browsers on other platforms can connect to a Linux host.
          </p>
        ) : (
          <>
            <div className="toggle-setting">
              <div className="setting-info">
                <div className="setting-label">Allow browser remote access</div>
                <div className="setting-description">
                  Off by default. Once enabled, starts whenever Tollo opens.
                </div>
              </div>
              <button
                type="button"
                className={`toggle-button ${info?.enabled ? "active" : ""}`}
                disabled={busy || !info}
                onClick={() => configure(!info?.enabled)}
                aria-label="Allow browser remote access"
                aria-pressed={info?.enabled ?? false}
              />
            </div>
            <div className="form-group">
              <label className="form-label" htmlFor="remote-port">
                Port
              </label>
              <div className="form-row">
                <input
                  id="remote-port"
                  type="number"
                  min="1"
                  max="65535"
                  className="form-input"
                  value={port}
                  onChange={(event) => setPort(event.target.value)}
                  disabled={busy}
                />
                <button
                  className="btn-primary"
                  disabled={busy || !info}
                  onClick={() => configure(info?.enabled ?? false)}
                >
                  Save / retry
                </button>
              </div>
            </div>
            <p role="status">
              {info?.running
                ? "Remote service is running."
                : info?.enabled
                  ? "Remote service is enabled but unavailable."
                  : "Remote service is disabled."}
            </p>
            {info?.enabled && (
              <>
                <p>
                  The desktop and browser share one mpv player. Your Player
                  Command is used again when remote access is disabled. Preview
                  pauses during managed playback.
                </p>
                <p>
                  Open an address below on a device on the same trusted network.
                  The private link grants control; share it only with people you
                  trust. HTTP connections are not encrypted.
                </p>
                <p className="form-help">
                  QR codes include the access key. Share them only with people
                  you want to give control.
                </p>
                {info.addresses
                  .filter((address) => address.local_network)
                  .map((address) => (
                    <RemoteConnectionAddress
                      key={address.url}
                      address={address}
                      token={info.token}
                      onCopy={copy}
                    />
                  ))}
                {info.addresses.some((address) => !address.local_network) && (
                  <details className="remote-other-addresses">
                    <summary>Other addresses</summary>
                    <p className="form-help">
                      Virtual networks, VPNs and local testing addresses. Use
                      one only if your device can reach that network. If your
                      home network uses a bridge or virtual interface, its
                      address may also appear here.
                    </p>
                    {info.addresses
                      .filter((address) => !address.local_network)
                      .map((address) => (
                        <RemoteConnectionAddress
                          key={address.url}
                          address={address}
                          token={info.token}
                          onCopy={copy}
                        />
                      ))}
                  </details>
                )}
                <div className="form-group">
                  <label className="form-label" htmlFor="remote-key">
                    Access key (for manual connection)
                  </label>
                  <div className="form-row">
                    <input
                      id="remote-key"
                      type="password"
                      readOnly
                      className="form-input"
                      value={info.token}
                    />
                    <button
                      className="btn-primary"
                      onClick={() => copy(info.token)}
                    >
                      Copy key
                    </button>
                  </div>
                </div>
                <button
                  className="btn-primary"
                  disabled={busy}
                  onClick={revoke}
                >
                  Revoke access
                </button>
                <p className="form-help">
                  Revoking disconnects previously authorised browsers. Disabling
                  also stops the TV player. These settings and access keys stay
                  on this computer and are excluded from backups.
                </p>
              </>
            )}
          </>
        )}
        {(error || info?.error) && <p role="alert">{error || info?.error}</p>}
        {message && <p role="status">{message}</p>}
      </div>
    </section>
  );
}
