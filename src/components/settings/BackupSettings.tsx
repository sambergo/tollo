import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ExportResult, ImportPreview } from "../../types/backup";

export function BackupSettings() {
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const tokenRef = useRef<string | null>(null);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      if (tokenRef.current) {
        void invoke("cancel_user_data_import", { token: tokenRef.current });
      }
    };
  }, []);

  async function run(action: () => Promise<void>) {
    setBusy(true);
    setMessage("");
    setError("");
    try {
      await action();
    } catch (err) {
      if (mounted.current) setError(String(err));
    } finally {
      if (mounted.current) setBusy(false);
    }
  }

  const exportBackup = () =>
    run(async () => {
      const path = await invoke<ExportResult>("export_user_data");
      if (path && mounted.current) setMessage(`Backup saved to ${path}`);
    });

  const importBackup = () =>
    run(async () => {
      const result = await invoke<ImportPreview | null>(
        "preview_user_data_import",
      );
      if (!mounted.current) {
        if (result) {
          await invoke("cancel_user_data_import", { token: result.token });
        }
        return;
      }
      tokenRef.current = result?.token ?? null;
      setPreview(result);
    });

  const cancelImport = () =>
    run(async () => {
      if (!preview) return;
      await invoke("cancel_user_data_import", { token: preview.token });
      tokenRef.current = null;
      setPreview(null);
    });

  const confirmImport = () =>
    run(async () => {
      if (!preview) return;
      try {
        await invoke("confirm_user_data_import", { token: preview.token });
      } catch (err) {
        tokenRef.current = null;
        setPreview(null);
        throw err;
      }
      tokenRef.current = null;
      window.location.reload();
    });

  return (
    <section className="settings-card" aria-busy={busy}>
      <div className="card-header">
        <h3>Backup &amp; restore</h3>
      </div>
      <div className="card-content backup-content">
        <p>
          Save settings, channel lists, favorites, saved filters, and group
          selections. Local playlist files are included. History and caches are
          excluded.
        </p>
        <p>
          Backups are unencrypted and include source URLs and configured
          commands. Remote playlists are fetched again after restoring.
        </p>
        <div className="backup-actions">
          <button
            className="btn-primary"
            disabled={busy || preview !== null}
            onClick={exportBackup}
          >
            Export backup
          </button>
          <button
            className="btn-primary"
            disabled={busy || preview !== null}
            onClick={importBackup}
          >
            Import backup
          </button>
        </div>
        {busy && <p role="status">Working…</p>}
        {message && <p role="status">{message}</p>}
        {error && <p role="alert">{error}</p>}
        {preview && (
          <div className="backup-preview" aria-label="Import preview">
            <h4>Review backup</h4>
            <p>
              Exported {new Date(preview.exported_at).toLocaleString()} with
              Tollo {preview.app_version}
            </p>
            <p>
              {preview.list_names.length} channel lists (
              {preview.local_playlist_count} local), {preview.favorite_count}{" "}
              favorites, {preview.filter_count} saved filters, and{" "}
              {preview.group_count} group selections.
            </p>
            {preview.list_names.length > 0 && (
              <ul>
                {preview.list_names.map((name) => (
                  <li key={name}>{name}</li>
                ))}
              </ul>
            )}
            <p>Default list: {preview.default_list ?? "None"}</p>
            <dl className="backup-settings">
              <dt>Player command</dt>
              <dd>
                <code>{preview.settings.player_command}</code>
              </dd>
              <dt>Clipboard command</dt>
              <dd>
                <code>{preview.settings.clipboard_command}</code>
              </dd>
              <dt>Cache duration</dt>
              <dd>{preview.settings.cache_duration_hours} hours</dd>
              <dt>Preview</dt>
              <dd>
                {preview.settings.enable_preview ? "Enabled" : "Disabled"}
              </dd>
              <dt>Mute on start</dt>
              <dd>{preview.settings.mute_on_start ? "Enabled" : "Disabled"}</dd>
              <dt>Show controls</dt>
              <dd>{preview.settings.show_controls ? "Enabled" : "Disabled"}</dd>
              <dt>Autoplay</dt>
              <dd>{preview.settings.autoplay ? "Enabled" : "Disabled"}</dd>
            </dl>
            <p>
              Restoring replaces your settings, channel lists, favorites, saved
              filters, and group selections. Watch history is preserved. Tollo
              reloads after restoring. Player and clipboard commands may need
              adjustment on another operating system.
            </p>
            <div className="backup-actions">
              <button
                className="btn-primary"
                disabled={busy}
                onClick={confirmImport}
              >
                Replace user data &amp; restore
              </button>
              <button
                className="btn-secondary"
                disabled={busy}
                onClick={cancelImport}
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>
    </section>
  );
}
