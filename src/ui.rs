use std::sync::Arc;

use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    prelude::Alignment,
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{
    app::{App, Mode},
    components::{mode_box, notification_box, selected_channel_box, settings_box},
};

/// Renders the user interface widgets.
#[allow(dead_code)]
pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui-org/ratatui/tree/master/examples
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(frame.size());

    // HEAD
    let header_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    let header = Paragraph::new(Text::styled("TOLLO", Style::default()))
        .alignment(Alignment::Center)
        .block(header_block);

    frame.render_widget(header, vertical_chunks[0]);

    match app.mode {
        Mode::Playing => {
            let (channel, pid, _running) = {
                let mpv_player_ref = Arc::clone(&app.mpv_player);
                let mpv = mpv_player_ref.lock().unwrap();
                let channel = mpv.channel.as_ref().unwrap_or(&String::from("")).clone();
                let pid = mpv.pid.unwrap_or_default();
                let running = mpv.running;
                (channel, pid, running)
            };
            // if !running {
            //     app.mode = Mode::Normal;
            // }
            let playing_text = format!(
                "
Playing:        {}
PID:                {}

Return:             Return, Esc
Quit app:           Ctrl-Q
",
                channel, pid
            );
            let playing_widget = Paragraph::new(playing_text)
                .block(Block::default().borders(Borders::ALL))
                .alignment(Alignment::Center);
            frame.render_widget(playing_widget, vertical_chunks[1])
        }
        _ => {
            // BODY - LEFT (LIST)
            let body_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(66), Constraint::Percentage(34)].as_ref())
                .split(vertical_chunks[1]);

            let mut list_items = Vec::<ListItem>::new();
            list_items.extend(
                app.channel_state
                    .items
                    .iter()
                    .map(|channel| ListItem::new(channel.name.to_string())),
            );

            let title = format!("Channels ({})", list_items.len());
            let list = List::new(list_items)
                .block(Block::default().title(title).borders(Borders::ALL))
                .style(Style::default())
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>");
            frame.render_stateful_widget(list, body_chunks[0], &mut app.channel_state.state);

            // BODY - RIGHT
            let body_right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(35),
                        Constraint::Percentage(45),
                        Constraint::Percentage(20),
                    ]
                    .as_ref(),
                )
                .split(body_chunks[1]);

            let selected_channel = if let Some(i) = app.channel_state.state.selected() {
                app.channel_state.items.get(i)
            } else {
                None
            };

            let current_channel_box = selected_channel_box(selected_channel);
            frame.render_widget(current_channel_box, body_right_chunks[0]);

            if app.notification.is_some() {
                let notification_box = notification_box(app);
                frame.render_widget(notification_box, body_right_chunks[1]);
            } else {
                let mode_box = mode_box(app);
                frame.render_widget(mode_box, body_right_chunks[1]);
            }

            let settings_box = settings_box(app);
            frame.render_widget(settings_box, body_right_chunks[2]);
        }
    }

    // FOOTER

    let m3u_url_text = app.settings.m3u_url.clone();
    let m3u_url_footer = Paragraph::new(m3u_url_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    // let search_text = Paragraph::new(app.filter.clone()).block(key_block);
    let search_footer = Paragraph::new(app.filter.clone())
        .block(Block::default().borders(Borders::ALL).fg(Color::White))
        .alignment(Alignment::Center);

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(vertical_chunks[2]);

    // TODO poista
    frame.render_widget(search_footer, footer_chunks[0]);
    frame.render_widget(m3u_url_footer, footer_chunks[1]);
}
