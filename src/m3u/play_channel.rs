use crate::app::{App, Mode, MpvPlayer};
use std::{
    process::{Command, Stdio},
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

#[allow(dead_code)]
pub fn play_channel(app: &mut App, mpv_player_ref: Arc<Mutex<MpvPlayer>>) {
    if app.mpv_started {
        return;
    };
    app.clear_state();
    let selected_channel = if let Some(i) = app.channel_state.state.selected() {
        app.channel_state.items.get(i)
    } else {
        None
    };
    if let Some(channel) = selected_channel {
        app.mpv_started = true;
        app.mode = Mode::Playing;
        let channel_name = channel.name.clone();
        let player_command = app.settings.player.clone();
        let mut args: Vec<String> = app.settings.args.clone();
        args.push(channel.url.clone());
        // let cmd = format!(
        //     "{} {} {}",
        //     &app.settings.player,
        //     &app.settings.arguments.join(" "),
        //     &channel.url
        // );
        thread::spawn(move || {
            let mut mpv: MutexGuard<'_, MpvPlayer> = mpv_player_ref.lock().unwrap();
            mpv.running = true;
            mpv.channel = Some(channel_name);
            // change mpv.channel to channel.name
            let mut mpv_process = Command::new(player_command)
                .args(&args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("failed to execute process");
            mpv.pid = Some(mpv_process.id());
            mpv.started = true;
            drop(mpv);
            mpv_process.wait().expect("failed to wait for process");
            let mut mpv: MutexGuard<'_, MpvPlayer> = mpv_player_ref.lock().unwrap();
            mpv.running = false;
        });
    } else {
        app.add_notification("No selected channel".to_string());
    }
}
