extern crate eintel;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use eintel::Channels;
use eintel::{AudioNotification, Event, IntelChannel, LocalChannel, NotificationService};
use std::sync::mpsc;
use std::thread;

pub fn main() {
    pretty_env_logger::init();
    log::set_max_level(log::LevelFilter::Trace);

    let (tx, rx) = mpsc::channel();
    let local = LocalChannel::new(tx.clone());
    let mut intel = IntelChannel::new(tx.clone());
    let notifications = NotificationService::new();
    let audio_notification = AudioNotification::new();
    debug!("Before sound notification");
    audio_notification.notify("eIntel Online");

    thread::spawn(|| {
        debug!("Channel thread running");
        Channels::new()
            .player("Derzerek")
            .player("Yolla")
            .name("GotG Home Intel")
            .name("Derzerek")
            .watch(tx);
    });

    loop {
        match rx.recv() {
            Ok(Event::ChannelResumed(mut channel)) => if channel.is_local() {
                local.process(&mut channel);
            },
            Ok(Event::ChannelChanged(mut channel)) => if channel.is_local() {
                local.process(&mut channel);
            } else {
                intel.process(&mut channel);
            },
            Ok(Event::LocationChanged(location)) => intel.new_location(location),
            Ok(Event::IntelReport(message)) => notifications.notify(message),
            Ok(event) => warn!("Unknown event: {:?}", event),
            Err(e) => {
                error!("{:?}: {}", e, e);
                return;
            }
        };
    }
}
