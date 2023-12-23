#[allow(dead_code)]
pub fn print_help_message() {
    println!(
        r#"
Usage:
tollo [url / path]
Settings: ~/.config/tollo/tollo.toml

Options:
-h, --help               Display this help message
-r, --reload             Reload playlist
-p, --preserve           Preserve old playlist
[url / path]             playlist for this run (this option will use the specified URL or filepath for a single run, doesn't change the default URL.)
Note: To set a default URL, please add it to the `~/.config/tollo/tollo.toml` config file.

| Key                     | Action                    |
| ----------------------- | ------------------------- |
| Normal mode             |                           |
| `q`                     | Quit the application      |
| `/` or `i`              | Switch to search mode     |
| `j` or `down`           | Next channel              |
| `k` or `up`             | Previous channel          |
| `Enter`                 | Play the selected channel |
| `Ctrl+l` or `dd`        | Clear filter              |
| `cc`                    | Clear filter & search     |
| `Ctrl+f` or `gf`        | Show favorites            |
| `Shift+f` or `cf`       | Add / rm favorites        |
| `Shift+r` or `gr`       | Reload playlist           |
| `Shift+s`               | Sort channels             |
| `gg`                    | Select the first channel  |
| `G`                     | Select the last channel   |
| `Alt+[1-9]`             | Save filter               |
| `[1-9]`                 | Use saved filter          |
|                         |                           |
| Search mode             |                           |
| `Enter`                 | Play the selected channel |
| `Ctrl+j` or `down`      | Next channel              |
| `Ctrl+k` or `up`        | Previous channel          |
| `Ctrl+f`                | Show favorites            |
| `Ctrl+l`                | Clear filter              |
| `Ctrl+c`, `Esc` or `jk` | Normal mode               |
| `Ctrl+q`                | Quit the application      |
| -----------------       | ------------------------- |
        "#
    );
}
