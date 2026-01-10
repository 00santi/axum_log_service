use std::time::SystemTime;
use axum::body::Bytes;
use serde::Deserialize;

pub async fn health(_req: String) -> String {
    String::from("ok!")
}

pub async fn event(req: Bytes) -> String {
    if req.len() > 400 {
        return String::from("expected body length < 400");
    }

    let req = req.to_vec();
    let input: InputEvent = match serde_json::from_slice(&req) {
        Ok(input) => input,
        _ => return String::from("expected JSON body"),
    };

    let event = Event::from(input);

    //storage.lock().push(event);
    String::from("added body to logs")
}

#[derive(Deserialize)]
struct InputEvent {
    source: String,
    level: Option<String>,
    body: Option<String>,
}


struct Event {
    timestamp: SystemTime,
    source: String,
    level: Level,
    body: String,
}

impl Event {
    fn from(input_event: InputEvent) -> Event {
        let timestamp = SystemTime::now();

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
            timestamp,
            source: input_event.source,
            level,
            body: input_event.body.unwrap_or_default(),
        }
    }
}

enum Level {
    Info, Warn, Error, Debug, INVALID,
}
