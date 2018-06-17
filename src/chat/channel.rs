use super::super::chrono::prelude::*;
use super::super::encoding::all::UTF_16LE;
use super::super::encoding::{DecoderTrap, Encoding};
use super::super::errors::*;
use super::super::memmap;
use super::header;

use super::Message;

use std::fs;
use std::hash::{Hash, Hasher};
use std::path;

#[derive(Debug, Clone)]
pub struct Header {
    pub id: String,
    pub name: String,
    pub listener: String,
    pub started_at: DateTime<Utc>,
    pub offset: usize,
}

impl Header {
    pub fn from(mem: &[u8]) -> Result<Header> {
        let len = header::Length::from(mem).ok_or("unable to open file")?;
        let content = UTF_16LE
            .decode(&mem[0..len], DecoderTrap::Ignore)
            .ok()
            .ok_or("unable to decode file")?;
        let fields = header::Fields::from(content);

        let started_at = Utc.datetime_from_str(
            fields
                .get("Session started")
                .ok_or("No session started field")?,
            "%Y.%m.%d %H:%M:%S",
        )?;

        Ok({
            Header {
                id: fields
                    .get("Channel ID")
                    .ok_or("Missing 'Channel ID' field")?
                    .to_owned(),
                name: fields
                    .get("Channel Name")
                    .ok_or("Missing 'Channel Name' field")?
                    .to_owned(),
                listener: fields
                    .get("Listener")
                    .ok_or("Missing 'Listener' field")?
                    .to_owned(),
                started_at: started_at,
                offset: len,
            }
        })
    }

    pub fn len(&self) -> usize {
        self.offset
    }
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub header: Header,
    pub path: path::PathBuf,
    pub offset: usize,
}

impl Hash for Channel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.header.name.hash(state);
        self.header.listener.hash(state);
    }
}

impl PartialEq for Channel {
    fn eq(&self, other: &Channel) -> bool {
        self.header.listener == other.header.listener && self.header.name == other.header.name
    }
}
impl Eq for Channel {}

impl Channel {
    pub fn contents(&self) -> Result<memmap::Mmap> {
        let file = fs::OpenOptions::new().read(true).open(&self.path)?;
        Ok(unsafe { memmap::MmapOptions::new().map(&file)? })
    }

    pub fn from(path: &path::PathBuf) -> Result<Channel> {
        let file = fs::OpenOptions::new().read(true).open(path)?;
        let mem = unsafe { memmap::MmapOptions::new().map(&file)? };
        let header = Header::from(&mem)?;
        let offset = header.offset;
        Ok({
            Channel {
                header: header,
                path: path.clone(),
                offset: offset,
            }
        })
    }

    pub fn messages(&mut self) -> Result<Vec<Message>> {
        let mem = self.contents()?;

        let messages = UTF_16LE
            .decode(&mem[self.offset..], DecoderTrap::Ignore)
            .ok()
            .ok_or("unable to decode file")?
            .lines()
            .map(|line| line.trim().replace("\u{feff}", ""))
            .filter_map(|line| Message::from(&self.header, &line).ok())
            .collect();
        self.offset = mem.len();
        Ok(messages)
    }
}
