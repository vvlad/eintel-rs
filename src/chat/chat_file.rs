
use std::io::prelude::*;
use std::io::BufReader;
use std::io::{Error, ErrorKind};
use std::io::Result;
use std::fs::File;

extern crate encoding;
use self::encoding::{Encoding, DecoderTrap};
use self::encoding::all::UTF_16LE as UTF16;
use std::collections::HashMap;
use std::str::Lines;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct ChatFile {
    path: String,
    content: String,
    player: String,
    name: String,
    created_at: String,
    headers: HashMap<String, String>,
    offset: usize,
}

const CHANNEL_NAME: &'static str = "Channel Name";
const CHANNEL_LISTENER: &'static str = "Listener";
const CHANNEL_STARTED: &'static str = "Session started";
const CHANNEL_HEADER_PATTERN: &'static str = "---------------------------------------------------------------";

impl ChatFile {
    pub fn id(&self) -> String {
        return format!("{}-{}", self.player, self.name); 
    }

    pub fn player(&self) -> String {
        return self.player.clone();
    }

    pub fn name(&self) -> String {
        return self.name.clone();
    }

    pub fn content(&self) -> String {
        return self.content.clone();
    }

    pub fn content_length(&self) -> usize {
        return self.content.len();
    }

    pub fn lines(&mut self) -> Lines {
        return self.content.split_at(self.offset).1.lines();
    }

    pub fn headers(&self) -> HashMap<String, String> {
        return self.headers.clone();
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    fn read_utf16(path: &str) -> Result<Vec<u8>> {
        let mut file = try!(File::open(path));
        let mut buf: Vec<u8> = vec![]; 
        try!(file.read_to_end(&mut buf));
        drop(file);
        match UTF16.decode(&buf, DecoderTrap::Ignore) {
            Ok(content) => { Ok(content.into_bytes()) } ,
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        }
    }
}

impl From<String> for ChatFile {
    fn from(path: String) -> Self {
        let buffer = String::from_utf8(ChatFile::read_utf16(&path).expect("should read")).expect("should convert");
        
        let mut parts = buffer.splitn(3, CHANNEL_HEADER_PATTERN);
        let headers = parse_channel_headers(parts.nth(1).map_or("", |m| m).to_string());
        let content = parts.nth(0).map_or("", |m| m).to_string();

        ChatFile{
            path: path,
            content: content, 
            headers: headers.clone(), 
            player: headers.get(CHANNEL_LISTENER).map_or("", |m| m).to_string(),
            name: headers.get(CHANNEL_NAME).map_or("", |m| m).to_string(),
            created_at: headers.get(CHANNEL_STARTED).map_or("", |m| m).to_string(), 
            offset: 0,
        }
    }

}

impl Hash for ChatFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.player.hash(state);
    }
}

impl PartialEq for ChatFile {
    fn eq(&self, other: &ChatFile) -> bool {
        self.name == other.name && self.player == other.player 
    }
}

impl Eq for ChatFile {}

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
