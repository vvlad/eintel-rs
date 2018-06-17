use super::chat;
use super::universe;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub enum ThreatAssetment {
    Unknown,
    NoThreat(universe::System),
    ProximityIrelevant(u16),
    ProximityAlertLow(u16),
    ProximityAlertHigh(u16),
    ProximityAlertCritical(u16),
    StatusRequest(universe::System),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub message: String,
    pub player: String,
    pub tokens: Vec<String>,
    pub route: universe::Route,
    pub origin: universe::System,
    pub involved_players: Vec<String>,
    pub threat_assement: ThreatAssetment,
    pub sender: String,
}

impl Message {
    pub fn new(message: chat::Message, location: &universe::System) -> Option<Message> {
        let line = normalize(&message.message);
        let tokens = tokenize(line.clone());
        let (route, tokens) = Self::route(&tokens, &location);
        let system = route.as_ref().map(|r| r.destination.clone());
        let (threat_level, tokens) = assess_thread_level(tokens, &route);
        let players = possible_names(line.clone());

        Some({
            Message {
                player: message.listener.clone(),
                message: message.message.clone(),
                tokens: tokens,
                route: route?,
                origin: system?,
                involved_players: players,
                threat_assement: threat_level,
                sender: message.sender.clone(),
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
                universe::System::find(&token).and_then(|system| {
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
        .filter(|token| !universe::System::find(token).is_some())
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
