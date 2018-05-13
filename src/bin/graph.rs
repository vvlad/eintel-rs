extern crate eve_universe;
extern crate algorithms;
extern crate eintel;

use std::collections::HashMap;

use eve_universe::{SYSTEMS, SYSTEM_NAMES, System};

pub struct Graph;

type GraphPath = HashMap<usize, Option<usize>>;

impl Graph {
    pub fn new() -> Self {
        Graph 
    }

    pub fn route(&self, source: usize, destination: usize) -> Vec<System> {
        let (pred, succ, w) = self.walk(source, destination);
        let mut path : Vec<usize> = vec![];
        let mut node = Some(w);

        println!("f:{} b:{}", pred.len(), succ.len());
        while node.is_some() {
            let value = node.take().unwrap();
            path.push(value);
            node = pred.get(&value).cloned().unwrap();
        }

        path.reverse();
        
        node = Some(*path.last().unwrap());
        while node.is_some() {
            let value = node.take().unwrap();
            path.push(value);
            node = succ.get(&value).cloned().unwrap();
        }

        path.iter().map( |id| SYSTEMS[id].clone() ).collect::<Vec<_>>()

    }

    pub fn walk(&self, source: usize, destination: usize) -> (GraphPath, GraphPath, usize) {
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
                    for w in self.succ(*v){
                        if pred.get(&w).is_none() {
                            forward.push(w);
                            pred.insert(w,Some(*v));
                        }
                        if succ.contains_key(&w) {
                            return (pred, succ, w);
                        }
                    }
                }
            }else{
                let current = backward;
                backward = vec![];

                for v in current.iter() {
                    for w in self.pred(*v){
                        if succ.get(&w).is_none() {
                            backward.push(w);
                            succ.insert(w,Some(*v));
                        }
                        if pred.contains_key(&w) {
                            return (pred, succ, w);
                        }
                    }
                }
            }
        };

        return (HashMap::default(), HashMap::default(), 0)
    }

    fn succ(&self, node: usize) -> Vec<usize> {
        SYSTEMS[&node].neighbours.clone()
    }

    fn pred(&self, node: usize) -> Vec<usize> {
        SYSTEMS[&node].neighbours.clone()
    }
}

pub fn main() {
    let start = &SYSTEMS[&SYSTEM_NAMES["O-CNPR"].first().unwrap()];
    let stop = &SYSTEMS[&SYSTEM_NAMES["8S28-3"].first().unwrap()];
    match eintel::universe::route(start, stop ) {
        Some(route) => { println!("{:?}", route.systems.iter().map( |s| s.name ).collect::<Vec<_>>()) },
        None => { println!("no route found") }
    }

}
