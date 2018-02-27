extern crate regex;
use self::regex::Regex;

extern crate notify;
use self::notify::{Watcher, RecursiveMode, RawEvent, raw_watcher};

use std::path;
use std::env;
use std::sync::mpsc;
use std::thread;
use chat::ChatFile;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Message {
    player: String,
    channel: String,
    message: String
}

#[derive(Debug)]
struct ChatEvent {
    path: String,
    channel: String,
    version: u64
}

pub struct Chat{
    directory: path::PathBuf,
    channels: Vec<String>,
    players: Vec<String>,
    chat_files: HashMap<String,ChatFile>,
}

impl Default for Chat {
    fn default() -> Self {
        Chat{
            directory: path::PathBuf::from(env::home_dir().expect("should exist")),
            channels: vec![ "Local".to_owned() ],
            players: vec![],
            chat_files: HashMap::new(),
        }
    }

}

impl Chat {
    pub fn start(mut self) -> mpsc::Receiver<Message> {
        let (tx, rx) = mpsc::channel();
        let path = self.directory.to_str().expect("should exist").to_owned();
        let channels = self.channels.clone();
        let (events_tx, events_rx) = mpsc::channel();

        thread::spawn(move || { watch(path, channels, events_tx).is_ok(); });

        thread::spawn(move || {
            loop {
                match events_rx.recv() {
                    Ok(event) => { self.handle_event(event, tx.clone()) },
                    Err(e) => { println!("{:?}", e) }
                }
            }
        });
        return rx;
    }

    fn handle_event(&mut self, event: ChatEvent, messages: mpsc::Sender<Message>) {
        let file = self.chat_file(event);

        for line in file.clone().lines() {
            let message = Message{
                player: file.player(),
                channel: file.name(),
                message: line.to_owned()
            };
            messages.send(message).ok();
        }

    }

    fn chat_file(&mut self, event: ChatEvent) -> &ChatFile {
        let mut file = ChatFile::from(event.path);
        let id = file.id();
        let mut files = self.chat_files.clone();
        let offset = match files.get(&id) {
            Some(file) => file.content_length(),
            None => {
                if file.name() == "Local" {
                    0    
                }else{
                    file.content_length()
                }
            },
        };
        file.set_offset(offset);
        files.insert(id.clone(), file);
        self.chat_files = files;
        let file = self.chat_files.get_mut(&id).unwrap();
        file
    }
}

pub struct ChatBuilder {
    chat: Chat
}

impl ChatBuilder {
    pub fn new() -> Self {
        ChatBuilder{
            chat: Chat::default()
        }
    }

    pub fn directory(mut self, path: &str) -> Self {
        self.chat.directory = path::PathBuf::from(path);
        return self;
    }

    pub fn channel(mut self, channel: &str) -> Self {
        self.chat.channels.push(channel.to_owned());
        return self;
    }
    
    pub fn player(mut self, player: &str) -> Self {
        self.chat.players.push(player.to_owned());
        return self;
    }

    pub fn build(self) -> Chat {
        return self.chat;
    }
}

// extern crate crossbeam_channel;
//
// use std::sync::mpsc;
//
//
// use std::vec::Vec;
//
// use chat::utf16::UTF16File;
// use std::io::Read;
// use std::thread;
//
// pub struct Chat {
//     path: String,
//     channels: Vec<String>,
//     players: Vec<String>,
//     channel_files: HashMap<String, ChannelFile>,
//     broadcast: mpsc::Sender<Message>,
//     control_tx: crossbeam_channel::Sender<ControlMessage>,
//     control_rx: crossbeam_channel::Receiver<ControlMessage>
// }
//


//
// impl Chat {
//
//     pub fn new(path: &'static str, broadcast: mpsc::Sender<Message>) -> Chat {
//         let (tx, rx) = crossbeam_channel::unbounded();
//
//         Chat{
//             path: String::from(path),
//             channels: vec![],
//             players: vec![],
//             channel_files: HashMap::new(),
//             broadcast: broadcast,
//             control_tx: tx,
//             control_rx: rx
//         }
//     }
//
//     pub fn watch_channel(&mut self, channel_name: &'static str) {
//         self.channels.push(String::from(channel_name));
//     }
//
//     pub fn watch_player(&mut self, player_name: &'static str) {
//         self.players.push(String::from(player_name));
//     }
//
//     pub fn start(&mut self) {
//         println!("Watching {}", self.path);
//         let mut channels = vec![ String::from("Local") ];
//         channels.append(self.channels.clone().as_mut());
//         let path = self.path.clone();
//         let watcher_rx = self.control_rx.clone();
//         let watcher_tx = self.control_tx.clone();
//         let consumer_rx = self.control_rx.clone();
//
//         thread::spawn(move || {
//             watch(path, channels, watcher_tx, watcher_rx ).unwrap(); 
//         });
//
//         self.consume_events(consumer_rx);
//     }
//
//     pub fn wait(&self) {
//     }
//
//     fn consume_events(&mut self, rx: crossbeam_channel::Receiver<ControlMessage> ) {
//         loop {
//             match rx.recv() {
//                 Ok(ControlMessage::Event(event)) => { self.file_did_change(event) },
//                 Ok(ControlMessage::Quit) => break,
//                 Err(e) => { println!("error: {:?}", e) }
//             };
//         };
//     }
//
//     fn file_did_change(&mut self, event: Event) {
//         let broadcast = self.broadcast.clone();
//         let file = self.channel_file(event); 
//         let player_name = file.player_name();
//         let channel_name = file.channel_name();
//
//         for line in file.new_lines() {
//             let message = Message{
//                 player: player_name.clone(),
//                 channel: channel_name.clone(),
//                 message: line.trim().trim_matches('\u{feff}').to_string()
//             };
//             broadcast.send(message).unwrap();
//         }
//     }
//
//     fn channel_file(&mut self, event: Event) -> &mut ChannelFile {
//         let new_file = ChannelFile::from(event.path);
//         let id = new_file.id();
//         let mut files = self.channel_files.clone();
//
//         if ! files.contains_key(&id) {
//             files.insert(id.clone(), new_file.clone());
//         }; 
//
//         self.channel_files = files;
//
//         let file = self.channel_files.get_mut(&id).unwrap();
//         file.update(new_file);
//         return file;
//     }
// }

//
fn watch(path: String, channels: Vec<String>, events: mpsc::Sender<ChatEvent>) -> notify::Result<()> 
{

    let (tx, rx) = mpsc::channel();
    let mut watcher = try!(raw_watcher(tx));

    try!(watcher.watch(path, RecursiveMode::Recursive));
    let regex =  Regex::new(&format!(r"({})_(\d+)_(\d+).TXT$", channels.iter()
        .map( |channel| channel.to_uppercase())
        .collect::<Vec<_>>()
        .join("|"))).expect("must compile");

    loop {
        match rx.recv() {
            Ok(RawEvent{path: Some(path), op: Ok(_), cookie: _}) => {
                let normalized_path = String::from(path.to_str().expect("must exist"));
                let upcase_path = normalized_path.to_uppercase();
                if regex.is_match(&upcase_path) {
                    let parts = regex.captures(&upcase_path).expect("should match");
                    let version = format!("{}{}",
                        parts.get(2).map_or("".to_string(), |m| m.as_str().to_string()),
                        parts.get(3).map_or("".to_string(), |m| m.as_str().to_string())
                    ).parse::<u64>().unwrap();
                    let event = ChatEvent{
                        path: normalized_path,
                        channel: parts.get(1).map_or("".to_string(), |m| m.as_str().to_string()),
                        version: version, 
                    };
                    events.send(event).unwrap();
                }
            },
            Ok(event) => println!("broken event: {:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
