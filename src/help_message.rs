#[allow(dead_code)]
pub fn print_help_message() {
    println!(
        r#"
Usage:
tollo [url]
Settings: ~/.config/tollo/tollo.toml

Options:
-h, --help               Display this help message
[url]                    playlist for this run (this option will use the specified URL for a single run, doesn't change the default URL.)
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
| `gg`                    | Select the first channel  |
| `G`                     | Select the last channel   |
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
