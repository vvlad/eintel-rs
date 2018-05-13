#[macro_use] extern crate lazy_static;

#[derive(Clone, Hash, Eq, PartialEq, Debug)]
pub struct System{
    pub id: u32,
    pub name: &'static str,
    pub constelation: &'static str,
    pub region: &'static str,
    pub neighbours: Vec<usize>
}

mod data;
pub use data::*;

mod ships;
pub use ships::SHIPS;

pub fn ship_exists(ship_name: &str) -> bool {
    ships::SHIPS.contains_key(&ship_name.to_uppercase())
}
