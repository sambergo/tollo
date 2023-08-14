# Tollo

IPTV terminal UI player with fuzzy search written in rust.

## Features

- [Built with ratatui](https://github.com/ratatui-org/ratatui)
- Compatible with `m3u` playlists.
- Fuzzy search for channels by name.
- Keyboard shortcuts for easy navigation.
- Utilizes `mpv` or player of your choice for playing the streams.

## Installation

Before you can run this application, make sure you have Rust installed. If you don't, you can install it from the [official website](https://www.rust-lang.org/tools/install).

Now clone the repository and run the application:

```bash
git clone https://github.com/sambergo/tollo.git
cd tollo
chmod +x install.sh
./install.sh
```

## Configuration

```toml
[settings]
player = "mpv"
args = ["--fullscreen", "--volume=50" ]
m3u_url = "https://iptv-org.github.io/iptv/index.m3u"

```

## TODO

- [] Favorites
- [] Playlist from path?

## Usage

```bash
# defaults ~/.config/tollo/tollo.toml url
tollo [URL]
```

### Key bindings

| Key                     | Action                                    |
| ----------------------- | ----------------------------------------- |
| Normal mode             |                                           |
| `q`                     | Quit the application                      |
| `/` or `i`              | Switch to search mode                     |
| `j` or `down`           | Next channel                              |
| `k` or `up`             | Previous channel                          |
| `Enter`                 | Plays the selected channel                |
| `Ctrl+l`                | Clear filter                              |
| `g`                     | Select the first channel                  |
| `G`                     | Select the last channel                   |
| -----------------       | ----------------------------------------- |
| Search mode             |                                           |
| `Ctrl+c`, `Esc` or `jk` | Normal mode                               |
| `Ctrl+j` or `down`      | Next channel                              |
| `Ctrl+k` or `up`        | Previous channel                          |
| `Ctrl+l`                | Clear filter                              |
| `Ctrl+q`                | Quit application                          |
| `Enter`                 | Plays the selected channel                |
| -----------------       | ----------------------------------------- |

## Dependencies

- mpv or video player of your choice

## License

[The MIT License](https://opensource.org/licenses/MIT)
