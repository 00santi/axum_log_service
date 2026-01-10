use std::{
    fmt::{ Display, Formatter },
    sync::Arc,
    path::{ Path, PathBuf },
};
use axum::{
    body::Bytes,
    extract::State,
};
use serde::{
    Deserialize,
    Serialize
};
use tokio::{
    time::Duration,
    sync::Mutex,
};
use chrono::{
    DateTime,
    Utc
};


pub async fn health() -> String {
    String::from("ok!")
}

pub struct AppState {
    pub events: Mutex<Vec<Event>>,
    filepath: PathBuf,
    interval: Duration,
}

impl AppState {
    pub fn interval(&self) -> Duration {
        self.interval
    }
    pub fn filepath(&self) -> &Path {
        &self.filepath
    }
}

impl AppState {
    pub fn new(filename: &str, interval: u64) -> AppState {
        let filepath = PathBuf::from(filename);
        let interval = Duration::from_millis(interval);
        AppState {
            events: Mutex::new(Vec::new()),
            filepath,
            interval,
        }
    }
}

pub async fn event(storage: State<Arc<AppState>>, req: Bytes) -> String {
    if req.len() > 400 {
        return String::from("expected body length < 400");
    }

    let input: InputEvent = match serde_json::from_slice(&req.to_vec()) {
        Ok(input) => input,
        _ => return String::from("expected JSON body"),
    };

    let event = Event::from(input);
    storage.events.lock().await.push(event);
    String::from("added body to logs")
}

#[derive(Deserialize, Serialize)]
struct InputEvent {
    source: String,
    level: Option<String>,
    body: Option<String>,
}


pub struct Event {
    timestamp: DateTime<Utc>,
    source: String,
    level: Level,
    body: String,
}

impl Event {
    fn from(input_event: InputEvent) -> Event {
        let level = match input_event.level {
            None => Level::INVALID,
            Some(l) => {
                match l.to_lowercase().trim() {
                    "info" => Level::Info,
                    "warn" => Level::Warn,
                    "error" => Level::Error,
                    "debug" => Level::Debug,
                    _ => Level::INVALID
                }
            }
        };

        Event {
            timestamp: Utc::now(),
            source: input_event.source,
            level,
            body: input_event.body.unwrap_or(String::from("...")),
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = format!("{}: {}  |  source={}  |  time={}", self.level, self.body, self.source, self.timestamp);
        write!(f, "{}", s)
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Info => write!(f, "Info"),
            Level::Warn => write!(f, "Warn"),
            Level::Error => write!(f, "Error"),
            Level::Debug => write!(f, "Debug"),
            Level::INVALID => write!(f, "InvalidLevel"),
        }
    }
}

enum Level {
    Info, Warn, Error, Debug, INVALID,
}
