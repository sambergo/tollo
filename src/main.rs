use app::LastKeyPress;
use appdata::{config::init_settings, db::connect_db, local::is_same_as_prev};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use help_message::print_help_message;
use m3u::play_channel::play_channel;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{env, error::Error, io, iter::Iterator, sync::Arc, time::Instant};
mod app;
mod appdata;
mod components;
mod help_message;
mod m3u;
mod ui;
use crate::{
    app::{App, Mode},
    ui::render,
};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help_message();
        std::process::exit(0);
    }
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut settings = init_settings();
    if args.len() > 1 && (args[1].starts_with("http://") || args[1].starts_with("https://")) {
        let m3u_url = args[1].clone();
        settings.m3u_url = m3u_url;
    };
    let always_update: bool = !is_same_as_prev(&settings.m3u_url);
    // create app and run it
    let db = connect_db();
    let mut app = App::new(settings, db);
    app.get_favorites();
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
                    KeyCode::Esc | KeyCode::Enter => {
                        app.mode = Mode::Normal;
                        app.clear_state()
                    }
                    KeyCode::Char('q') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.running = false
                    }
                    _ => {}
                },
                Mode::Normal => match key.code {
                    KeyCode::Char('q') => {
                        app.running = false;
                    }
                    KeyCode::Char('/') | KeyCode::Char('i') => app.mode = Mode::Search,
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
                        app.handle_search();
                    }
                    KeyCode::Char('g') => app.channel_state.first(),
                    KeyCode::Char('G') => app.channel_state.last(),
                    KeyCode::Char('F') => app.toggle_favorite(),
                    KeyCode::Char('f') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.show_favorites = !app.show_favorites;
                        app.handle_search();
                    }
                    _ => {}
                },
                Mode::Search => {
                    match key.code {
                        KeyCode::Char('c')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Esc => app.mode = Mode::Normal,
                        KeyCode::Down => app.channel_state.next(),
                        KeyCode::Up => app.channel_state.previous(),
                        KeyCode::Char('w')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.ctrl_w();
                            app.handle_search();
                        }
                        KeyCode::Char('l')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.clear_filter();
                            app.handle_search();
                        }
                        KeyCode::Char('j')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.channel_state.next();
                        }
                        KeyCode::Char('k')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.channel_state.previous();
                        }
                        KeyCode::Char('q')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.running = false;
                        }
                        KeyCode::Char('f')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            app.show_favorites = !app.show_favorites;
                            app.handle_search();
                        }
                        KeyCode::Backspace => {
                            app.filter.pop();
                            app.handle_search();
                        }
                        KeyCode::Char('k')
                            if check_last_keypress_jk(&app.last_key_press, key.code) =>
                        {
                            app.filter.pop();
                            app.handle_search();
                            app.mode = Mode::Normal;
                        }
                        KeyCode::Char(value) => {
                            app.filter.push(value);
                            app.handle_search();
                        }
                        KeyCode::Enter => {
                            if app.notification.is_some() {
                                app.clear_state();
                            } else {
                                let mpv_player_ref = Arc::clone(&app.mpv_player);
                                play_channel(app, mpv_player_ref);
                            }
                        }
                        _ => {}
                    };
                    app.last_key_press = Some(LastKeyPress {
                        code: key.code,
                        time: Instant::now(),
                    });
                }
                Mode::UrlEdit => todo!(),
            }
        }
    }
}

fn check_last_keypress_jk(last_key_press: &Option<LastKeyPress>, current_key: KeyCode) -> bool {
    if current_key == KeyCode::Char('k') {
        if let Some(last_key) = last_key_press {
            if last_key.code == KeyCode::Char('j') {
                let now = Instant::now();
                let duration_since = now.duration_since(last_key.time);
                if duration_since.as_millis() < 300 {
                    return true;
                }
            }
        }
    }
    false
}

// pub fn handle_search(app: &mut App) {
//     let matcher = SkimMatcherV2::default();
//     let channels = if app.show_favorites {
//         app.all_channels
//             .iter()
//             .filter(|channel| channel.favorite)
//             .cloned()
//             .collect()
//     } else {
//         app.all_channels.clone()
//     };
//     if app.filter.is_empty() {
//         app.channel_state.items = channels;
//     } else {
//         let mut result: Vec<Channel> = channels
//             .iter()
//             .filter(|channel| {
//                 let score =
//                     matcher.fuzzy_match(&channel.name.to_lowercase(), &app.filter.to_lowercase());
//                 score.unwrap_or(0) > 50
//             })
//             .cloned()
//             .collect();

//         result.sort_by(|a, b| {
//             let distance_a = levenshtein(&a.name, &app.filter.to_lowercase());
//             let distance_b = levenshtein(&b.name, &app.filter.to_lowercase());
//             distance_a.cmp(&distance_b)
//         });
//         app.channel_state.items = result;
//     }
//     app.channel_state.first();
// }
