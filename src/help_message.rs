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
        "#
    );
}
