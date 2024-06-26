use crossterm::event::KeyCode;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Instant,
};
use strsim::levenshtein;

use ratatui::widgets::ListState;

use crate::{
    appdata::db::{
        add_favorite, add_saved_filter, delete_favorite, get_favorites, get_saved_filters,
    },
    m3u::fetch_channels::fetch_channels,
};

#[allow(dead_code)]
pub enum Mode {
    Normal,
    Search,
    UrlEdit,
    Playing,
}

#[derive(Clone, Debug)]
pub struct Channel {
    pub name: String,
    pub id: String,
    pub logo: String,
    pub favorite: bool,
    pub group: String,
    pub url: String,
}

pub struct ChannelList {
    pub state: ListState,
    pub items: Vec<Channel>,
}

#[allow(dead_code)]
impl ChannelList {
    pub fn current(&mut self) -> Option<&Channel> {
        let selected_channel = if let Some(i) = self.state.selected() {
            self.items.get(i)
        } else {
            None
        };
        selected_channel
    }
    pub fn first(&mut self) {
        self.state.select(Some(0))
    }
    pub fn last(&mut self) {
        self.state.select(Some(self.items.len() - 1))
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn unselect(&mut self) {
        self.state.select(None);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub player: String,
    pub args: Vec<String>,
    pub m3u_url: String,
}

pub struct MpvPlayer {
    pub pid: Option<u32>,
    pub running: bool,
    pub channel: Option<String>,
    pub started: bool,
}

pub struct LastKeyPress {
    pub code: KeyCode,
    pub time: Instant,
}

#[derive(Clone, Debug)]
pub struct SavedFilter {
    pub key: u32,
    pub filter: String,
}

pub struct App {
    pub mode: Mode,
    pub running: bool,
    pub mpv_started: bool,
    pub all_channels: Vec<Channel>,
    pub filter: String,
    pub channel_state: ChannelList,
    pub favorites: ChannelList,
    pub settings: Settings,
    pub notification: Option<String>,
    pub mpv_player: Arc<Mutex<MpvPlayer>>,
    pub last_key_press: Option<LastKeyPress>,
    pub db: Result<Connection, rusqlite::Error>,
    pub show_favorites: bool,
    pub saved_filters: Vec<SavedFilter>,
}

#[allow(dead_code)]
impl App {
    pub fn new(settings: Settings, db: Result<Connection, rusqlite::Error>) -> App {
        App {
            mode: Mode::Normal,
            running: true,
            mpv_started: false,
            all_channels: Vec::new(),
            filter: String::new(),
            channel_state: ChannelList {
                state: ListState::default(),
                items: vec![],
            },
            favorites: ChannelList {
                state: ListState::default(),
                items: vec![],
            },
            settings,
            notification: None,
            mpv_player: Arc::new(Mutex::new(MpvPlayer {
                pid: None,
                running: false,
                channel: None,
                started: false,
            })),
            last_key_press: None,
            db,
            show_favorites: false,
            saved_filters: vec![],
        }
    }
    pub fn tick(&self) {}
    pub fn quit(&mut self) {
        self.running = false;
    }
    pub fn get_channels(&mut self, always_reload: bool, never_reload: bool) {
        if let Ok(fetched_channels) = fetch_channels(
            &self.settings.m3u_url,
            always_reload,
            never_reload,
            &self.favorites.items,
        ) {
            self.all_channels = fetched_channels
        };
        self.channel_state.items = self.all_channels.clone();
        if self.all_channels.first().is_some() {
            self.channel_state.first()
        }
    }

    pub fn get_favorites(&mut self) {
        if let Ok(favs) = get_favorites(&self.db) {
            self.favorites.items = favs
        }
    }

    pub fn get_saved_filters(&mut self) {
        if let Ok(saved_filters) = get_saved_filters(&self.db) {
            self.saved_filters = saved_filters
        }
    }

    pub fn toggle_favorite(&mut self) {
        if let Some(i) = self.channel_state.state.selected() {
            if let Some(item) = self.channel_state.items.get_mut(i) {
                item.favorite = !item.favorite;
                for channel in self.all_channels.iter_mut() {
                    if channel.url == item.url {
                        channel.favorite = !channel.favorite
                    }
                }
                if item.favorite {
                    add_favorite(&self.db, item);
                    self.get_favorites();
                } else {
                    delete_favorite(&self.db, item);
                    self.favorites.items.retain(|fav| fav.url != item.url);
                    self.handle_search();
                }
            }
        }
    }
    pub fn ctrl_w(&mut self) {
        if self.filter.is_empty() {
            return;
        }
        let mut words = self.filter.split_whitespace().collect::<Vec<&str>>();
        words.pop();
        self.filter = words.join(" ");
    }
    pub fn new_saved_filter(&mut self, key: u32) {
        let saved_filter: SavedFilter = SavedFilter {
            key,
            filter: self.filter.clone(),
        };
        add_saved_filter(&self.db, saved_filter);
        self.get_saved_filters();
    }

    pub fn use_saved_filter(&mut self, key: u32) {
        if let Some(filter) = self.saved_filters.iter().find(|ss| ss.key == key) {
            self.filter = filter.filter.clone();
            self.handle_search();
            self.sort_currently_filtered_channels();
        }
    }
    pub fn handle_search(&mut self) {
        let matcher = SkimMatcherV2::default();
        let channels = if self.show_favorites {
            self.favorites.items.clone()
        } else {
            self.all_channels.clone()
        };
        if self.filter.is_empty() {
            self.channel_state.items = channels;
        } else {
            let filter_low = self.filter.clone().to_lowercase();
            let mut result: Vec<Channel> = channels
                .iter()
                .filter(|channel| {
                    let score = matcher.fuzzy_match(&channel.name.to_lowercase(), &filter_low);
                    score.unwrap_or(0) > 50
                })
                .cloned()
                .collect();

            result.sort_by(|a, b| {
                let distance_a = levenshtein(&a.name, &filter_low);
                let distance_b = levenshtein(&b.name, &filter_low);
                distance_a.cmp(&distance_b)
            });
            self.channel_state.items = result;
        }
        self.channel_state.first();
    }
    pub fn add_notification(&mut self, notification: String) {
        if let Some(old) = &self.notification {
            self.notification = Some(format!(
                "{}

{}",
                old, notification
            ))
        } else {
            self.notification = Some(notification)
        }
    }
    pub fn clear_state(&mut self) {
        self.notification = None;
        self.mpv_started = false;
        let mut mpv: MutexGuard<'_, MpvPlayer> = self.mpv_player.lock().unwrap();
        mpv.channel = None;
        mpv.pid = None;
        // drop(mpv);
    }
    pub fn clear_filter(&mut self) {
        self.filter = String::new();
    }

    pub fn sort_currently_filtered_channels(&mut self) {
        // Sort the items in channel_state based on the channel name
        if self.show_favorites {
            self.favorites.items.sort_by(|a, b| a.name.cmp(&b.name));
        } else {
            self.channel_state.items.sort_by(|a, b| a.name.cmp(&b.name));
        }
    }
}
