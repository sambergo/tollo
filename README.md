# Tollo

IPTV terminal UI with fuzzy search written in rust.

## Features

- Compatible with `m3u` playlists.
- Fuzzy search for channels by name.
- Keyboard shortcuts for easy navigation.
- Control mode-based interface for simplicity and ease of use.
- Utilizes `mpv` for playing the streams.

## Installation

Before you can run this application, make sure you have Rust installed. If you don't, you can install it from the [official website](https://www.rust-lang.org/tools/install).

Now clone the repository and run the application:

```bash
git clone https://github.com/sambergo/tollo.git
cd tollo
chmod +x install.sh
./install.sh
```

## Usage

The interface is divided into different modes - Normal, Search and Playing. Below are the controls for the app:

### Normal mode

- `q` - Quit the application
- `s` or `/` or `i` - Switch to search mode
- `j` or `Down Key` - Next Channel
- `k` or `Up Key` - Previous Channel
- `Esc` - Clear state
- `Enter` - Plays the selected channel
- `Ctrl+l` - Clear filter
- `g` - Select the first channel
- `G` - Select the last channel

### Search mode

In search mode, a user can type to filter the channels. The following commands are available:

- `Ctrl+c` or `Esc` - Exit search mode
- `Down` - Next Channel
- `Up` - Previous Channel
- `Ctrl+j` - Next Channel
- `Ctrl+k` - Previous Channel
- `Ctrl+l` - Clear filter
- `Ctrl+q` - Quit Application
- `Backspace` - Remove the last character from the filter
- `Enter` - Plays the selected channel

## Dependencies

## License

Tollo is distributed under the terms of the MIT license. See [LICENSE](LICENSE) for details.
