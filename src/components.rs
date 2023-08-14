use ratatui::{
    style::Style,
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{App, Channel, Mode};

pub fn mode_box(app: &App) -> Paragraph {
    let title = match app.mode {
        Mode::Normal => "Normal mode",
        Mode::Search => "Search mode",
        Mode::UrlEdit => "Edit mode",
        Mode::Playing => "Playing mode",
    };
    let p = match app.mode {
        Mode::Normal => {
            "
Navigate:           🠳, J & 🠱, K
Search:             /, I
Play:               Return
Toggle favorite:    F
Clear search:       Ctrl-L
Reload channels:    R
Quit:               Q
"
        }
        Mode::Search => {
            "
Type to search.

Normal mode:    ESC, Ctrl-C, JK
Play:           Return
Navigate:       🠳, Ctrl-J & 🠱, Ctrl-K
Clear:          Ctrl-L
Quit:           Ctrl-Q
"
        }
        _ => "Edit mode",
    };
    let mode_box = Paragraph::new(Text::from(p))
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    mode_box
}

pub fn notification_box(app: &App) -> Paragraph {
    let title = "Notifications (Press ESC to clear)";
    let p = app.notification.as_ref().cloned().unwrap();
    let notification_box = Paragraph::new(Text::from(p))
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    notification_box
}

pub fn selected_channel_box(channel: Option<&Channel>) -> Paragraph {
    if let Some(c) = channel {
        let fav = if c.favorite { "⭐" } else { "" };
        let text = format!(
            "{}
{}
{}
{}
{}
",
            &c.name, &c.group, &c.id, fav, &c.url
        );
        let current_channel_box = Paragraph::new(Text::styled(text, Style::default()))
            .block(
                Block::default()
                    .title("Selected channel")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true });
        current_channel_box
    } else {
        let current_channel_box = Paragraph::new(Text::styled("no channel", Style::default()))
            .block(
                Block::default()
                    .title("Selected channel")
                    .borders(Borders::ALL),
            );
        current_channel_box
    }
}

pub fn settings_box<'a>(app: &App) -> Paragraph<'a> {
    let title = "Settings";
    let p = format!(
        "Command to run:
{} {}",
        app.settings.player,
        app.settings.args.join(" ")
    );
    let settings_box = Paragraph::new(Text::styled(p, Style::default()))
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    settings_box
}
