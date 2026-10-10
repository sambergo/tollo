use serde::Serialize;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

#[derive(Clone, Serialize)]
pub struct Playback {
    pub channel: Option<String>,
    pub state: String,
    pub paused: bool,
    pub muted: bool,
    pub volume: f64,
    pub error: Option<String>,
}

impl Default for Playback {
    fn default() -> Self {
        Self {
            channel: None,
            state: "stopped".into(),
            paused: false,
            muted: false,
            volume: 100.0,
            error: None,
        }
    }
}

pub type Snapshot = Arc<Mutex<Playback>>;

pub struct Player {
    child: Option<tokio::process::Child>,
    directory: Option<tempfile::TempDir>,
    monitor: Option<tokio::task::JoinHandle<()>>,
    pub snapshot: Snapshot,
    generation: Arc<AtomicU64>,
    #[cfg(test)]
    pub headless: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            child: None,
            directory: None,
            monitor: None,
            snapshot: Arc::new(Mutex::new(Playback::default())),
            generation: Arc::new(AtomicU64::new(0)),
            #[cfg(test)]
            headless: false,
        }
    }
}

impl Player {
    pub async fn close(&mut self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        if let Some(task) = self.monitor.take() {
            task.abort();
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
        self.directory = None;
        *self.snapshot.lock().unwrap() = Playback::default();
    }

    async fn start(&mut self) -> Result<(), String> {
        if !cfg!(target_os = "linux") {
            return Err("Browser Remote hosting is currently supported on Linux only.".into());
        }
        if let Some(child) = self.child.as_mut() {
            if child.try_wait().map_err(|e| e.to_string())?.is_none() {
                return Ok(());
            }
        }
        self.close().await;
        let directory = tempfile::Builder::new()
            .prefix("tollo-remote-")
            .tempdir()
            .map_err(|e| e.to_string())?;
        let path = directory.path().join("mpv.sock");
        let mut command = tokio::process::Command::new("mpv");
        command.args(["--idle=yes", "--force-window=yes", "--fullscreen"]);
        #[cfg(test)]
        if self.headless {
            command.args(["--no-config", "--vo=null", "--ao=null", "--force-window=no"]);
        }
        self.child = Some(
            command
                .arg(format!("--input-ipc-server={}", path.display()))
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .kill_on_drop(true)
                .spawn()
                .map_err(|_| "Could not start mpv. Install mpv on the TV computer.".to_string())?,
        );
        self.directory = Some(directory);
        for _ in 0..100 {
            if path.exists() {
                #[cfg(target_os = "linux")]
                {
                    // Connect before loading so even an immediate failure is observed.
                    let stream = tokio::net::UnixStream::connect(&path)
                        .await
                        .map_err(|e| e.to_string())?;
                    self.monitor = Some(tokio::spawn(observe(stream, self.snapshot.clone())));
                }
                return Ok(());
            }
            if self
                .child
                .as_mut()
                .unwrap()
                .try_wait()
                .map_err(|e| e.to_string())?
                .is_some()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        self.close().await;
        Err("mpv could not start. Run Tollo in the TV desktop session.".into())
    }

    #[cfg(target_os = "linux")]
    async fn ipc(&self, command: Value, wait_for_load: bool) -> Result<Value, String> {
        use tokio::io::AsyncWriteExt;
        let directory = self
            .directory
            .as_ref()
            .ok_or("The TV player is not running.")?;
        let operation = async {
            let mut stream = tokio::net::UnixStream::connect(directory.path().join("mpv.sock"))
                .await
                .map_err(|_| {
                    "Lost connection to mpv. Select a channel to restart it.".to_string()
                })?;
            stream
                .write_all(format!("{}\n", json!({"command":command,"request_id":1})).as_bytes())
                .await
                .map_err(|e| e.to_string())?;
            read_reply(stream, wait_for_load).await
        };
        tokio::time::timeout(
            Duration::from_secs(if wait_for_load { 20 } else { 3 }),
            operation,
        )
        .await
        .map_err(|_| {
            "Stream loading timed out. Check playback status or select a channel to retry."
                .to_string()
        })?
    }

    #[cfg(not(target_os = "linux"))]
    async fn ipc(&self, _: Value, _: bool) -> Result<Value, String> {
        Err("Browser Remote hosting is currently supported on Linux only.".into())
    }

    pub async fn begin_play(&mut self, name: String, url: String) -> Result<PendingPlay, String> {
        if let Err(error) = self.start().await {
            let mut status = self.snapshot.lock().unwrap();
            status.channel = Some(name);
            status.state = "failed".into();
            status.error = Some(error.clone());
            return Err(error);
        }
        {
            let mut status = self.snapshot.lock().unwrap();
            status.channel = Some(name);
            status.state = "loading".into();
            status.error = None;
        }
        self.ipc(json!(["set_property", "pause", false]), false)
            .await?;
        let expected = self.generation.fetch_add(1, Ordering::SeqCst) + 1;
        #[cfg(target_os = "linux")]
        {
            use tokio::io::AsyncWriteExt;
            let directory = self
                .directory
                .as_ref()
                .ok_or("The TV player is not running.")?;
            let mut stream = tokio::net::UnixStream::connect(directory.path().join("mpv.sock"))
                .await
                .map_err(|_| {
                    "Lost connection to mpv. Select a channel to restart it.".to_string()
                })?;
            stream
                .write_all(
                    format!(
                        "{}\n",
                        json!({"command":["loadfile",url,"replace"],"request_id":1})
                    )
                    .as_bytes(),
                )
                .await
                .map_err(|e| e.to_string())?;
            Ok(PendingPlay {
                stream,
                expected,
                generation: self.generation.clone(),
                snapshot: self.snapshot.clone(),
            })
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (url, expected);
            Err("Browser Remote hosting is currently supported on Linux only.".into())
        }
    }

    pub async fn control(&mut self, action: &str, volume: Option<u8>) -> Result<(), String> {
        let command = match action {
            "stop" => {
                if self.child.is_none()
                    || self
                        .child
                        .as_mut()
                        .unwrap()
                        .try_wait()
                        .map_err(|e| e.to_string())?
                        .is_some()
                {
                    self.close().await;
                    return Ok(());
                }
                json!(["stop"])
            }
            "pause" => json!(["cycle", "pause"]),
            "mute" => json!(["cycle", "mute"]),
            "volume" => json!([
                "set_property",
                "volume",
                volume
                    .filter(|v| *v <= 100)
                    .ok_or("Volume must be between 0 and 100.")?
            ]),
            _ => return Err("Unknown playback action.".into()),
        };
        if action == "stop" {
            self.generation.fetch_add(1, Ordering::SeqCst);
        }
        self.ipc(command, false).await?;
        if action == "stop" {
            let mut status = self.snapshot.lock().unwrap();
            status.channel = None;
            status.state = "stopped".into();
            status.error = None;
        }
        Ok(())
    }
}

pub struct PendingPlay {
    #[cfg(target_os = "linux")]
    stream: tokio::net::UnixStream,
    expected: u64,
    generation: Arc<AtomicU64>,
    snapshot: Snapshot,
}

impl PendingPlay {
    pub async fn wait(self) -> Result<bool, String> {
        #[cfg(target_os = "linux")]
        let result = tokio::time::timeout(Duration::from_secs(20), read_reply(self.stream, true))
            .await
            .map_err(|_| {
                "Stream loading timed out. Check playback status or select a channel to retry."
                    .to_string()
            })
            .and_then(|result| result);
        #[cfg(not(target_os = "linux"))]
        let result: Result<Value, String> =
            Err("Browser Remote hosting is currently supported on Linux only.".into());
        if self.generation.load(Ordering::SeqCst) != self.expected {
            return Ok(false);
        }
        let mut status = self.snapshot.lock().unwrap();
        match result {
            Ok(_) => {
                status.state = if status.paused { "paused" } else { "playing" }.into();
                status.error = None;
                Ok(true)
            }
            Err(error) => {
                status.error = Some(error.clone());
                status.state = "failed".into();
                Err(error)
            }
        }
    }
}

#[cfg(target_os = "linux")]
async fn read_reply(stream: tokio::net::UnixStream, wait_for_load: bool) -> Result<Value, String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut lines = BufReader::new(stream).lines();
    let mut started = false;
    while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
        let reply: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        if reply["request_id"] == 1 {
            if reply["error"] != "success" {
                return Err("mpv could not execute this playback action.".into());
            }
            if !wait_for_load {
                return Ok(reply["data"].clone());
            }
        }
        if reply["event"] == "start-file" {
            started = true;
        }
        if started && reply["event"] == "file-loaded" {
            return Ok(Value::Null);
        }
        if started && reply["event"] == "end-file" && reply["reason"] != "redirect" {
            return Err(stream_error(&reply));
        }
    }
    Err("The TV player closed. Select a channel to restart it.".into())
}

fn stream_error(reply: &Value) -> String {
    // file_error is mpv's error description, not a provider URL or diagnostic log.
    format!(
        "Stream failed: {}. Retry with other players closed.",
        reply["file_error"].as_str().unwrap_or("playback ended")
    )
}

#[cfg(target_os = "linux")]
async fn observe(mut stream: tokio::net::UnixStream, snapshot: Snapshot) {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    for (id, property) in ["pause", "mute", "volume"].iter().enumerate() {
        if stream
            .write_all(
                format!("{}\n", json!({"command":["observe_property",id,property]})).as_bytes(),
            )
            .await
            .is_err()
        {
            return;
        }
    }
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let Ok(reply) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let mut status = snapshot.lock().unwrap();
        match reply["event"].as_str() {
            Some("file-loaded") => {
                status.state = if status.paused { "paused" } else { "playing" }.into();
                status.error = None;
            }
            Some("end-file") if reply["reason"] == "error" => {
                status.state = "failed".into();
                status.error = Some(stream_error(&reply));
            }
            Some("end-file") if reply["reason"] == "eof" => {
                status.state = "stopped".into();
                status.channel = None;
            }
            Some("property-change") => match reply["name"].as_str() {
                Some("pause") => {
                    status.paused = reply["data"].as_bool().unwrap_or(false);
                    if status.state == "playing" || status.state == "paused" {
                        status.state = if status.paused { "paused" } else { "playing" }.into();
                    }
                }
                Some("mute") => status.muted = reply["data"].as_bool().unwrap_or(false),
                Some("volume") => status.volume = reply["data"].as_f64().unwrap_or(100.0),
                _ => {}
            },
            _ => {}
        }
    }
    let mut status = snapshot.lock().unwrap();
    if status.channel.is_some() {
        status.state = "failed".into();
        status.error = Some("The TV player closed. Select a channel to restart it.".into());
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    #[tokio::test]
    async fn accepted_command_without_loaded_stream_times_out_with_visible_error() {
        use tokio::io::AsyncWriteExt;
        let (stream, mut peer) = tokio::net::UnixStream::pair().unwrap();
        peer.write_all(b"{\"event\":\"start-file\"}\n{\"request_id\":1,\"error\":\"success\"}\n")
            .await
            .unwrap();
        let snapshot = Arc::new(Mutex::new(Playback {
            channel: Some("Slow channel".into()),
            state: "loading".into(),
            ..Playback::default()
        }));
        let pending = PendingPlay {
            stream,
            expected: 1,
            generation: Arc::new(AtomicU64::new(1)),
            snapshot: snapshot.clone(),
        };
        assert!(pending.wait().await.unwrap_err().contains("timed out"));
        let state = snapshot.lock().unwrap();
        assert_eq!(state.state, "failed");
        assert!(state.error.as_ref().unwrap().contains("timed out"));
    }

    #[tokio::test]
    async fn player_reports_failure_and_restarts_after_exit() {
        if std::process::Command::new("mpv")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let mut player = Player {
            headless: true,
            ..Player::default()
        };
        let result = player
            .begin_play(
                "Missing".into(),
                "/nonexistent-tollo-remote-test.wav".into(),
            )
            .await
            .unwrap()
            .wait()
            .await;
        assert!(result.unwrap_err().contains("Stream failed"));
        assert!(player.snapshot.lock().unwrap().error.is_some());
        player.control("stop", None).await.unwrap();
        assert!(player.snapshot.lock().unwrap().channel.is_none());
        player.child.as_mut().unwrap().kill().await.unwrap();
        player.start().await.unwrap();
        assert!(player.child.as_mut().unwrap().try_wait().unwrap().is_none());
        player.close().await;
    }
    fn wave(path: &std::path::Path) {
        let size = 8000u32 * 2 * 30;
        let mut bytes = b"RIFF".to_vec();
        bytes.extend((size + 36).to_le_bytes());
        bytes.extend(b"WAVEfmt ");
        bytes.extend(16u32.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(1u16.to_le_bytes());
        bytes.extend(8000u32.to_le_bytes());
        bytes.extend(16000u32.to_le_bytes());
        bytes.extend(2u16.to_le_bytes());
        bytes.extend(16u16.to_le_bytes());
        bytes.extend(b"data");
        bytes.extend(size.to_le_bytes());
        bytes.resize(size as usize + 44, 0);
        std::fs::write(path, bytes).unwrap();
    }

    #[tokio::test]
    async fn switching_unpauses_and_reuses_player_and_stop_cancels_loading() {
        if std::process::Command::new("mpv")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let mut player = Player {
            headless: true,
            ..Player::default()
        };
        let first = directory.path().join("first.wav");
        wave(&first);
        assert!(player
            .begin_play("First".into(), first.to_string_lossy().into())
            .await
            .unwrap()
            .wait()
            .await
            .unwrap());
        let pid = player.child.as_ref().unwrap().id();
        player.control("pause", None).await.unwrap();
        let second = directory.path().join("second.wav");
        wave(&second);
        assert!(player
            .begin_play("Second".into(), second.to_string_lossy().into())
            .await
            .unwrap()
            .wait()
            .await
            .unwrap());
        assert_eq!(player.child.as_ref().unwrap().id(), pid);
        assert_eq!(
            player
                .ipc(json!(["get_property", "pause"]), false)
                .await
                .unwrap(),
            false
        );
        player.control("volume", Some(35)).await.unwrap();
        player.control("mute", None).await.unwrap();
        assert_eq!(
            player
                .ipc(json!(["get_property", "volume"]), false)
                .await
                .unwrap(),
            35.0
        );
        assert_eq!(
            player
                .ipc(json!(["get_property", "mute"]), false)
                .await
                .unwrap(),
            true
        );
        assert!(player.control("arbitrary-command", None).await.is_err());
        assert!(player.control("volume", Some(101)).await.is_err());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let stalled = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_secs(30)).await;
        });
        let pending = player
            .begin_play("Slow".into(), format!("http://{address}/slow.wav"))
            .await
            .unwrap();
        let load = tokio::spawn(pending.wait());
        tokio::time::sleep(Duration::from_millis(100)).await;
        tokio::time::timeout(Duration::from_secs(3), player.control("stop", None))
            .await
            .unwrap()
            .unwrap();
        assert!(!tokio::time::timeout(Duration::from_secs(3), load)
            .await
            .unwrap()
            .unwrap()
            .unwrap());
        assert!(player.snapshot.lock().unwrap().channel.is_none());
        stalled.abort();
        player.close().await;
    }
}
