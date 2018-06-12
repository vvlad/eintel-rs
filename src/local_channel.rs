use std::sync::mpsc;
use {ChannelInfo, Event, PlayerLocation};

extern crate regex;
use self::regex::Regex;

use System;

pub struct LocalChannel {
    regex: Regex,
    events: mpsc::Sender<Event>,
}

impl LocalChannel {
    pub fn new(events: mpsc::Sender<Event>) -> LocalChannel {
        LocalChannel {
            regex: Regex::new(r"^\[ \d{4}\.\d{2}\.\d{2} \d{2}:\d{2}:\d{2} \] EVE System > Channel changed to Local : (.*)").expect("must compile"),
            events: events
        }
    }

    pub fn process(&self, change: &mut ChannelInfo) {
        fn inner(messages: Vec<String>, regex: &Regex) -> Option<System> {
            let mut system = None;
            for message in messages {
                if let Some(matches) = regex.captures(&message) {
                    system = System::find(matches.get(1)?.as_str());
                }
            }
            system
        }

        if let Some(system) = inner(change.messages(), &self.regex) {
            self.events
                .send(Event::LocationChanged({
                    PlayerLocation {
                        player: change.player(),
                        system: system,
                    }
                }))
                .is_ok();
        }
    }
}
