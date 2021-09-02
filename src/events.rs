use chrono::UTC;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub slug: String,
    #[serde(alias = "type", rename(serialize = "type"))]
    pub type_s: String,
    pub metadata: serde_json::Value,
    pub user_id: String,
    pub invocation_id: String,
    pub datetime: String,
}

impl Event {
    pub fn new(
        slug: &str,
        type_s: &str,
        metadata: serde_json::Value,
        user_id: &str,
        invocation_id: &str,
    ) -> Event {
        Event {
            slug: slug.to_string(),
            type_s: type_s.to_string(),
            user_id: user_id.to_string(),
            invocation_id: invocation_id.to_string(),
            metadata: metadata,
            datetime: format!("{}", UTC::now()),
        }
    }
}

pub struct EventLog {
    path: path::PathBuf,
}

impl EventLog {
    pub fn new(dir: &path::PathBuf) -> EventLog {
        EventLog {
            path: dir.join("events.log"),
        }
    }

    pub fn record_event(&self, event: &Event) {
        super::debug_print(format!("appending_event_log path={:?}", self.path));

        if !self.path.exists() {
            fs::create_dir_all(&self.path.parent().unwrap()).unwrap();
            fs::File::create(&self.path).unwrap();
        }

        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .unwrap();
        let json = serde_json::to_string(&event).unwrap();
        writeln!(file, "{}", json).unwrap();
    }

    pub fn get_events(&self) -> Vec<Event> {
        let mut events = Vec::new();
        if self.path.exists() {
            let file = fs::File::open(&self.path).unwrap();
            // file.lines(
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                let event = serde_json::from_str(&line);
                if event.is_ok() {
                    events.push(event.unwrap());
                }
            }
        }
        events
    }

    pub fn clear(&self) {
        fs::remove_file(&self.path).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_standard() {
        serde_json::from_str::<Event>(
            r#"{
                "slug": "test",
                "type": "test",
                "metadata": {},
                "user_id": "test",
                "invocation_id": "test",
                "datetime": "test"
            }"#,
        )
        .unwrap();
    }
    #[test]
    fn deserialize_old() {
        serde_json::from_str::<Event>(
            r#"{
                "slug": "test",
                "type_s": "test",
                "metadata": {},
                "user_id": "test",
                "invocation_id": "test",
                "datetime": "test"
            }"#,
        )
        .unwrap();
    }
    #[test]
    fn serialize() {
        let mut event = Event::new("test", "test", serde_json::Value::Null, "test", "test");
        event.datetime = "test".to_string();
        let json = serde_json::to_string(&event).unwrap();
        assert_eq!(
            json,
            r#"{"slug":"test","type":"test","metadata":null,"user_id":"test","invocation_id":"test","datetime":"test"}"#
        );
    }
}
