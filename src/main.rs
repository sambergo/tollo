use app::Channel;
use appdata::config::init_settings;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use m3u::play_channel::play_channel;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{env, error::Error, io, iter::Iterator, sync::Arc};
use strsim::levenshtein;
mod app;
mod appdata;
mod components;
mod m3u;
mod ui;
use crate::app::{App, Mode};
use crate::ui::render;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut settings = init_settings();
    let mut always_update: bool = false;
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1].starts_with("http://") || args[1].starts_with("https://")) {
        let m3u_url = args[1].clone();
        settings.m3u_url = m3u_url;
        always_update = true;
    };
    // create app and run it
    let mut app = App::new(settings);
    app.get_channels(always_update);
    run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) {
    while app.running {
        match terminal.draw(|frame| render(app, frame)) {
            Ok(it) => it,
            Err(_) => return,
        };
        if let Event::Key(key) = match event::read() {
            Ok(it) => it,
            Err(_) => return,
        } {
            match app.mode {
                Mode::Playing => match key.code {
                    KeyCode::Esc | KeyCode::Enter => app.mode = Mode::Normal,
                    KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.running = false
                    }
                    _ => {}
                },
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        app.running = false;
                    }
                    KeyCode::Char('s') | KeyCode::Char('/') | KeyCode::Char('i') => {
                        app.mode = Mode::Search
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.channel_state.next(),
                    KeyCode::Up | KeyCode::Char('k') => app.channel_state.previous(),
                    KeyCode::Esc => app.clear_state(),
                    KeyCode::Enter => {
                        if app.notification.is_some() {
                            app.clear_state()
                        } else {
                            let mpv_player_ref = Arc::clone(&app.mpv_player);
                            play_channel(app, mpv_player_ref)
                        }
                    }
                    KeyCode::Char('l') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.clear_filter();
                        handle_search(app);
                    }
                    KeyCode::Char('g') => app.channel_state.first(),
                    KeyCode::Char('G') => app.channel_state.last(),
                    _ => {}
                },
                Mode::Search => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.mode = Mode::Normal
                    }
                    KeyCode::Esc => app.mode = Mode::Normal,
                    KeyCode::Down => app.channel_state.next(),
                    KeyCode::Up => app.channel_state.previous(),
                    KeyCode::Char('l') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.clear_filter();
                        handle_search(app);
                    }
                    KeyCode::Char('j') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.channel_state.next()
                    }
                    KeyCode::Char('k') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.channel_state.previous()
                    }
                    KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.running = false
                    }
                    KeyCode::Backspace => {
                        app.filter.pop();
                        handle_search(app)
                    }
                    KeyCode::Char(value) => {
                        app.filter.push(value);
                        handle_search(app)
                    }
                    KeyCode::Enter => {
                        if app.notification.is_some() {
                            app.clear_state()
                        } else {
                            let mpv_player_ref = Arc::clone(&app.mpv_player);
                            play_channel(app, mpv_player_ref)
                        }
                    }
                    _ => {}
                },
                Mode::UrlEdit => todo!(),
            }
        }
    }
}

pub fn handle_search(app: &mut App) {
    let matcher = SkimMatcherV2::default();
    if app.filter.is_empty() {
        app.channel_state.items = app.all_channels.clone();
    } else {
        let mut result: Vec<Channel> = app
            .all_channels
            .iter()
            .filter(|channel| {
                let score =
                    matcher.fuzzy_match(&channel.name.to_lowercase(), &app.filter.to_lowercase());
                score.unwrap_or(0) > 50
            })
            .cloned()
            .collect();

        result.sort_by(|a, b| {
            let distance_a = levenshtein(&a.name, &app.filter.to_lowercase());
            let distance_b = levenshtein(&b.name, &app.filter.to_lowercase());
            distance_a.cmp(&distance_b)
        });
        app.channel_state.items = result;
    }
    app.channel_state.select_first();
}
