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
use std::{env, error::Error, io, iter::Iterator, path::Path, sync::Arc, time::Instant};
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
// use arboard::Clipboard;

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal

    let mut args: Vec<String> = env::args().collect();
    let mut always_reload: bool = false;
    let mut never_reload: bool = false;
    if let Some(index) = args.iter().position(|arg| arg == "-h" || arg == "--help") {
        print_help_message();
        args.remove(index);
        std::process::exit(0);
    } else if let Some(index) = args.iter().position(|arg| arg == "-r" || arg == "--reload") {
        always_reload = true;
        args.remove(index);
    } else if let Some(index) = args
        .iter()
        .position(|arg| arg == "-p" || arg == "--preserve")
    {
        never_reload = true;
        args.remove(index);
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
    } else if args.len() > 1 {
        let argument = &args[1];
        let path = Path::new(argument);
        if let Some(file_name) = path.file_name() {
            if let Some(extension) = file_name.to_str().and_then(|s| s.split('.').last()) {
                if extension == "m3u" {
                    settings.m3u_url = argument.to_string();
                } else {
                    println!("Invalid M3U file or path: {}", argument);
                    std::process::exit(0);
                }
            } else {
                println!("Invalid M3U file or path: {}", argument);
                std::process::exit(0);
            }
        } else {
            println!("Invalid M3U file or path: {}", argument);
            std::process::exit(0);
        }
    }
    if !always_reload {
        always_reload = !is_same_as_prev(&settings.m3u_url);
    }

    // create app and run it
    let db = connect_db();
    let mut app = App::new(settings, db);
    app.get_favorites();
    app.get_saved_filters();
    app.get_channels(always_reload, never_reload);
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
                    KeyCode::Char('d') => {
                        if check_last_keypress_interval(&app.last_key_press, key.code, 'd', 'd') {
                            app.clear_filter();
                            app.handle_search();
                        }
                    }
                    KeyCode::Char('g') => {
                        if check_last_keypress_interval(&app.last_key_press, key.code, 'g', 'g') {
                            app.channel_state.first();
                        }
                    }
                    KeyCode::Char('G') => app.channel_state.last(),
                    KeyCode::Char('F') => app.toggle_favorite(),
                    KeyCode::Char('f') => {
                        if key.modifiers.contains(event::KeyModifiers::CONTROL)
                            || check_last_keypress_interval(&app.last_key_press, key.code, 'g', 'f')
                        {
                            app.show_favorites = !app.show_favorites;
                            app.handle_search();
                        }
                        if check_last_keypress_interval(&app.last_key_press, key.code, 'c', 'f') {
                            app.toggle_favorite();
                        }
                    }
                    KeyCode::Char('r') => {
                        if key.modifiers.contains(event::KeyModifiers::CONTROL)
                            || check_last_keypress_interval(&app.last_key_press, key.code, 'g', 'r')
                        {
                            app.get_channels(true, false);
                        }
                    }
                    KeyCode::Char('S') => app.sort_currently_filtered_channels(),
                    KeyCode::Char('c') => {
                        if check_last_keypress_interval(&app.last_key_press, key.code, 'c', 'c') {
                            app.clear_filter();
                            app.handle_search();
                            app.mode = Mode::Search;
                        }
                    }
                    KeyCode::Char('1') => {
                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                            app.new_saved_filter(1);
                        } else {
                            app.use_saved_filter(1);
                        }
                    }
                    KeyCode::Char('2') => {
                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                            app.new_saved_filter(2);
                        } else {
                            app.use_saved_filter(2);
                        }
                    }
                    KeyCode::Char('3') => {
                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                            app.new_saved_filter(3);
                        } else {
                            app.use_saved_filter(3);
                        }
                    }
                    KeyCode::Char('4') => {
                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                            app.new_saved_filter(4);
                        } else {
                            app.use_saved_filter(4);
                        }
                    }
                    KeyCode::Char('5') => {
                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                            app.new_saved_filter(5);
                        } else {
                            app.use_saved_filter(5);
                        }
                    }
                    KeyCode::Char('6') => {
                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                            app.new_saved_filter(6);
                        } else {
                            app.use_saved_filter(6);
                        }
                    }
                    KeyCode::Char('7') => {
                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                            app.new_saved_filter(7);
                        } else {
                            app.use_saved_filter(7);
                        }
                    }
                    KeyCode::Char('8') => {
                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                            app.new_saved_filter(8);
                        } else {
                            app.use_saved_filter(8);
                        }
                    }
                    KeyCode::Char('9') => {
                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                            app.new_saved_filter(9);
                        } else {
                            app.use_saved_filter(9);
                        }
                    }
                    KeyCode::Char('0') => {
                        if key.modifiers.contains(event::KeyModifiers::ALT) {
                            app.new_saved_filter(0);
                        } else {
                            app.use_saved_filter(0);
                        }
                    }
                    // TODO
                    // KeyCode::Char('y') => {
                    //     if check_last_keypress_interval(&app.last_key_press, key.code, 'y', 'y') {
                    //         let mut clipboard = Clipboard::new().unwrap();
                    //         let the_string = "Hello, world!";
                    //         clipboard.set_text(the_string).unwrap();
                    //     }
                    // }
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
                            if check_last_keypress_interval(
                                &app.last_key_press,
                                key.code,
                                'j',
                                'k',
                            ) =>
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
                }
                Mode::UrlEdit => todo!(),
            }
            app.last_key_press = Some(LastKeyPress {
                code: key.code,
                time: Instant::now(),
            });
        }
    }
}

fn check_last_keypress_interval(
    last_key_press: &Option<LastKeyPress>,
    current_key: KeyCode,
    target_last: char,
    target_now: char,
) -> bool {
    if current_key == KeyCode::Char(target_now) {
        if let Some(last_key) = last_key_press {
            if last_key.code == KeyCode::Char(target_last) {
                let now = Instant::now();
                let duration_since = now.duration_since(last_key.time);
                if duration_since.as_millis() < 500 {
                    return true;
                }
            }
        }
    }
    false
}
