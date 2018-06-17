use super::errors::*;
use super::intel;
use super::universe;

mod audio_notification;
pub mod debounced_message;
mod desktop_notification;
mod tts;

use std::sync::mpsc;

pub struct Notifications {
    debouncer: mpsc::Sender<debounced_message::DebounceMessages>,
}

impl Notifications {
    pub fn new() -> Notifications {
        Notifications {
            debouncer: debounced_message::DebounceMessages::channel(),
        }
    }

    pub fn deliver(&self, message: intel::Message) -> Result<()> {
        self.debouncer
            .send(debounced_message::DebounceMessages::Intel(message))?;
        Ok(())
    }
}

pub enum Notification {
    None,
    Sound(String),
    Desktop(String),
}

impl From<intel::Message> for Notification {
    fn from(message: intel::Message) -> Notification {
        match message.threat_assement {
            intel::ThreatAssetment::NoThreat(system) => {
                Notification::Sound(format!("{} is clear", human_system(&system)))
            }
            intel::ThreatAssetment::ProximityAlertCritical(_) => {
                Notification::Sound(format!("{} threat in local DOCKDOCKDOCK", message.player))
            }
            intel::ThreatAssetment::ProximityAlertHigh(jumps) => Notification::Sound(format!(
                "Threat {} jumps away from {} in {}",
                jumps,
                message.player,
                human_system(&message.origin)
            )),
            intel::ThreatAssetment::ProximityAlertLow(jumps) => {
                let text = format!(
                    "Threat {} jumps away from {} in {}",
                    jumps, message.player, message.origin.name
                );
                warn!("{}", text);
                Notification::Desktop(text)
            }
            intel::ThreatAssetment::ProximityIrelevant(_jumps) => Notification::None,
            intel::ThreatAssetment::StatusRequest(system) => {
                let text = format!("status request in {}", system.name);
                warn!("{}", text);
                Notification::None
            }
            intel::ThreatAssetment::Unknown => {
                error!("Unable to asses threat level for {}", message.message);
                Notification::None
            }
        }
    }
}

fn human_system(system: &universe::System) -> String {
    let location = if system.name.find("-") == Some(2) {
        system.name[0..4].to_string()
    } else {
        system.name[0..3].to_string()
    };
    return location.replace("-", " tac ");
}
