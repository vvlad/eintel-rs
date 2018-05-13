extern crate regex;

use self::regex::Regex;

use chat;
use std::sync::mpsc;

lazy_static! {
    static ref MESSAGE_LINE: Regex = Regex::new(r"\[ (\d{4}\.\d{2}\.\d{2} \d{2}:\d{2}:\d{2}) \] (.+) > (.*)").expect("should compile");
    static ref LOCATION_CHANGED: Regex = Regex::new(r"\[ \d{4}\.\d{2}\.\d{2} \d{2}:\d{2}:\d{2} \] EVE System > Channel changed to Local : (.*)").expect("should compile");
}

#[derive(Debug)]
pub struct Channel {
    pub name: String,
    pub player: String,
    current_version: u64,
    content_offset: usize,
    messages: mpsc::Sender<chat::Event>
}

impl Channel {

    pub fn new(name: String, player: String, messages: mpsc::Sender<chat::Event>) -> Self {
        Channel{
            name: name,
            player: player,
            current_version: 0,
            content_offset: 0,
            messages: messages
        }
    }

    pub fn update(&mut self, content: String, version: u64) {
        if self.current_version != version {
            self.content_offset = 0;
        }

        let new_content = content.split_at(self.content_offset).1;
        let mut lines = new_content.lines().collect::<Vec<_>>();

        if self.current_version == 0 && self.name != "Local" {
            lines = vec![lines.last().unwrap()]; 
        }

        if self.name == "Local" {
            self.reflect_on_location_change(lines);
        }else{
            self.reflect_on_human_input(lines);
        }
        self.content_offset += new_content.len();
        self.current_version = version;
    }

    fn reflect_on_human_input(&self, lines: Vec<&str>) {
        for line in lines {
            if let Ok((sender, message)) = self.parse_line(line)  {

                let message = chat::Message {
                    player: self.player.clone(),
                    message: message,
                    sender: sender,
                    channel: self.name.clone()
                };

                self.messages.send(chat::Event::ChatMessage(message)).is_ok();
            }
        }
    }

    fn reflect_on_location_change(&self, lines: Vec<&str>) {
        if let Some(line) = lines.iter().filter(|line| LOCATION_CHANGED.is_match(line) ).last() {
            let parts = LOCATION_CHANGED.captures(&line).expect("should match");
            let message = chat::Location{
                player: self.player.clone(),
                system: parts[1].to_owned() 
            };
            self.messages.send(chat::Event::Location(message)).is_ok();
        }
    }

    fn parse_line(&self, line: &str) -> Result<(String,String), &str> {
        if MESSAGE_LINE.is_match(line) {
            let parts = MESSAGE_LINE.captures(line).expect("should match");
            let (_time, sender, message) = (
                parts[1].to_owned(),
                parts[2].to_owned(),
                parts[3].to_owned()
            );
            Ok((sender, message))
        }else{
            Err("no match")
        }
    }
}

