extern crate eve_universe;

use chat;
use std::sync::mpsc;
use std::collections::HashMap;
use std::cell::RefCell;
use std::fs::File;
use std::io::prelude::*;
use self::eve_universe::{System, ship_exists };
use universe;

pub struct Intel {
    locations: HashMap<String, System>
}

impl Intel {
    pub fn new() -> Self {
        Intel{
            locations: HashMap::new()
        }
    }

    pub fn run(&mut self, channel: mpsc::Receiver<chat::Event>) {
        loop {
            match channel.recv() {
                Ok(chat::Event::ChatMessage(message)) => {
                    if let Some(system) = self.locations.get(&message.player) {
                        let intel_message = Message::new(message, system.to_owned());
                        if intel_message.is_interesting() {
                            println!("=======\n{:?} {:?}\n======\n", intel_message.message.message, intel_message.involved_players);
                        }
                    }
                },
                Ok(chat::Event::Location(location)) => {
                    println!("{:?}", location);
                    self.locations.insert(location.player,  universe::find(location.system));
                },
                Err(error) => {
                    println!("Intel#run error: {:?} {}", error, error);
                }
            }
        }
    }
}

thread_local!{
    static STOP_WORDS: RefCell<Vec<String>> = RefCell::new(load_stop_words());
}

#[derive(Debug)]
pub struct Message {
    message: chat::Message,
    player_location: System,
    tokens: Vec<String>,
    route: Option<universe::Route>,
    system: Option<System>,
    involved_players: Vec<String>
}

impl Message{
    pub fn new(message: chat::Message, player_location: System) -> Self {
        let line = normalize(message.message.clone());
        let tokens = build_tokens(line.clone());
        let route  = Self::route(&tokens, &player_location);
        let system = route.as_ref().map( |r| r.destination.clone() );
        let players = possible_names(line.clone());

        Message {
            player_location: player_location,
            message: message,
            tokens: tokens,
            route: route,
            system: system,
            involved_players: players
        }
    }

    fn route(tokens: &Vec<String>, destination: &System) -> Option<universe::Route> {
       let systems = universe::find_systems(tokens);
       let mut routes = systems
           .iter()
           .map( |system| universe::route(system, destination) )
           .filter( |route| route.is_some() )
           .map( |route| route.unwrap() )
           .collect::<Vec<_>>();

       routes.sort_by( |a, b| a.distance.cmp(&b.distance) );
       routes.pop()
    }

    fn is_interesting(&self) -> bool {
        self.route.is_some()
    }
}

fn build_tokens(text: String) -> Vec<String> {
    text.split_whitespace()
        .map(|x| x.to_uppercase().replace("*", ""))
        .filter(stop_words)
        .filter(ships)
        .collect()
}

fn normalize(text: String) -> String {
    let result = text
        .replace("*", "")
        .split("  ")
        .map( |x| {
            x.split(" ")
                .map( |x| x.to_string() )
                .filter(ships)
                .filter(stop_words)
                .collect::<Vec<String>>()
                .join(" ")
        })
        .filter( |s| !s.is_empty() )
        .collect::<Vec<_>>()
        .join("  ");

    println!("[{}] => [{}]", text, result); 
    result

}

fn possible_names(line: String) -> Vec<String> {
    let names = line.split("  ")
        .map( |token| token.to_string() )
        .filter( |token| !universe::is_system_name(&token.to_string()) )
        .filter(ships) 
        .map( |token| token.to_string() )
        .collect::<Vec<String>>();
    return names;
}

fn load_stop_words() -> Vec<String> {
    let mut file = File::open("data/stop_words.txt").unwrap();
    let mut content = String::new();

    file.read_to_string(&mut content).is_ok();

    return content.lines().map(|x| { x.to_string() }).collect();
}

fn stop_words(word: &String) -> bool {
    STOP_WORDS.with(|f| !f.borrow().contains(&word.to_uppercase())) 
}

fn ships(token: &String) -> bool {
    ship_exists(&token)
}

