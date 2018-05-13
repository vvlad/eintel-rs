extern crate rusqlite;
extern crate eve_universe;

use std::collections::{HashSet, HashMap};
use self::eve_universe::*;

pub fn is_system_name(token: &String) -> bool {
    SYSTEM_NAMES.get(token).is_some()
}

pub fn find_systems(names: &Vec<String>) -> Vec<System> {
    
    names.iter()
      .map(|name| SYSTEM_NAMES.get(&name.to_uppercase()) )
      .filter( |result| result.is_some() )
      .flat_map( |result| result.unwrap() )
      .collect::<HashSet<_>>()
      .iter()
      .map( |index| SYSTEMS[index].clone() )
      .collect::<Vec<_>>()

}

pub fn find(name: String) -> System {
    SYSTEM_NAMES.get(&name)
      .unwrap()
      .iter()
      .map( |index| SYSTEMS[index].clone() )
      .filter( |system| system.name == name ).nth(0).unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Route {
    pub systems: Vec<System>,
    pub distance: i16,
    pub destination: System,
    pub source: System
}

fn unwrap_node<T: Clone>(node: Option<&Option<T>> ) -> Option<T> {
    if let Some(value) = node.cloned() {
        if let Some(node) = value {
            Some(node)
        }else{
            None
        }
    } else {
        None
    }
}
pub fn route(source: &System, destination: &System) -> Option<Route> {
    if source.neighbours.len() == 0 || destination.neighbours.len() == 0 {
        return None
    }
    let (pred, succ, w) = walk(source.id as usize, destination.id as usize);
    let mut path : Vec<usize> = vec![];
    let mut node = Some(w);

    while node.is_some() {
        let value = node.take().unwrap();
        path.push(value);
        node = unwrap_node(pred.get(&value))
    }

    path.reverse();
    
    node = Some(*path.last().unwrap());
    while node.is_some() {
        let value = node.take().unwrap();
        path.push(value);
        node = unwrap_node(succ.get(&value));
    }

    let mut system_path = path.iter().map( |id| SYSTEMS[id].clone() ).collect::<Vec<_>>();
    system_path.dedup();
    let len = system_path.len() as i16;
    if system_path.len() > 0 {
        Some(Route{
            systems: system_path,
            distance: len,
            destination: destination.clone(),
            source: source.clone()
        })
    }else{
        None 
    }
}

type GraphPath = HashMap<usize, Option<usize>>;

pub fn walk(source: usize, destination: usize) -> (GraphPath, GraphPath, usize) {
    let mut forward = vec![source];
    let mut backward = vec![destination];
    let mut pred: GraphPath = HashMap::new();
    let mut succ: GraphPath = HashMap::new();
 
    pred.insert(source, None);
    succ.insert(destination, None);

    if source == destination {
        return (pred, succ, source);
    }

    while forward.len() > 0 && backward.len() > 0 {
        if forward.len() <= backward.len() {
            let current = forward;
            forward = vec![];
            for v in current.iter() {
                for w in SYSTEMS[v].neighbours.iter() {
                    if pred.get(w).is_none() {
                        forward.push(*w);
                        pred.insert(*w,Some(*v));
                    }
                    if succ.contains_key(&w) {
                        return (pred, succ, *w);
                    }
                }
            }
        }else{
            let current = backward;
            backward = vec![];

            for v in current.iter() {
                for w in SYSTEMS[v].neighbours.iter() {
                    if succ.get(w).is_none() {
                        backward.push(*w);
                        succ.insert(*w,Some(*v));
                    }
                    if pred.contains_key(w) {
                        return (pred, succ, *w);
                    }
                }
            }
        }
    };

    return (HashMap::default(), HashMap::default(), 0)
}

