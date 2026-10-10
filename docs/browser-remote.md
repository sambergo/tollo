# Browser Remote

Browser Remote lets you browse Tollo’s channels and favourites from another
computer or phone and control playback on the computer connected to your TV.
The controlling device only needs a browser. Video and audio play on the host;
closing the browser does not stop playback.

## Host requirements

Hosting is supported on **Linux** in this release. Windows, macOS, Android, and
other devices can connect using a browser, but Windows and macOS Tollo cannot host
this service yet.

On the TV computer:

- Install Tollo and mpv (Ubuntu: `sudo apt install mpv`).
- Add/select your channel playlists in Tollo, and let them finish downloading.
- Run Tollo in the graphical desktop session displayed on the TV. An SSH or remote
  desktop session can target a different screen or audio device.
- Select the TV’s HDMI audio output in your desktop sound settings if necessary.

The host’s library is the source of truth. You can use Tollo’s portable backup and
restore to transfer playlists and favourites from another installation before
setting up the remote.

## Enable and connect

1. On the TV computer, open **Settings → Browser Remote**.
2. Enable **Allow browser remote access**. The service is off on fresh installations.
3. Confirm the status says **Remote service is running**. The default port is
   `8790`; change it and select **Save / retry** if another program uses it.
4. Use **Copy private link** beside the host’s network address. Open that link in a
   browser on a device on the same local network.
5. Alternatively, open the displayed address (for example,
   `http://192.168.1.50:8790`) and enter the access key copied from Settings.

The `127.0.0.1` address only works on the host itself. If several network addresses
appear, choose the one for the Wi-Fi or Ethernet network shared by your devices.
Addresses can change when your router assigns a new address; Settings shows the
current addresses.

Browsers remember their access key when browser storage is available. The opt-in,
port, and access key persist across Tollo restarts. Tollo must remain open; this
feature does not install a system service or start Tollo at login.

## Browse and control

- Select a playlist, search channel names/groups, or choose a group.
- Use **Channels**, **Favourites**, or **Recent** to switch library views.
- Select a channel to play it on the TV computer. A new selection replaces the
  stream in the same mpv window.
- Use the star button to add/remove favourites. Changes are shared with the host
  desktop and other connected browsers; existing favourite ordering is retained.
- Use **Pause/Resume**, **Stop**, **Mute/Unmute**, and the volume slider to control
  mpv. The volume control does not change the television’s hardware volume.
- **Disconnect** forgets this browser’s key and leaves playback running.

While Browser Remote is enabled, desktop external-play actions and browser
controls use one managed mpv player. Tollo’s embedded preview is suspended during
managed playback to avoid opening a second provider connection. Your saved
preview preference is retained. The configured **Player Command** is used again
when Browser Remote is disabled.

Playback status distinguishes loading, playing, paused, stopped, and failed
streams. A rejected stream or closed mpv produces an error in the browser. Select
a channel to retry or restart the player; no separate `mpv` or `tollo-tv start`
command is needed. Library changes refresh automatically; **Refresh** requests
an immediate update.

## Disable or revoke access

- **Revoke access** generates a new key. Previously authorised browsers must
  reconnect with the new private link/key. Current playback continues.
- Disabling **Allow browser remote access** closes the listener, stops its player,
  invalidates old access, and restores the normal external-player behaviour.
- Remote settings and keys are local to this computer and **excluded from portable
  backups**. Importing a library does not enable network access or replace keys.

## Network and troubleshooting

Use this service on a **trusted local network**. HTTP traffic is not encrypted.
Anyone with the private link or key can control playback and change favourites.
The connection key is placed in the link fragment and sent as an authorization
header, rather than in server query strings. Stream/provider credentials stay
on the host and are not sent to the browser.

Tollo does not change firewall rules or configure router port forwarding.
If the address is unreachable, check that:

- Tollo is open and Settings reports the remote service running.
- Both devices are on the same network. Guest Wi-Fi/client isolation may block
  communication between devices.
- The host firewall allows inbound TCP connections to the selected port from your
  local network. For example, with Ubuntu UFW and a `192.168.1.0/24` home subnet:
  `sudo ufw allow from 192.168.1.0/24 to any port 8790 proto tcp`.
  Substitute your actual subnet and configured port.
- You are using a host network address rather than `127.0.0.1`.

Do not forward this port to the public internet. A private VPN or authenticated
HTTPS proxy is outside the setup covered by this release.

If a channel fails, stop other previews/players using the same provider account,
wait briefly, and retry. Providers may limit simultaneous connections, and
expired URLs or server failures can affect playback. Selecting another channel
or retrying restarts mpv if it has exited.

## Optional SSH prototype

The earlier [SSH playback prototype](remote-playback.md) remains available for
users who prefer running Tollo on a laptop and forwarding playback to an SSH
host. It is independent of Browser Remote. Browser Remote runs Tollo on the TV
computer, uses that computer’s library, and requires no SSH setup.
