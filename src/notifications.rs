use std::hash::Hash;
use std::hash::Hasher;

use std::collections::HashSet;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use {IntelMessage, ThreatAssetment};

use audio_notification::AudioNotification;
use desktop_notification::desktop_notification;
use System;

pub enum Notification {
    None,
    Sound(String),
    Desktop(String),
}

impl From<IntelMessage> for Notification {
    fn from(message: IntelMessage) -> Notification {
        match message.threat_assement {
            ThreatAssetment::NoThreat(system) => {
                Notification::Sound(format!("{} is clear", human_system(&system)))
            }
            ThreatAssetment::ProximityAlertCritical(_) => {
                Notification::Sound(format!("{} threat in local DOCKDOCKDOCK", message.player))
            }
            ThreatAssetment::ProximityAlertHigh(jumps) => Notification::Sound(format!(
                "Threat {} jumps away from {} in {}",
                jumps,
                message.player,
                human_system(&message.origin)
            )),
            ThreatAssetment::ProximityAlertLow(jumps) => {
                let text = format!(
                    "Threat {} jumps away from {} in {}",
                    jumps, message.player, message.origin.name
                );
                warn!("{}", text);
                Notification::Desktop(text)
            }
            ThreatAssetment::ProximityIrelevant(_jumps) => Notification::None,
            ThreatAssetment::StatusRequest(system) => {
                let text = format!("status request in {}", system.name);
                warn!("{}", text);
                Notification::None
            }
            ThreatAssetment::Unknown => {
                error!("Unable to asses threat level for {}", message.message);
                Notification::None
            }
        }
    }
}

struct DebouncedMessage(IntelMessage);

impl DebouncedMessage {
    fn new(message: IntelMessage) -> DebouncedMessage {
        DebouncedMessage(message)
    }
}

impl Hash for DebouncedMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.message.hash(state);
    }
}

impl PartialEq for DebouncedMessage {
    fn eq(&self, other: &DebouncedMessage) -> bool {
        self.0.message == other.0.message
    }
}

impl Eq for DebouncedMessage {}

enum DebounceMessages {
    Intel(IntelMessage),
    Tick,
}

impl DebounceMessages {
    fn channel() -> mpsc::Sender<DebounceMessages> {
        let (tx, rx) = mpsc::channel();
        let tick = tx.clone();
        let audio = AudioNotification::new();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(200));
            tick.send(DebounceMessages::Tick).is_ok();
        });

        thread::spawn(move || {
            let mut queue: HashSet<DebouncedMessage> = HashSet::new();
            // let session_connection =
            //     Rc::new(dbus::Connection::get_private(dbus::BusType::Session).unwrap());
            // let dbus = DBusClient::new(DBUS_ID, DBUS_PATH, session_connection);

            loop {
                match rx.recv() {
                    Ok(DebounceMessages::Intel(intel)) => {
                        let debounced = DebouncedMessage::new(intel);
                        let message = if let Some(existing) = queue.take(&debounced) {
                            if existing.0.route.distance > debounced.0.route.distance {
                                existing
                            } else {
                                debounced
                            }
                        } else {
                            debounced
                        };
                        queue.insert(message);
                    }
                    Ok(DebounceMessages::Tick) => {
                        for message in queue.drain() {
                            let notification = Notification::from(message.0.clone());
                            match notification {
                                Notification::Sound(text) => {
                                    audio.notify(&text);
                                }
                                Notification::Desktop(text) => {
                                    desktop_notification(&text, &message.0.message);
                                }
                                Notification::None => {}
                            };
                        }
                    }
                    Err(error) => {
                        error!("{:?}: {}", error, error);
                        return;
                    }
                };
            }
        });

        tx
    }
}

pub struct NotificationService {
    debouncer: mpsc::Sender<DebounceMessages>,
}

impl NotificationService {
    pub fn new() -> Self {
        NotificationService {
            debouncer: DebounceMessages::channel(),
        }
    }

    pub fn notify(&self, message: IntelMessage) {
        self.debouncer
            .send(DebounceMessages::Intel(message))
            .is_ok();
    }
}

fn human_system(system: &System) -> String {
    let location = if system.name.find("-") == Some(2) {
        system.name[0..4].to_string()
    } else {
        system.name[0..3].to_string()
    };
    return location.replace("-", " tac ");
}
