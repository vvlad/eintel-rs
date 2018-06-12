use std::collections::HashMap;
use std::fs::File;

extern crate bson;
use self::bson::{decode_document, Bson};
extern crate serde;

lazy_static! {
    static ref UNIVERSE: Universe = {
        match init() {
            Some(universe) => universe,
            None => panic!("unable to load universe"),
        }
    };
}

fn init() -> Option<Universe> {
    let mut file = File::open("universe.bson").ok()?;
    let doc = decode_document(&mut file).ok()?;
    match bson::from_bson(Bson::Document(doc)) {
        Ok(universe) => {
            return Some(universe);
        }
        Err(e) => {
            println!("unable to load 'universe.bson'. Reason: {}", e);
            return None;
        }
    };
}

#[derive(Clone, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct System {
    pub id: String,
    pub name: String,
    pub constelation: String,
    pub region: String,
    pub neighbours: Vec<String>,
}

impl System {
    pub fn find(name: &str) -> Option<System> {
        UNIVERSE
            .system_aliases
            .get(&name.to_string().to_uppercase())?
            .iter()
            .find(|&name| UNIVERSE.systems.contains_key(name))
            .map(|name| UNIVERSE.systems.get(name).unwrap().clone())
    }

    pub fn get(id: &str) -> System {
        UNIVERSE.systems.get(id).unwrap().clone()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Universe {
    pub systems: HashMap<String, System>,
    pub system_aliases: HashMap<String, Vec<String>>,
    pub ships: Vec<String>,
    pub stop_words: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route {
    pub systems: Vec<System>,
    pub distance: u16,
    pub destination: System,
    pub source: System,
}

#[inline]
fn unwrap_node<T: Clone>(node: Option<&Option<T>>) -> Option<T> {
    Some(node.cloned()??)
}

#[inline]
pub fn ship_exists(name: &str) -> bool {
    UNIVERSE.ships.contains(&name.to_uppercase())
}

#[inline]
pub fn is_stop_word(word: &str) -> bool {
    UNIVERSE.stop_words.contains(&word.to_uppercase())
}

pub fn route(source: &System, destination: &System) -> Option<Route> {
    if source.neighbours.len() == 0 || destination.neighbours.len() == 0 {
        return None;
    }
    let (pred, succ, w) = walk(&source.id, &destination.id);
    let mut path: Vec<String> = vec![];
    let mut node = Some(w);

    while node.is_some() {
        let value = node.take().unwrap();
        node = unwrap_node(pred.get(&value));
        path.push(value);
    }

    path.reverse();

    node = Some(path.last().unwrap().to_string());
    while node.is_some() {
        let value = node.take().unwrap();
        node = unwrap_node(succ.get(&value));
        path.push(value);
    }

    let mut system_path = path
        .iter()
        .map(|id| System::get(id).clone())
        .collect::<Vec<_>>();

    system_path.dedup();
    let len = system_path.len() as u16;
    if len > 0 {
        Some(Route {
            systems: system_path,
            distance: len - 1,
            destination: destination.clone(),
            source: source.clone(),
        })
    } else {
        None
    }
}

type GraphPath = HashMap<String, Option<String>>;

fn walk(source: &str, destination: &str) -> (GraphPath, GraphPath, String) {
    let mut forward = vec![source.to_string()];
    let mut backward = vec![destination.to_string()];
    let mut pred: GraphPath = HashMap::new();
    let mut succ: GraphPath = HashMap::new();

    pred.insert(source.to_string(), None);
    succ.insert(destination.to_string(), None);

    if source == destination {
        return (pred, succ, source.to_string());
    }

    while forward.len() > 0 && backward.len() > 0 {
        if forward.len() <= backward.len() {
            let current = forward;
            forward = vec![];
            for v in current.iter() {
                for w in System::get(v).neighbours.iter() {
                    if pred.get(w).is_none() {
                        forward.push(w.to_string());
                        pred.insert(w.to_string(), Some(v.to_string()));
                    }
                    if succ.contains_key(w) {
                        return (pred, succ, w.to_string());
                    }
                }
            }
        } else {
            let current = backward;
            backward = vec![];

            for v in current.iter() {
                for w in System::get(v).neighbours.iter() {
                    if succ.get(w).is_none() {
                        backward.push(w.to_string());
                        succ.insert(w.to_string(), Some(v.to_string()));
                    }
                    if pred.contains_key(w) {
                        return (pred, succ, w.clone());
                    }
                }
            }
        }
    }

    return (HashMap::default(), HashMap::default(), String::default());
}
