extern crate eintel;

use eintel::universe::{route, ship_exists, System};

pub fn main() {
    let jita = System::find("Jita").unwrap();
    let agg = System::find("AGG").unwrap();

    let route = route(&jita, &agg).unwrap();
    println!(
        "{:?}",
        route
            .systems
            .iter()
            .map(|system| system.name.clone())
            .collect::<Vec<_>>()
    );
    println!("{:?}", ship_exists("Tengu"));
}
