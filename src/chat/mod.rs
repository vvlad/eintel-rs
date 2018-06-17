use super::config;
use super::errors::*;
use super::events::Event;

pub mod channel;
mod header;

use chrono::prelude::*;
use notify;
use notify::Watcher;
use regex;
use std::collections::HashSet;
use std::path;
use std::sync::mpsc;
use std::time;

lazy_static! {
    static ref message_pattern: regex::Regex = {
        regex::Regex::new(r"^\[ (\d{4}\.\d{2}\.\d{2} \d{2}:\d{2}:\d{2}) \] (.+) > (.*)")
            .expect("must compile")
    };
}

#[derive(Debug)]
pub struct Message {
    pub received_at: DateTime<Utc>,
    pub listener: String,
    pub channel: String,
    pub sender: String,
    pub message: String,
}

impl Message {
    pub fn from(info: &channel::Header, line: &str) -> Result<Message> {
        let matches = message_pattern.captures(line).ok_or("match failed")?;
        let sender = matches.get(2).unwrap().as_str().to_string();
        let line = matches.get(3).unwrap().as_str().trim().to_owned();
        let received_at =
            Utc.datetime_from_str(matches.get(1).unwrap().as_str(), "%Y.%m.%d %H:%M:%S")?;

        Ok({
            Message {
                received_at: received_at,
                listener: info.listener.clone(),
                channel: info.name.clone(),
                sender: sender,
                message: line,
            }
        })
    }

    pub fn is_local_channel(&self) -> bool {
        self.channel == "Local"
    }
}

pub fn watch(conf: &config::Config, chan: mpsc::Sender<Event>) -> Result<()> {
    let mut channels = HashSet::new();
    let mut messages: Vec<Message> = vec![];

    for mut channel in restore_channels(conf)?.into_iter() {
        messages.append(&mut channel.messages()?);
        channels.insert(channel);
    }

    messages.sort_by(|first, last| first.received_at.cmp(&last.received_at));
    for message in messages.into_iter() {
        if is_relevant_message(&message, &conf) {
            chan.send(Event::PreviousMessage(message))?;
        }
    }

    let (tx, fs_events) = mpsc::channel();
    let mut watcher: notify::RecommendedWatcher =
        notify::Watcher::new(tx, time::Duration::from_millis(200)).unwrap();

    watcher.watch(conf.chat_logs.clone(), notify::RecursiveMode::Recursive)?;
    loop {
        match fs_events.recv()? {
            notify::DebouncedEvent::Write(path) | notify::DebouncedEvent::Chmod(path) => {
                let mut channel = figureout_channel(channel::Channel::from(&path)?, &mut channels);
                for message in channel.messages()?.into_iter() {
                    if is_relevant_message(&message, &conf) {
                        chan.send(Event::NewMessage(message))?;
                    }
                }
                channels.replace(channel);
            }
            _ => {}
        };
    }
}

fn is_relevant_message(message: &Message, conf: &config::Config) -> bool {
    conf.players.iter().any(|name| &message.listener == name)
        && (message.channel == "Local" && message.sender == "EVE System")
        || (message.channel != "Local" && message.sender != "EVE System")
}

fn figureout_channel(
    new: channel::Channel,
    channels: &mut HashSet<channel::Channel>,
) -> channel::Channel {
    match channels.take(&new) {
        Some(old) => if old.header.started_at >= new.header.started_at {
            old
        } else {
            new
        },
        None => new,
    }
}

fn restore_channels(conf: &config::Config) -> Result<HashSet<channel::Channel>> {
    let channel_candidates = conf
        .chat_logs
        .read_dir()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter_map(|path| relevant_channel_path(path, &conf.channels))
        .filter_map(|path| channel::Channel::from(&path).ok());

    let mut channels: HashSet<channel::Channel> = HashSet::new();

    for new in channel_candidates {
        let channel = figureout_channel(new, &mut channels);
        channels.replace(channel);
    }

    Ok(channels)
}

fn relevant_channel_path(path: path::PathBuf, channels: &Vec<String>) -> Option<path::PathBuf> {
    let valid = channels.iter().any({
        |name| {
            path.file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(name)
        }
    });

    if valid {
        Some(path)
    } else {
        None
    }
}
