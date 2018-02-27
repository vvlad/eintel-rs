extern crate eintel;

use eintel::chat; 
use eintel::intel;

use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx_messages, chat_messages) = mpsc::channel();

    let mut chat = chat::Chat::new("/home/vvlad/Documents/EVE/logs/Chatlogs", tx_messages); 
    chat.watch_channel("GotG Home Intel");
    chat.watch_channel("Derzerek");
    chat.watch_player("Derzerek");

    thread::spawn(move || {
        loop {
            match chat_messages.recv() {
                Ok(message) => println!("{:?}", message),
                Err(e) => println!("main#error: {:?}", e)
            }
        }
    });

    chat.start();
}
