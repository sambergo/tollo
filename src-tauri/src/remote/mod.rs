mod config;
mod player;

use crate::{m3u_parser::Channel, state::DbState};
use axum::{
    extract::{Extension, Query, Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tauri::{Emitter, Manager};
use tokio::sync::Mutex;

#[derive(Clone)]
struct Access {
    token: String,
    active: Arc<AtomicBool>,
}

#[derive(Clone)]
struct Service {
    app: tauri::AppHandle,
    access: Access,
    player: Arc<Mutex<player::Player>>,
    snapshot: player::Snapshot,
}

struct Session {
    service: Service,
    task: tokio::task::JoinHandle<()>,
}

struct Runtime {
    config: config::Config,
    path: PathBuf,
    session: Option<Session>,
    error: Option<String>,
    player: Arc<Mutex<player::Player>>,
    snapshot: player::Snapshot,
}

pub struct RemoteState(Mutex<Runtime>);

#[derive(Clone, Serialize)]
pub struct RemoteInfo {
    supported: bool,
    enabled: bool,
    running: bool,
    addresses: Vec<RemoteAddress>,
    token: String,
    port: u16,
    error: Option<String>,
}

impl RemoteState {
    pub fn new(path: PathBuf) -> Self {
        let (config, error) = match config::load(&path) {
            Ok(config) => (config, None),
            Err(error) => (config::Config::default(), Some(error)),
        };
        let player = player::Player::default();
        let snapshot = player.snapshot.clone();
        Self(Mutex::new(Runtime {
            config,
            path,
            session: None,
            error,
            player: Arc::new(Mutex::new(player)),
            snapshot,
        }))
    }

    pub async fn restore(&self, app: tauri::AppHandle) {
        let mut runtime = self.0.lock().await;
        if runtime.config.enabled {
            if let Err(error) = runtime.start(app).await {
                runtime.error = Some(error);
            }
        }
    }

    pub async fn shutdown(&self) {
        let mut runtime = self.0.lock().await;
        runtime.stop_server().await;
        runtime.player.lock().await.close().await;
    }
}

impl Runtime {
    fn info(&self) -> RemoteInfo {
        let running = self.session.as_ref().is_some_and(|s| {
            s.service.access.active.load(Ordering::SeqCst) && !s.task.is_finished()
        });
        RemoteInfo {
            supported: cfg!(target_os = "linux"),
            enabled: self.config.enabled,
            running,
            addresses: if running {
                addresses(self.config.port)
            } else {
                Vec::new()
            },
            token: if self.config.enabled {
                self.config.token.clone()
            } else {
                String::new()
            },
            port: self.config.port,
            error: self.error.clone(),
        }
    }

    async fn stop_server(&mut self) {
        if let Some(session) = self.session.take() {
            session.service.access.active.store(false, Ordering::SeqCst);
            session.task.abort();
            let _ = session.task.await;
        }
    }

    async fn start(&mut self, app: tauri::AppHandle) -> Result<(), String> {
        if !cfg!(target_os = "linux") {
            return Err("Browser Remote hosting is currently supported on Linux only.".into());
        }
        let listener =
            tokio::net::TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, self.config.port))
                .await
                .map_err(|e| {
                    format!(
                        "Could not open remote port {}: {e}. Choose another port and retry.",
                        self.config.port
                    )
                })?;
        let service = Service {
            app,
            access: Access {
                token: self.config.token.clone(),
                active: Arc::new(AtomicBool::new(true)),
            },
            player: self.player.clone(),
            snapshot: self.snapshot.clone(),
        };
        let router = router(service.clone());
        let active = service.access.active.clone();
        let task = tokio::spawn(async move {
            if let Err(error) = axum::serve(listener, router).await {
                eprintln!("Browser Remote server stopped: {error}");
            }
            active.store(false, Ordering::SeqCst);
        });
        self.session = Some(Session { service, task });
        self.error = None;
        Ok(())
    }
}

#[derive(Clone, Serialize)]
pub struct RemoteAddress {
    url: String,
    interface_name: Option<String>,
    local_network: bool,
}

fn interface_addresses(
    interfaces: &Value,
    port: u16,
    physical: impl Fn(&str) -> bool,
) -> Vec<RemoteAddress> {
    let mut addresses = Vec::new();
    if let Some(interfaces) = interfaces.as_array() {
        for interface in interfaces {
            let name = interface["ifname"].as_str().unwrap_or("");
            if let Some(items) = interface["addr_info"].as_array() {
                for address in items {
                    if let Some(ip) = address["local"]
                        .as_str()
                        .and_then(|ip| ip.parse::<std::net::Ipv4Addr>().ok())
                    {
                        if !ip.is_loopback() && !ip.is_unspecified() {
                            addresses.push(RemoteAddress {
                                url: format!("http://{ip}:{port}"),
                                interface_name: Some(name.to_owned()),
                                local_network: physical(name),
                            });
                        }
                    }
                }
            }
        }
    }
    addresses
}

fn addresses(port: u16) -> Vec<RemoteAddress> {
    let mut addresses = Vec::new();
    #[cfg(target_os = "linux")]
    if let Ok(output) = std::process::Command::new("ip")
        .args(["-j", "-4", "address", "show", "up"])
        .output()
    {
        if let Ok(interfaces) = serde_json::from_slice::<Value>(&output.stdout) {
            // Hardware interfaces are the usual Wi-Fi/Ethernet choices. Keep
            // bridges, containers and tunnels available under Other addresses.
            addresses = interface_addresses(&interfaces, port, |name| {
                std::path::Path::new("/sys/class/net")
                    .join(name)
                    .join("device")
                    .exists()
            });
        }
    }
    if addresses.is_empty() {
        if let Ok(address) = std::net::UdpSocket::bind("0.0.0.0:0").and_then(|socket| {
            socket.connect("192.0.2.1:80")?;
            socket.local_addr()
        }) {
            addresses.push(RemoteAddress {
                url: format!("http://{}:{port}", address.ip()),
                interface_name: None,
                local_network: false,
            });
        }
    }
    addresses.sort_by(|a, b| a.url.cmp(&b.url));
    addresses.dedup_by(|a, b| a.url == b.url);
    addresses.push(RemoteAddress {
        url: format!("http://127.0.0.1:{port}"),
        interface_name: Some("lo".into()),
        local_network: false,
    });
    addresses
}

#[tauri::command]
pub async fn get_remote_info(state: tauri::State<'_, RemoteState>) -> Result<RemoteInfo, String> {
    Ok(state.0.lock().await.info())
}

#[tauri::command]
pub async fn get_remote_playback(
    state: tauri::State<'_, RemoteState>,
) -> Result<player::Playback, String> {
    Ok(state
        .0
        .lock()
        .await
        .snapshot
        .lock()
        .map_err(|e| e.to_string())?
        .clone())
}

#[tauri::command]
pub async fn set_remote_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, RemoteState>,
    enabled: bool,
    port: u16,
) -> Result<RemoteInfo, String> {
    if enabled && !cfg!(target_os = "linux") {
        return Err("Browser Remote hosting is currently supported on Linux only.".into());
    }
    if port == 0 {
        return Err("Choose a port between 1 and 65535.".into());
    }
    let mut runtime = state.0.lock().await;
    let mut next = runtime.config.clone();
    next.enabled = enabled;
    next.port = port;
    if !enabled {
        next.token = config::new_token();
    }
    config::save(&runtime.path, &next)?;
    runtime.stop_server().await;
    runtime.config = next;
    runtime.error = None;
    if enabled {
        if let Err(error) = runtime.start(app.clone()).await {
            runtime.error = Some(error);
        }
    } else {
        runtime.player.lock().await.close().await;
    }
    let info = runtime.info();
    let _ = app.emit("remote-config", &info);
    Ok(info)
}

#[tauri::command]
pub async fn revoke_remote_access(
    app: tauri::AppHandle,
    state: tauri::State<'_, RemoteState>,
) -> Result<RemoteInfo, String> {
    let mut runtime = state.0.lock().await;
    let mut next = runtime.config.clone();
    next.token = config::new_token();
    config::save(&runtime.path, &next)?;
    runtime.stop_server().await;
    runtime.config = next;
    if runtime.config.enabled {
        if let Err(error) = runtime.start(app.clone()).await {
            runtime.error = Some(error);
        }
    }
    let info = runtime.info();
    let _ = app.emit("remote-config", &info);
    Ok(info)
}

pub async fn try_desktop_play(
    app: &tauri::AppHandle,
    channel: Channel,
) -> Option<Result<(), String>> {
    let state = app.state::<RemoteState>();
    let service =
        {
            let runtime = state.0.lock().await;
            if !runtime.config.enabled || !cfg!(target_os = "linux") {
                return None;
            }
            match runtime.session.as_ref() {
                Some(session) if session.service.access.active.load(Ordering::SeqCst) => {
                    session.service.clone()
                }
                _ => return Some(Err(
                    "Browser Remote is enabled but unavailable. Retry or disable it in Settings."
                        .into(),
                )),
            }
        };
    let _operation = match crate::operation_gate::read_async().await {
        Ok(guard) => guard,
        Err(error) => return Some(Err(error)),
    };
    Some(play_selected(&service, channel).await)
}

async fn play_selected(service: &Service, channel: Channel) -> Result<(), String> {
    let mut player = service.player.lock().await;
    if !service.access.active.load(Ordering::SeqCst) {
        return Err("Remote access was disabled or revoked.".into());
    }
    // Tell the desktop to detach its preview before mpv opens the same stream.
    let _ = service.app.emit("remote-preview-suspend", true);
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    let pending = player
        .begin_play(channel.name.clone(), channel.url.clone())
        .await;
    drop(player);
    let result = match pending {
        Ok(pending) => pending.wait().await,
        Err(error) => Err(error),
    };
    let _ = service
        .app
        .emit("remote-playback", service.snapshot.lock().unwrap().clone());
    if matches!(result, Ok(true)) {
        let state = service.app.state::<DbState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute("INSERT OR REPLACE INTO history (name,logo,url,group_title,tvg_id,resolution,extra_info,timestamp) VALUES (?1,?2,?3,?4,?5,?6,?7,CURRENT_TIMESTAMP)",
            rusqlite::params![channel.name,channel.logo,channel.url,channel.group_title,channel.tvg_id,channel.resolution,channel.extra_info]).map_err(|e| e.to_string())?;
        let _ = service.app.emit("remote-library-changed", ());
    }
    result.map(|_| ())
}

fn router(service: Service) -> Router {
    let api = Router::new()
        .route("/library", get(library))
        .route("/status", get(status))
        .route("/control", post(control))
        .route("/favorite", post(favorite))
        .route_layer(middleware::from_fn_with_state(
            service.access.clone(),
            authorize,
        ));
    Router::new()
        .route("/", get(|| async { Html(include_str!("web/index.html")) }))
        .route(
            "/remote.js",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
                    include_str!("web/remote.js"),
                )
            }),
        )
        .route(
            "/remote.css",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    include_str!("web/remote.css"),
                )
            }),
        )
        .nest("/api", api)
        .layer(middleware::from_fn(security_headers))
        .with_state(service)
}

async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    for (name, value) in [("cache-control", "no-store"), ("x-content-type-options", "nosniff"), ("referrer-policy", "no-referrer"),
        ("content-security-policy", "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'")] {
        response.headers_mut().insert(axum::http::HeaderName::from_static(name), value.parse().unwrap());
    }
    response
}

async fn authorize(State(access): State<Access>, mut request: Request, next: Next) -> Response {
    if !access.active.load(Ordering::SeqCst) {
        return failure(
            StatusCode::SERVICE_UNAVAILABLE,
            "Remote access was disabled or revoked.",
        );
    }
    let expected = format!("Bearer {}", access.token);
    if request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        != Some(expected.as_str())
    {
        return failure(
            StatusCode::UNAUTHORIZED,
            "Enter the access key shown in Tollo Settings.",
        );
    }
    if let Some(origin) = request.headers().get(header::ORIGIN) {
        let host = request
            .headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        if origin.to_str().ok() != Some(format!("http://{host}").as_str()) {
            return failure(StatusCode::FORBIDDEN, "Cross-origin access is not allowed.");
        }
    }
    request.extensions_mut().insert(access);
    next.run(request).await
}

fn failure(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({"error": message}))).into_response()
}

#[derive(Default, Deserialize)]
struct LibraryQuery {
    list_id: Option<i32>,
    view: Option<String>,
    search: Option<String>,
    group: Option<String>,
    offset: Option<usize>,
}
#[derive(Serialize)]
struct RemoteChannel {
    id: String,
    name: String,
    group: String,
    resolution: String,
}
fn channel_id(channel: &Channel) -> String {
    let mut hash = Sha256::new();
    hash.update(channel.name.as_bytes());
    hash.update([0]);
    hash.update(channel.url.as_bytes());
    format!("{:x}", hash.finalize())
}
impl From<&Channel> for RemoteChannel {
    fn from(channel: &Channel) -> Self {
        Self {
            id: channel_id(channel),
            name: channel.name.clone(),
            group: channel.group_title.clone(),
            resolution: channel.resolution.clone(),
        }
    }
}

fn collection_inner(
    app: &tauri::AppHandle,
    list_id: Option<i32>,
) -> Result<(Vec<Channel>, Vec<Channel>, Vec<Channel>), String> {
    let channels = crate::channels::get_cached_channels(app.state(), app.state(), list_id)?;
    let favorites = crate::favorites::get_favorites_inner(app.state())?;
    let history = crate::history::get_history(app.state())?;
    Ok((channels, favorites, history))
}
fn select_channel(
    items: (Vec<Channel>, Vec<Channel>, Vec<Channel>),
    id: &str,
) -> Result<Channel, String> {
    items
        .0
        .into_iter()
        .chain(items.1)
        .chain(items.2)
        .find(|channel| channel_id(channel) == id)
        .ok_or_else(|| "Channel is no longer in the library. Refresh and try again.".into())
}

fn page_channels(
    channels: &[Channel],
    favorites: &[Channel],
    history: &[Channel],
    query: &LibraryQuery,
) -> (
    Vec<RemoteChannel>,
    usize,
    std::collections::BTreeSet<String>,
) {
    let groups = channels
        .iter()
        .chain(favorites)
        .chain(history)
        .map(|channel| channel.group_title.as_str())
        .filter(|group| !group.is_empty())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(str::to_owned)
        .collect();
    let source = match query.view.as_deref() {
        Some("favorites") => favorites,
        Some("history") => history,
        _ => channels,
    };
    let search = query.search.as_deref().unwrap_or("").to_lowercase();
    let group = query.group.as_deref().unwrap_or("");
    let matches = source
        .iter()
        .filter(|channel| {
            (group.is_empty() || channel.group_title == group)
                && (search.is_empty()
                    || format!("{} {}", channel.name, channel.group_title)
                        .to_lowercase()
                        .contains(&search))
        })
        .collect::<Vec<_>>();
    let page = matches
        .iter()
        .skip(query.offset.unwrap_or(0))
        .take(150)
        .map(|channel| RemoteChannel::from(*channel))
        .collect();
    (page, matches.len(), groups)
}

async fn library(State(service): State<Service>, Query(query): Query<LibraryQuery>) -> Response {
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<Value, String> {
        let _guard = crate::operation_gate::read()?;
        let lists = crate::playlists::get_channel_lists_inner(service.app.state())?;
        let (channels, favorites, history) = collection_inner(&service.app, query.list_id)?;
        let (page, total, groups) = page_channels(&channels, &favorites, &history, &query);
        let public = |items: &[Channel]| items.iter().map(RemoteChannel::from).collect::<Vec<_>>();
        Ok(json!({"lists": lists.iter().map(|list| json!({"id":list.id,"name":list.name,"is_default":list.is_default})).collect::<Vec<_>>(),
            "channels": page, "favorites": public(&favorites), "history": public(&history), "groups":groups, "total":total}))
    }).await;
    match result {
        Ok(Ok(value)) => Json(value).into_response(),
        _ => failure(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not load the library. Check Tollo on the TV computer.",
        ),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Control {
    action: String,
    channel_id: Option<String>,
    list_id: Option<i32>,
    volume: Option<u8>,
}

async fn control(State(service): State<Service>, Json(command): Json<Control>) -> Response {
    if command.action == "play" {
        let _guard = match crate::operation_gate::read_async().await {
            Ok(guard) => guard,
            Err(_) => {
                return failure(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "The library is temporarily unavailable.",
                )
            }
        };
        let app = service.app.clone();
        let id = command.channel_id.unwrap_or_default();
        let selected = tauri::async_runtime::spawn_blocking(move || {
            select_channel(collection_inner(&app, command.list_id)?, &id)
        })
        .await;
        return match selected {
            Ok(Ok(channel)) => match play_selected(&service, channel).await {
                Ok(()) => Json(json!({"ok":true})).into_response(),
                Err(error) => failure(StatusCode::BAD_GATEWAY, &error),
            },
            _ => failure(
                StatusCode::NOT_FOUND,
                "Channel is no longer in the library. Refresh and try again.",
            ),
        };
    }
    let mut player = service.player.lock().await;
    if !service.access.active.load(Ordering::SeqCst) {
        return failure(
            StatusCode::SERVICE_UNAVAILABLE,
            "Remote access was disabled or revoked.",
        );
    }
    match player.control(&command.action, command.volume).await {
        Ok(()) => {
            let _ = service
                .app
                .emit("remote-playback", service.snapshot.lock().unwrap().clone());
            Json(json!({"ok":true})).into_response()
        }
        Err(error) => failure(StatusCode::BAD_GATEWAY, &error),
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Favorite {
    channel_id: String,
    list_id: Option<i32>,
    enabled: bool,
}
async fn favorite(
    State(service): State<Service>,
    Extension(access): Extension<Access>,
    Json(command): Json<Favorite>,
) -> Response {
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        let _guard = crate::operation_gate::read()?;
        let channel = select_channel(
            collection_inner(&service.app, command.list_id)?,
            &command.channel_id,
        )?;
        if !access.active.load(Ordering::SeqCst) {
            return Err("Remote access was disabled or revoked.".into());
        }
        crate::favorites::set_favorite_inner(service.app.state(), channel, command.enabled)?;
        let _ = service.app.emit("remote-library-changed", ());
        Ok(())
    })
    .await;
    match result {
        Ok(Ok(())) => Json(json!({"ok":true})).into_response(),
        Ok(Err(error)) => failure(StatusCode::BAD_REQUEST, &error),
        Err(_) => failure(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not update favourites.",
        ),
    }
}
async fn status(State(service): State<Service>) -> Response {
    Json(service.snapshot.lock().unwrap().clone()).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn address_grouping_uses_interface_type_instead_of_ip_prefix() {
        let interfaces = json!([
            {"ifname":"enp1s0", "addr_info":[{"local":"172.16.0.5"}]},
            {"ifname":"wlan0", "addr_info":[{"local":"10.0.0.5"}]},
            {"ifname":"tun0", "addr_info":[{"local":"192.168.5.10"}]},
            {"ifname":"docker0", "addr_info":[{"local":"172.17.0.1"}]},
            {"ifname":"lo", "addr_info":[{"local":"127.0.0.1"}]}
        ]);
        let addresses =
            interface_addresses(&interfaces, 8790, |name| matches!(name, "enp1s0" | "wlan0"));
        assert_eq!(addresses.len(), 4);
        assert_eq!(addresses.iter().filter(|a| a.local_network).count(), 2);
        assert!(addresses[0].url.starts_with("http://172.16."));
        assert_eq!(addresses[0].interface_name.as_deref(), Some("enp1s0"));
        assert!(!addresses[2].local_network);
        assert!(!addresses[3].local_network);
    }

    fn sample() -> Channel {
        Channel {
            name: "News".into(),
            url: "https://secret:password@example.test/live".into(),
            logo: "https://secret/logo".into(),
            group_title: "TV".into(),
            tvg_id: String::new(),
            resolution: String::new(),
            extra_info: String::new(),
        }
    }
    #[test]
    fn browser_metadata_does_not_expose_stream_credentials() {
        let metadata = serde_json::to_string(&RemoteChannel::from(&sample())).unwrap();
        assert!(!metadata.contains("secret"));
        assert!(!metadata.contains("password"));
        assert!(select_channel((vec![sample()], vec![], vec![]), "https://attacker.test").is_err());
        assert_eq!(
            select_channel((vec![sample()], vec![], vec![]), &channel_id(&sample())).unwrap(),
            sample()
        );
        assert!(serde_json::from_value::<Control>(
            json!({"action":"play","url":"https://attacker.test"})
        )
        .is_err());
    }

    #[test]
    fn library_pages_search_groups_and_favorites_without_large_responses() {
        let channels = (0..10000)
            .map(|number| {
                let mut channel = sample();
                channel.name = format!("Channel {number:05}");
                channel.group_title = if number % 2 == 0 { "News" } else { "Sports" }.into();
                channel
            })
            .collect::<Vec<_>>();
        let query = LibraryQuery {
            offset: Some(150),
            ..LibraryQuery::default()
        };
        let (page, total, groups) = page_channels(&channels, &[], &[], &query);
        assert_eq!(total, 10000);
        assert_eq!(page.len(), 150);
        assert_eq!(page[0].name, "Channel 00150");
        assert_eq!(groups.len(), 2);
        let query = LibraryQuery {
            search: Some("CHANNEL 000".into()),
            group: Some("News".into()),
            ..LibraryQuery::default()
        };
        let (page, total, _) = page_channels(&channels, &[], &[], &query);
        assert_eq!(total, 50);
        assert_eq!(page[0].name, "Channel 00000");
        let favorites = vec![channels[9].clone(), channels[2].clone()];
        let query = LibraryQuery {
            view: Some("favorites".into()),
            ..LibraryQuery::default()
        };
        let (page, total, _) = page_channels(&channels, &favorites, &[], &query);
        assert_eq!(total, 2);
        assert_eq!(page[0].name, "Channel 00009");
    }

    #[tokio::test]
    async fn authentication_origin_and_revocation_are_enforced() {
        let access = Access {
            token: "test-key".into(),
            active: Arc::new(AtomicBool::new(true)),
        };
        let app = Router::new()
            .route("/", get(|| async { "ok" }))
            .route_layer(middleware::from_fn_with_state(access.clone(), authorize));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = reqwest::Client::new();
        let url = format!("http://{address}/");
        assert_eq!(
            client.get(&url).send().await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            client
                .get(&url)
                .bearer_auth("wrong")
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            client
                .get(&url)
                .bearer_auth("test-key")
                .header("Origin", "https://other.test")
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            client
                .get(&url)
                .bearer_auth("test-key")
                .header("Origin", format!("http://{address}"))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        access.active.store(false, Ordering::SeqCst);
        assert_eq!(
            client
                .get(&url)
                .bearer_auth("test-key")
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        task.abort();
    }
}
