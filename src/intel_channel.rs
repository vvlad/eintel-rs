use std::collections::{HashMap, HashSet};
use std::sync::mpsc;

use universe;
use {ChannelInfo, Event, IntelMessage, PlayerLocation, System, ThreatAssetment};
extern crate regex;

pub struct IntelChannel {
    events: mpsc::Sender<Event>,
    player_locations: HashMap<String, PlayerLocation>,
    regex: regex::Regex,
}

impl IntelChannel {
    pub fn new(channel: mpsc::Sender<Event>) -> Self {
        IntelChannel {
            events: channel,
            player_locations: HashMap::new(),
            regex: regex::Regex::new(r"^\[ \d{4}\.\d{2}\.\d{2} \d{2}:\d{2}:\d{2} \] (.+) > (.*)")
                .expect("must compile"),
        }
    }

    pub fn new_location(&mut self, location: PlayerLocation) {
        info!("{} is in {}", location.player, location.system.name);
        self.player_locations
            .insert(location.player.clone(), location);
    }

    pub fn process(&mut self, info: &mut ChannelInfo) -> Option<()> {
        let location = self.player_locations.get(&info.player())?;
        let messages = info.messages();
        for message in messages {
            if let Some(matches) = self.regex.captures(&message) {
                let sender = matches.get(1)?.as_str().to_string();
                if sender == "EVE System" {
                    continue;
                };
                let line = matches.get(2)?.as_str().trim().to_owned();
                let message = self.debounce(IntelMessage::new(line, &location, sender)?)?;
                self.events.send(Event::IntelReport(message)).is_ok();
            }
        }
        Some(())
    }

    fn debounce(&self, message: IntelMessage) -> Option<IntelMessage> {
        Some(message)
    }
}

impl IntelMessage {
    pub fn new(message: String, location: &PlayerLocation, sender: String) -> Option<Self> {
        let line = normalize(&message);
        let tokens = tokenize(line.clone());
        let (route, tokens) = Self::route(&tokens, &location.system);
        let system = route.as_ref().map(|r| r.destination.clone());
        let (threat_level, tokens) = assess_thread_level(tokens, &route);
        let players = possible_names(line.clone());

        Some({
            IntelMessage {
                player: location.player.clone(),
                message: message,
                tokens: tokens,
                route: route?,
                origin: system?,
                involved_players: players,
                threat_assement: threat_level,
                sender: sender,
            }
        })
    }

    fn route(
        tokens: &Vec<String>,
        destination: &universe::System,
    ) -> (Option<universe::Route>, Vec<String>) {
        let mut system_names = HashSet::new();
        let mut routes = tokens
            .iter()
            .map(|ref mut token| {
                let x = token.clone();
                System::find(&token).and_then(|system| {
                    system_names.insert(x);
                    Some(system)
                })
            })
            .filter_map(|system| universe::route(destination, &system?))
            .collect::<Vec<_>>();

        routes.sort_by(|a, b| a.distance.cmp(&b.distance));

        let new_tokens = tokens
            .iter()
            .filter(|token| !system_names.contains(token.clone()))
            .map(|token| token.to_owned())
            .collect::<Vec<_>>();

        (routes.pop(), new_tokens)
    }
}

fn tokenize(text: String) -> Vec<String> {
    text.split_whitespace()
        .map(|x| {
            x.to_uppercase()
                .replace("*", "")
                .replace("?", "")
                .replace("Solar System -", "")
        })
        .filter(stop_words)
        .filter(ships)
        .collect()
}

fn normalize(text: &str) -> String {
    text.replace("*", "")
        .split("  ")
        .map(|x| {
            x.split(" ")
                .map(|x| x.to_string())
                .filter(ships)
                .filter(stop_words)
                .collect::<Vec<String>>()
                .join(" ")
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("  ")
}

fn possible_names(line: String) -> Vec<String> {
    line.split("  ")
        .map(|token| token.to_string())
        .filter(|token| !System::find(token).is_some())
        .filter(ships)
        .filter(stop_words)
        .map(|token| token.to_string())
        .collect::<Vec<String>>()
}

fn stop_words(word: &String) -> bool {
    !universe::is_stop_word(word)
}

fn ships(token: &String) -> bool {
    !universe::ship_exists(&token)
}

fn assess_thread_level(
    tokens: Vec<String>,
    route: &Option<universe::Route>,
) -> (ThreatAssetment, Vec<String>) {
    match route {
        Some(route) => {
            let mut tokens = tokens.clone();
            let mut level = match route.distance {
                1...4 => ThreatAssetment::ProximityAlertHigh(route.distance),
                5...10 => ThreatAssetment::ProximityAlertLow(route.distance),
                0 => ThreatAssetment::ProximityAlertCritical(route.distance),
                _ => ThreatAssetment::ProximityIrelevant(route.distance),
            };
            let no_proximity = vec!["CLR", "CLEAR", "CLEA"]
                .iter()
                .map(|&x| x.to_owned())
                .collect::<Vec<String>>();
            let no_threat = vec!["STS", "STATUS", "STAT"]
                .iter()
                .map(|&x| x.to_owned())
                .collect::<Vec<String>>();

            if let Some(new_tokens) = tokens_difference(&tokens, no_proximity) {
                if route.distance <= 5 {
                    (
                        ThreatAssetment::NoThreat(route.destination.clone()),
                        new_tokens,
                    )
                } else {
                    (
                        ThreatAssetment::ProximityIrelevant(route.distance),
                        new_tokens,
                    )
                }
            } else if let Some(new_tokens) = tokens_difference(&tokens, no_threat) {
                (
                    ThreatAssetment::StatusRequest(route.destination.clone()),
                    new_tokens,
                )
            } else {
                (level, tokens)
            }
        }
        None => (ThreatAssetment::Unknown, tokens),
    }
}

fn tokens_difference(first: &Vec<String>, last: Vec<String>) -> Option<Vec<String>> {
    let len = first.len();
    let mut last = last.iter();
    let last = last.by_ref();
    let mut first = first.iter();
    let first = first.by_ref();

    let resutls = first
        .filter(|first_item| !last.any(|last_item| *first_item == last_item))
        .map(|item| item.to_string())
        .collect::<Vec<String>>();

    if resutls.len() != len {
        Some(resutls)
    } else {
        None
    }
}
