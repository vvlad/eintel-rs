extern crate eintel;

use eintel::chat::chat::change_info;
use eintel::chat::{Event, Location, Channel};
use eintel::intel;
use std::sync::mpsc;
use std::thread;

extern crate regex;

use regex::Regex;

pub fn main(){
	let (channel_tx, channel_rx) = mpsc::channel();
    let mut intel = intel::Intel::new();
    let (intel_channel, intel_messages) = mpsc::channel();

    intel_channel.send(Event::Location(Location{
        system: "8S28-3".to_owned(),
        player: "Derzerek".to_owned()
    })).is_ok();

    thread::spawn(move|| intel.run(intel_messages));
   
	thread::spawn(move|| { 
		let regex = Regex::new(r"(?i)(LOCAL|DERZEREK|GOTG HOME INTEL)_(\d+)_(\d+).TXT$").unwrap();
		let change = change_info("/home/vvlad/Documents/EVE/logs/Chatlogs/GotG Home Intel_20180310_134408.txt", &regex);

		let mut channel = Channel::new(change.name, change.player, channel_tx);	
		let lines = change.content.clone().lines().map(|line| line.to_owned()).collect::<Vec<_>>();

        println!("Total lines: {}", lines.len());
        let mut content = String::new();
        
        for line in lines.iter() {
           content = format!("{}\n{}",content, line); 
           channel.update(content.clone(), change.version);
        }
	});


    loop {
        match channel_rx.recv() {
            Ok(event) => { intel_channel.send(event).is_ok(); }, 
            Err(error) => { println!("error@main: {:?} {}", error, error); return}
        };
    }


}
