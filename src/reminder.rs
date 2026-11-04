use chrono::{Local, Timelike};
use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::{
    io::Cursor,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

const REMINDER_WAV: &[u8] = include_bytes!("../assets/bell.wav");

const CHECK_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct ReminderConfig {
    pub enabled: bool,
    pub start_hour: u32,
    pub end_hour: u32,
}

impl Default for ReminderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            start_hour: 7,
            end_hour: 19,
        }
    }
}

enum Command {
    UpdateConfig(ReminderConfig),
    TestSound,
    Shutdown,
}

pub struct ReminderService {
    tx: Sender<Command>,
    worker: Option<JoinHandle<()>>,
}

impl ReminderService {
    pub fn start(config: ReminderConfig) -> Self {
        let (tx, rx) = mpsc::channel();

        let worker = thread::Builder::new()
            .name("hourly-reminder".into())
            .spawn(move || run_worker(rx, config))
            .expect("Failed to spawn reminder thread");

        Self {
            tx,
            worker: Some(worker),
        }
    }

    pub fn update_config(&self, config: ReminderConfig) {
        let _ = self.tx.send(Command::UpdateConfig(config));
    }

    pub fn test_sound(&self) {
        let _ = self.tx.send(Command::TestSound);
    }
}

impl Drop for ReminderService {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Shutdown);

        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn hour_allowed(hour: u32, config: ReminderConfig) -> bool {
    if config.start_hour <= config.end_hour {
        hour >= config.start_hour && hour <= config.end_hour
    } else {
        // Operating hours cross midnight.
        hour >= config.start_hour || hour <= config.end_hour
    }
}

fn play_sound(player: &Player) {
    let cursor = Cursor::new(REMINDER_WAV);

    match Decoder::try_from(cursor) {
        Ok(source) => {
            // Stop any previous reminder.
            player.stop();

            // Set volume to 1.0.
            player.set_volume(1.0);

            // Start playing the embedded WAV.
            player.append(source);
        }
        Err(err) => {
            tracing::error!("Failed to decode reminder WAV: {err}");
        }
    }
}

fn run_worker(rx: Receiver<Command>, mut config: ReminderConfig) {
    // Initialize audio inside the worker thread.
    // Keep the stream alive for the worker's lifetime.
    let audio = match DeviceSinkBuilder::open_default_sink() {
        Ok(device) => {
            let player = Player::connect_new(device.mixer());

            Some((device, player))
        }
        Err(err) => {
            tracing::error!("Audio initialization failed: {err}");
            None
        }
    };

    let mut last_triggered: Option<i64> = None;

    loop {
        // Process incoming commands, waiting at most 250 ms.
        match rx.recv_timeout(CHECK_INTERVAL) {
            Ok(Command::UpdateConfig(new_config)) => {
                config = new_config;
            }

            Ok(Command::TestSound) => {
                if let Some((_, player)) = &audio {
                    play_sound(player);
                }
            }

            Ok(Command::Shutdown) => break,

            Err(RecvTimeoutError::Disconnected) => break,

            Err(RecvTimeoutError::Timeout) => {}
        }

        let now = Local::now();

        // Unix timestamp identifying the beginning
        // of this actual hour, including DST transitions.
        let hour_id = now.timestamp() - i64::from(now.minute()) * 60 - i64::from(now.second());

        if config.enabled
            && now.minute() == 0
            && hour_allowed(now.hour(), config)
            && last_triggered != Some(hour_id)
        {
            // Mark this hour as handled.
            last_triggered = Some(hour_id);

            tracing::info!("Hourly reminder triggered at {}", now.format("%H:%M:%S"));

            if let Some((_, player)) = &audio {
                play_sound(player);
            } else {
                tracing::warn!("Reminder triggered, but audio is unavailable");
            }
        }
    }

    tracing::info!("Hourly reminder worker stopped");
}
