#![feature(mpsc_select)]
#![feature(vec_remove_item)]
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

use universe::System;

mod channel;
pub use channel::{ChannelInfo, Channels};

mod local_channel;
pub use local_channel::LocalChannel;

mod intel_channel;
pub use intel_channel::IntelChannel;

mod notifications;
pub use notifications::NotificationService;

mod audio_notification;
pub use audio_notification::AudioNotification;
mod desktop_notification;
mod tts;

pub mod universe;

#[derive(Debug, Clone)]
pub enum ThreatAssetment {
    Unknown,
    NoThreat(System),
    ProximityIrelevant(u16),
    ProximityAlertLow(u16),
    ProximityAlertHigh(u16),
    ProximityAlertCritical(u16),
    StatusRequest(System),
}

#[derive(Debug, Clone)]
pub struct IntelMessage {
    pub message: String,
    pub player: String,
    pub tokens: Vec<String>,
    pub route: universe::Route,
    pub origin: universe::System,
    pub involved_players: Vec<String>,
    pub threat_assement: ThreatAssetment,
    pub sender: String,
}

#[derive(Debug)]
pub struct PlayerLocation {
    pub player: String,
    pub system: System,
}

#[derive(Debug)]
pub enum Event {
    ChannelResumed(ChannelInfo),
    ChannelChanged(ChannelInfo),
    LocationChanged(PlayerLocation),
    IntelReport(IntelMessage),
    Unknown,
}
