use super::chat;
use super::errors::*;
use super::events;
use super::universe;

use regex;
use std::collections::HashMap;
use std::sync::mpsc;

mod message;
pub use self::message::{Message, ThreatAssetment};

pub struct Intel {
    locations: HashMap<String, universe::System>,
    events: mpsc::Sender<events::Event>,
}

lazy_static! {
    static ref LOCATION_MESSAGE: regex::Regex =
        regex::Regex::new(r"^Channel changed to Local : (.*)").expect("must compile");
}

impl Intel {
    pub fn new(events: mpsc::Sender<events::Event>) -> Intel {
        Intel {
            locations: HashMap::new(),
            events: events,
        }
    }

    pub fn intel_message(&self, message: chat::Message) -> Result<()> {
        if let Some(location) = self.locations.get(&message.listener) {
            if let Some(intel) = message::Message::new(message, &location) {
                self.events.send(events::Event::IntelReport(intel))?;
            }
        }
        Ok(())
    }

    pub fn location_message(&mut self, message: chat::Message) -> Result<()> {
        if let Some(tokens) = LOCATION_MESSAGE.captures(&message.message) {
            let name = tokens.get(1).ok_or("should match")?.as_str();
            let system = universe::System::find(name).ok_or("no such system")?;

            info!("{} is in {}", message.listener, system.name);
            self.locations.insert(message.listener.clone(), system);
        }
        Ok(())
    }
}
