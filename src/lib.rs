use std::time::SystemTime;

// leaf, it just replies to GET / and GET /health. Of course, types are likely different, this is just for logic
pub async fn health(_req: String) -> String {
    String::from("ok!")
}

struct Event {
    timestamp: SystemTime,
    source: String, // maybe different type?
    level: Level, // good type for now
    body: String,
    metadata: Option<String>, // possibly a different type
}

// INVALID could be deleted if we dont want to save invalid logs. I propose we save everything, but im open to any options
enum Level {
    Info, Warn, Error, Debug, INVALID,
}
