use std::path;
use std::env;
use std::sync::mpsc;
use std::thread;
use chat::channel;
use chat;

extern crate notify;
use self::notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};

extern crate regex;
use self::regex::Regex;

use std::io::Result;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;

extern crate encoding;
use self::encoding::{Encoding, DecoderTrap};
use self::encoding::all::UTF_16LE as UTF16;

type Player = String;

pub struct Chat {
    directory: path::PathBuf,
    channel_names: Vec<String>,
    players: Vec<Player>,
    messages: mpsc::Sender<chat::Event>
}


impl Chat {
    pub fn new(messages: mpsc::Sender<chat::Event>) -> Self {
        let mut path = path::PathBuf::from(env::home_dir().expect("should exist"));
        path.push("Documents/EVE/logs/Chatlogs");
        Chat{
            directory: path,
            channel_names: vec!["Local".to_owned()],
            players: vec![],
            messages: messages
        }
    }

    pub fn channel(&mut self, name: &str) {
        self.channel_names.push(name.to_owned());
    }

    pub fn player(&mut self, name: &str) {
        self.players.push(name.to_owned());
    }

    pub fn run(&mut self) {
        let path = self.directory.to_str().expect("should exist").to_owned();
        let channel_names = self.channel_names.clone();
        let (events_tx, events) = mpsc::channel();
        thread::spawn(move || { watch(path, channel_names, events_tx).is_ok(); });
        
        let mut channels = vec![];

        for player in self.players.iter() {
            for name in self.channel_names.iter() {
                let new_channel = channel::Channel::new(name.to_owned(), player.to_owned(), self.messages.clone());
                println!("{:?}, {:?}", self.directory.to_str(), new_channel);
                channels.push(new_channel);
            }
        }

        loop { 
            match events.recv() {
                Ok(event) => { 
                    let find_channel = |probe: &&mut channel::Channel| {
                        return probe.name.to_uppercase() == event.name.to_uppercase() && probe.player.to_uppercase() == event.player.to_uppercase() 
                    };
                    match channels.iter_mut().find(find_channel) {
                        Some(channel) => { channel.update(event.content.clone(), event.version); },
                        None => { return ; }
                    };
                }
                Err(error) => { println!("error: {:?} {}", error, error); break;}
            };
        }
    }
}

fn watch(path: String, channels: Vec<String>, events: mpsc::Sender<ChangeInfo>) -> notify::Result<()> 
{

    let (tx, rx) = mpsc::channel();
    let mut watcher = try!(raw_watcher(tx));

    try!(watcher.watch(path, RecursiveMode::Recursive));
    let pattern = format!(r"({})_(\d+)_(\d+).TXT$", channels.iter()
        .map( |channel| channel.to_uppercase())
        .collect::<Vec<_>>()
        .join("|"));

    println!("{}", pattern);
    let regex =  Regex::new(&pattern).expect("must compile");

    loop {
        match rx.recv() {
            Ok(RawEvent{path: Some(path), op: Ok(_), cookie: _}) => {
                let normalized_path = String::from(path.to_str().expect("must exist"));
                let upcase_path = normalized_path.to_uppercase();

                if regex.is_match(&upcase_path) {

                    println!("watch({})", normalized_path);
                    let mut info = change_info(&normalized_path, &regex); 
                    println!("{:?}", info);
                    events.send(info).is_ok();
                }
            },
            Ok(event) => println!("broken event: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

const CHANNEL_NAME: &'static str = "Channel Name";
const CHANNEL_LISTENER: &'static str = "Listener";
const CHANNEL_HEADER_PATTERN: &'static str = "---------------------------------------------------------------";

#[derive(Clone, Debug)]
pub struct ChangeInfo {
    pub player: String,
    pub name: String,
    pub content: String,
    path: String,
    pub version: u64
}

pub fn change_info(file_name: &str, version_regex: &Regex) -> ChangeInfo {
    let content = String::from_utf8(read_utf16(file_name).unwrap_or(vec![])).expect("ok charset");
    let mut parts = content.splitn(3, CHANNEL_HEADER_PATTERN);
    let headers = parse_channel_headers(parts.nth(1).unwrap_or(&"".to_owned()).to_owned());
    let upcase_path = file_name.to_uppercase();
    let version_parts = version_regex.captures(&upcase_path).expect("should match");
    let version = format!("{}{}",
        version_parts.get(2).map_or("".to_string(), |m| m.as_str().to_string()),
        version_parts.get(3).map_or("".to_string(), |m| m.as_str().to_string())
    ).parse::<u64>().unwrap_or(0);

    ChangeInfo {
        name: headers.get(CHANNEL_NAME).unwrap_or(&"".to_owned()).to_owned(),
        player: headers.get(CHANNEL_LISTENER).unwrap_or(&"".to_owned()).to_owned(),
        content: parts.nth(0).unwrap_or(&"".to_owned()).to_owned(),
        version: version,
        path: file_name.to_owned()
    }
}

pub fn read_utf16(path: &str) -> Result<Vec<u8>> {
    let mut file = try!(File::open(path));
    let mut buf: Vec<u8> = vec![]; 
    try!(file.read_to_end(&mut buf));
    drop(file);
    match UTF16.decode(&buf, DecoderTrap::Ignore) {
        Ok(content) => { Ok(content.into_bytes()) } ,
        Err(e) => return Err(Error::new(ErrorKind::Other, e)),
    }
}

fn parse_channel_headers(content: String) -> HashMap<String, String> {
    let mut headers : HashMap<String, String> = HashMap::new();
    for line in content.lines() {
        let normalized_line = line.trim();
        if normalized_line.is_empty() { continue }
        let mut parts = normalized_line.splitn(2, ":");
        let key = parts.nth(0).unwrap().to_string();
        let value = parts.nth(0).unwrap().trim().to_string();
        headers.insert(key, value); 
    }
    return headers;
}

