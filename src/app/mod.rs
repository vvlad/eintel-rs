use super::chat;
use super::config;
use super::errors::*;
use super::events;
use super::intel::Intel;
use super::notifications;

use std::sync::mpsc;
use std::thread;

pub fn run(conf: config::Config) -> Result<()> {
    let (tx, messages) = mpsc::channel();
    let watch_conf = conf.clone();
    let mut intel = Intel::new(tx.clone());
    let notifications = notifications::Notifications::new();
    thread::spawn(move || {
        chat::watch(&watch_conf, tx).is_ok();
    });

    loop {
        let message = messages.recv()?;
        match message {
            events::Event::PreviousMessage(message) => if message.is_local_channel() {
                intel.location_message(message)?;
            },
            events::Event::NewMessage(message) => if message.is_local_channel() {
                intel.location_message(message)?;
            } else {
                intel.intel_message(message)?;
            },
            events::Event::IntelReport(message) => notifications.deliver(message)?,
        };
    }
}
