use std::path::PathBuf;

extern crate encoding;
use self::encoding::all::UTF_16LE;
use self::encoding::{DecoderTrap, Encoding};

extern crate memmap;

use self::memmap::{Mmap, MmapOptions};

use std::fs::{read_dir, OpenOptions};

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::hash::{Hash, Hasher};
use std::sync::mpsc;
use std::time::Duration;

extern crate chrono;
use self::chrono::prelude::*;

extern crate notify;
use self::notify::Watcher;

use Event;

enum HeaderPositionLookup {
    ExpectingFF,
    ExpectingFE,
    Expecting5B,
    PositionFound,
}

impl HeaderPositionLookup {
    fn header_lookup_state(&self, c: u8) -> Self {
        match c {
            0xff => match self {
                HeaderPositionLookup::ExpectingFF => HeaderPositionLookup::ExpectingFE,
                _ => HeaderPositionLookup::ExpectingFF,
            },
            0xfe => match self {
                HeaderPositionLookup::ExpectingFE => HeaderPositionLookup::Expecting5B,
                _ => HeaderPositionLookup::ExpectingFF,
            },
            0x5b => match self {
                HeaderPositionLookup::Expecting5B => HeaderPositionLookup::PositionFound,
                _ => HeaderPositionLookup::ExpectingFF,
            },
            _ => HeaderPositionLookup::ExpectingFF,
        }
    }

    fn found_header(&self) -> bool {
        match self {
            HeaderPositionLookup::PositionFound => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
enum ChannelInfoParser {
    ExpectingSeparator,
    ExpectingField,
    Field(String, String),
    Done,
}

impl ChannelInfoParser {
    fn new() -> Self {
        ChannelInfoParser::ExpectingSeparator
    }

    fn parse_line(self, line: &str) -> Self {
        match self {
            ChannelInfoParser::ExpectingSeparator if line.starts_with("-") => {
                ChannelInfoParser::ExpectingField
            }
            ChannelInfoParser::Field(_, _) | ChannelInfoParser::ExpectingField
                if line.starts_with("-") =>
            {
                ChannelInfoParser::Done
            }
            ChannelInfoParser::Field(_, _) | ChannelInfoParser::ExpectingField => {
                let tokens = line
                    .splitn(2, ":")
                    .map(|token| token.trim())
                    .collect::<Vec<_>>();
                if tokens.len() != 2 {
                    ChannelInfoParser::ExpectingField
                } else {
                    ChannelInfoParser::Field(tokens[0].to_owned(), tokens[1].to_owned())
                }
            }
            ChannelInfoParser::Done | ChannelInfoParser::ExpectingSeparator => self,
        }
    }
}

#[derive(Debug, Eq, Clone)]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
    pub player: String,
    header_len: usize,
    offset: usize,
    version: usize,
    path: PathBuf,
    changes: Option<String>,
}

impl Hash for ChannelInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.player.hash(state);
    }
}

impl PartialEq for ChannelInfo {
    fn eq(&self, other: &ChannelInfo) -> bool {
        self.name == other.name && self.player == other.player
    }
}

impl ChannelInfo {
    fn same_version(&self, other: &ChannelInfo) -> bool {
        self.eq(other) && self.version == other.version
    }
}

impl ChannelInfo {
    pub fn new(path: PathBuf) -> Option<Self> {
        let mem = mem_open(&path)?;
        let size = header_size(&mem)?;
        let content = decode_utf16(&mem[0..size])?;
        let fields = header_fields(content);

        let version = Utc
            .datetime_from_str(fields.get("Session started")?, "%Y.%m.%d %H:%M:%S")
            .ok()?
            .timestamp() as usize;

        let header = ChannelInfo {
            id: fields.get("Channel ID")?.to_owned(),
            name: fields.get("Channel Name")?.to_owned(),
            player: fields.get("Listener")?.to_owned(),
            offset: size,
            header_len: size,
            version: version,
            path: path,
            changes: None,
        };
        Some(header)
    }

    pub fn is_local(&self) -> bool {
        self.id == "local"
    }

    pub fn player(&self) -> String {
        self.player.clone()
    }

    pub fn fast_forward(mut self) -> Self {
        if let Some(mem) = mem_open(&self.path) {
            self.offset = mem.len();
        }
        self
    }

    pub fn changes(&mut self) -> String {
        self.changes.clone().unwrap_or({
            fn content(path: &PathBuf, offset: usize) -> Option<(String, usize)> {
                let mem = mem_open(path)?;
                let content = decode_utf16(&mem[offset..])?;
                Some((content, mem.len()))
            }

            match content(&self.path, self.offset) {
                Some((content, offset)) => {
                    self.offset = offset;
                    self.changes = Some(content.clone());
                    content
                }
                None => "".to_owned(),
            }
        })
    }

    pub fn messages(&mut self) -> Vec<String> {
        self.changes()
            .lines()
            .map(|line| line.trim().replace("\u{feff}", ""))
            .map(|line| line.to_owned())
            .collect::<Vec<_>>()
    }
}

fn decode_utf16(buf: &[u8]) -> Option<String> {
    UTF_16LE.decode(buf, DecoderTrap::Ignore).ok()
}

fn header_size(buf: &[u8]) -> Option<usize> {
    let mut state = HeaderPositionLookup::ExpectingFF;
    let pos = buf.iter().position(|&c| {
        state = state.header_lookup_state(c);
        state.found_header()
    })?;

    Some(pos - 2)
}

fn header_fields(header: String) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    let mut parser = ChannelInfoParser::new();

    let header = header
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    for line in header.lines() {
        let state = parser.parse_line(line);
        if let ChannelInfoParser::Field(field, value) = state.clone() {
            map.insert(field, value);
        }

        parser = state;
    }

    return map;
}

pub struct Channels {
    path: PathBuf,
    players: Vec<String>,
    names: Vec<String>,
}

impl Channels {
    pub fn new() -> Self {
        Channels {
            path: env::home_dir().unwrap().join("Documents/EVE/logs/Chatlogs"),
            players: vec![],
            names: vec!["Local".to_owned()],
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.names.push(name.to_owned());
        self
    }

    pub fn player(mut self, name: &str) -> Self {
        self.players.push(name.to_owned());
        self
    }

    pub fn watch(&self, events: mpsc::Sender<Event>) {
        let mut info_cache = HashSet::new();

        for channel in self.all() {
            events.send(Event::ChannelResumed(channel.clone())).is_ok();
            info_cache.insert(channel.fast_forward());
        }

        let (tx, rx) = mpsc::channel();

        let mut watcher: notify::RecommendedWatcher =
            Watcher::new(tx, Duration::from_millis(200)).unwrap();

        watcher
            .watch(self.path.clone(), notify::RecursiveMode::Recursive)
            .is_ok();
        loop {
            match rx.recv() {
                Ok(notify::DebouncedEvent::Write(path))
                | Ok(notify::DebouncedEvent::Chmod(path)) => {
                    if let Some(info) = ChannelInfo::new(path) {
                        if self.is_relevant_channel(&info) {
                            let relevant_info = match info_cache.get(&info) {
                                Some(cached) => if info.same_version(cached) {
                                    cached.clone()
                                } else {
                                    info
                                },
                                None => info,
                            };
                            events
                                .send(Event::ChannelChanged(relevant_info.clone()))
                                .is_ok();
                            info_cache.replace(relevant_info.fast_forward());
                        }
                    }
                }
                Err(e) => {
                    println!("{:?}: {}", e, e);
                    return;
                }
                Ok(_event) => {
                    // debug!("unhandled event {:?}", event);
                }
            };
        }
    }

    pub fn all(&self) -> Vec<ChannelInfo> {
        fn build_channels<'r, F>(path: PathBuf, filter: F) -> Option<Vec<ChannelInfo>>
        where
            F: FnMut(&ChannelInfo) -> bool,
        {
            let channels = read_dir(path)
                .ok()?
                .filter_map(|entry| entry.ok())
                .map(|entry| ChannelInfo::new(entry.path()))
                .filter_map(|info| info)
                .filter(filter);

            let mut set: HashSet<ChannelInfo> = HashSet::new();

            for channel in channels {
                if set.contains(&channel) {
                    if channel.version > set.get(&channel)?.version {
                        set.replace(channel);
                    };
                } else {
                    set.insert(channel);
                }
            }

            let channels = set.drain().collect::<Vec<_>>();
            if channels.len() != 0 {
                Some(channels)
            } else {
                None
            }
        };

        build_channels(self.path.clone(), |info| self.is_relevant_channel(info)).unwrap_or_default()
    }

    pub fn is_relevant_channel(&self, info: &ChannelInfo) -> bool {
        self.players.contains(&info.player) && self.names.contains(&info.name)
    }
}

fn mem_open(path: &PathBuf) -> Option<Mmap> {
    let file = OpenOptions::new().read(true).open(path).ok()?;
    unsafe { MmapOptions::new().map(&file).ok() }
}
