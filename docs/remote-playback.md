# Play on a TV computer over SSH

This prototype runs Tollo on your laptop and one fullscreen mpv process on the
TV computer. Selecting another channel replaces the stream in the same player.
The TV computer downloads the stream directly; closing the laptop does not stop
playback. Tollo does not need to run on the TV computer.

Requirements: Linux on the TV computer, Python 3.8+ on both computers, OpenSSH on
the laptop, and mpv on the TV computer. The laptop wrapper also works on macOS.
The examples use the SSH alias `hp`; substitute your own hostname if needed.

## Install

On the TV computer, install mpv using your package manager (Ubuntu:
`sudo apt install mpv`). From this repository on the laptop:

```sh
ssh hp 'mkdir -p ~/.local/bin'
scp scripts/tollo-tv hp:.local/bin/tollo-tv
ssh hp 'chmod +x ~/.local/bin/tollo-tv'
```

First connect interactively with `ssh hp` to establish trust and configure key
authentication. Confirm `ssh -o BatchMode=yes hp true` succeeds: the wrapper cannot
prompt for a password or key passphrase when launched from Tollo. A desktop-launched
Tollo must have access to the same SSH agent as your terminal.

## Start the TV player

Log into the desktop displayed on the TV, open a terminal **in that desktop**, and run:

```sh
~/.local/bin/tollo-tv start
```

Leave it running. This opens an idle fullscreen player; Escape exits fullscreen
and `q` quits. Starting from the physical desktop gives mpv the correct display
and audio session. An SSH or remote-desktop session can target a different display.
Choose the TV's HDMI audio output in Ubuntu's sound settings if necessary.

## Connect Tollo

In Tollo on the laptop, set **Settings → external player command** to the absolute
path to Python followed by the absolute script path, `play`, and the SSH alias:

```text
/usr/bin/python3 /absolute/path/to/tollo/scripts/tollo-tv play hp
```

Use `command -v python3` to find Python's path. These paths must contain no spaces:
Tollo currently splits this setting on whitespace. Then select a channel and use
**Play in external player**. The embedded preview still plays locally; disable
preview/autoplay if you only want playback on the TV.

You can also try it directly from the repository:

```sh
python3 scripts/tollo-tv play hp 'https://your-provider.example/channel.m3u8'
python3 scripts/tollo-tv pause hp
python3 scripts/tollo-tv volume hp 50
python3 scripts/tollo-tv stop hp
python3 scripts/tollo-tv status hp
```

`pause` toggles pause. `stop` leaves the player open for another channel. `status`
prints `true` when mpv is idle and `false` when a file is loaded. A successful play
command means mpv accepted the request, not that the stream has finished loading
or is playable. Stream errors appear on the TV player. Volume controls mpv's volume,
not the TV hardware volume.

## Design and limitations

- SSH uses your existing configuration and keys; no extra network port is opened.
- URLs are JSON on SSH's standard input, never interpolated into shell commands.
- mpv's socket is under `~/.cache/tollo-tv/`, restricted to the current user.
- The same account must run the TV player and receive the SSH command.
- Receiver errors propagate as a failing process exit code. Tollo currently checks
  its launched process after three seconds, so a slower SSH failure may only appear
  in terminal output. Use the direct commands above to diagnose connection failures.
- This is a laptop prototype. A phone browser remote and automatic startup are not
  included. The TV player needs an active graphical session.

## Verification

Run `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts -p 'test_*.py' -v`.
The IPC integration test needs mpv and permission to create a local Unix socket;
it disables video/audio output and uses generated local audio files.
