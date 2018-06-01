
use std::path::{PathBuf};

extern crate encoding;
use self::encoding::{Encoding, DecoderTrap};
use self::encoding::all::UTF_16LE; 

extern crate memmap;

use memmap::{MmapOptions, Mmap};

use std::fs::{File, OpenOptions};

use std::sync::{Arc, Weak};
use std::collections::{HashMap};

pub struct ChannelHeader {
    id: String,
    name: String,
    player: String,
    offset: usize
}


enum ChannelHeaderFinder {
    ExpectingFF,
    ExpectingFE,
    Expecting5B,
    PositionFound,
}

impl ChannelHeaderFinder {
    fn header_lookup_state(&self, c: u8) -> Self {
        match c {
            0xff => { 
                if let ChannelHeaderFinder::ExpectingFF = self {
                    ChannelHeaderFinder::ExpectingFE
                }else{
                    ChannelHeaderFinder::ExpectingFF
                }
            }
            0xfe => {
                if let ChannelHeaderFinder::ExpectingFE = self {
                    ChannelHeaderFinder::Expecting5B
                }else{
                    ChannelHeaderFinder::ExpectingFF
                }
            }
            0x5b => { 
                if let ChannelHeaderFinder::Expecting5B = self {
                    ChannelHeaderFinder::PositionFound 
                }else{
                    ChannelHeaderFinder::ExpectingFF
                }
            }
            _ => { ChannelHeaderFinder::ExpectingFF }
        }
    }

    fn found_header(&self) -> bool {
        match self {
            ChannelHeaderFinder::PositionFound => { true },
            _ => { false }
        }
    }

}

#[derive(Clone, Debug)]
enum ChannelHeaderParser {
    ExpectingSeparator,
    ExpectingField,
    Field(String, String),
    Done
}

impl ChannelHeaderParser {
    pub fn parse(header: String, offset: usize) -> Option<ChannelHeader> {
        let mut map: HashMap<String, String> = HashMap::new();
        let mut state = ChannelHeaderParser::ExpectingSeparator;

        let header = header
            .lines()
            .map(|line| line.trim() )
            .filter(|line| !line.is_empty() )
            .collect::<Vec<_>>().join("\n");


        for line in header.lines() {
           state = state.parse_line(line);
           if let ChannelHeaderParser::Field(field, value) = state.clone() {
             map.insert(field, value); 
           }
        }

        match state {
            ChannelHeaderParser::Done => { 
                {
                    let fields = vec!["Channel ID", "Channel Name", "Listener"];
                    let missing_fields = fields 
                        .iter()
                        .filter(|field| map.contains_key(**field) ).collect::<Vec<_>>().len();
                    if missing_fields != 3 { return None }
                }

                {
                    let header = ChannelHeader{
                        id: map.get("Channel ID").unwrap().to_owned(),
                        name: map.get("Channel Name").unwrap().to_owned(),
                        player: map.get("Listener").unwrap().to_owned(),
                        offset: offset
                    };
                    Some(header)
                }
            },
            _ => None
        }
    }

    fn parse_line(self, line: &str) -> Self {
        match self {
            ChannelHeaderParser::ExpectingSeparator => { 
                if line.starts_with("-") {
                    ChannelHeaderParser::ExpectingField
                } else {
                   self 
                }
            },
            ChannelHeaderParser::Field(_, _) | ChannelHeaderParser::ExpectingField => {
                if line.starts_with("-") {
                    ChannelHeaderParser::Done
                } else {

                    let tokens = line
                        .splitn(2, ":")
                        .map(|x| x.trim() )
                        .collect::<Vec<_>>();
                    if tokens.len() != 2 { return ChannelHeaderParser::ExpectingField }

                    ChannelHeaderParser::Field(tokens[0].to_owned(), tokens[1].to_owned())
                }
            },
            ChannelHeaderParser::Done => { self }
        }
    }
}

pub struct ChannelInfo {
    header: ChannelHeader,
    mem: Arc<Mmap>
}

impl ChannelInfo {

    pub fn new(mem: Arc<Mmap>) -> Option<Self> {

        if let Some(header)  = ChannelInfo::parse(mem.as_ref()) {
            let ch = ChannelInfo {
                header: header,
                mem: mem
            };
            Some(ch)
        } else {
            None
        }
    }

    fn parse(buf: &[u8]) -> Option<ChannelHeader> {
        let mut state = ChannelHeaderFinder::ExpectingFF; 
        let pos = buf
            .iter()
            .position(|&c| { 
                state = state.header_lookup_state(c);
                state.found_header() 
            });

        if let Some(pos) = pos {
            let header = match UTF_16LE.decode(&buf[2..pos-2], DecoderTrap::Ignore) {
                Ok(content) => { content } ,
                Err(_) => { return None },
            };
            return ChannelHeaderParser::parse(header, pos)
        }

        None
    }
}

pub struct ChatLogs {
    map: HashMap<PathBuf, Weak<Mmap>>
}

impl ChatLogs {
    pub fn new() -> Self {
        ChatLogs{
            map: HashMap::new()
        }
    }

    pub fn open(&mut self, path: PathBuf) -> Option<ChannelInfo> {
        let mem = self.open_cached(path);

        ChannelInfo::new(mem)
    }

    fn open_cached(&mut self, path: PathBuf) -> Arc<Mmap> {

        let mem = match self.map.get(&path){
            Some(mem) => { mem.upgrade() }
            None => { None }
        };
        
        if let Some(mem) = mem {
            println!("Hit");
            Arc::clone(&mem)
        }else{
            println!("Miss");
            let mem = Arc::new(self.mem_open(&path));
            self.map.insert(path, Arc::downgrade(&mem));
            mem
        }
    }

    fn mem_open(&self, path: &PathBuf) -> Mmap {
        unsafe { 
            let file = OpenOptions::new()
                .read(true)
                .open(path)
                .unwrap();

            MmapOptions::new()
                .map(&file)
                .unwrap()
        }
    }
}

pub fn main() {
    
    let mut file = ChatLogs::new();
    let path = PathBuf::from("/home/vvlad/Documents/EVE/logs/Chatlogs/Local_20180531_232544.txt");
    let mut channel_file = file.open(path.clone());

    file.open(path.clone());

    println!("{}", channel_file.unwrap().header.name);

    channel_file = file.open(path.clone());
    file.open(path.clone());
}
