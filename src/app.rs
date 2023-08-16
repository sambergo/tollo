use crossterm::event::KeyCode;
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use ratatui::widgets::ListState;

use crate::m3u::fetch_channels::fetch_channels;

#[allow(dead_code)]
pub enum Mode {
    Normal,
    Search,
    UrlEdit,
    Playing,
}

#[derive(Clone)]
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

pub struct App {
    pub mode: Mode,
    pub running: bool,
    pub mpv_started: bool,
    pub all_channels: Vec<Channel>,
    pub filter: String,
    pub channel_state: ChannelList,
    pub favorites: Vec<String>,
    pub settings: Settings,
    pub notification: Option<String>,
    pub mpv_player: Arc<Mutex<MpvPlayer>>,
    pub last_key_press: Option<LastKeyPress>,
}

#[allow(dead_code)]
impl App {
    pub fn new(settings: Settings) -> App {
        App {
            mode: Mode::Search,
            running: true,
            mpv_started: false,
            all_channels: Vec::new(),
            filter: String::new(),
            channel_state: ChannelList {
                state: ListState::default(),
                items: vec![],
            },
            favorites: Vec::new(),
            settings,
            notification: None,
            mpv_player: Arc::new(Mutex::new(MpvPlayer {
                pid: None,
                running: false,
                channel: None,
                started: false,
            })),
            last_key_press: None,
        }
    }
    pub fn tick(&self) {}
    pub fn quit(&mut self) {
        self.running = false;
    }
    pub fn get_channels(&mut self, always_update: bool) {
        if let Ok(fetched_channels) = fetch_channels(&self.settings.m3u_url, always_update) {
            self.all_channels = fetched_channels
        };
        self.channel_state.items = self.all_channels.clone();
        if self.all_channels.first().is_some() {
            self.channel_state.first()
        }
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
    }
    pub fn clear_filter(&mut self) {
        self.filter = String::new();
    }
}
