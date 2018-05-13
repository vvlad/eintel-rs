extern crate eintel;

use eintel::chat;
use eintel::intel;
use std::sync::mpsc;
use std::thread;

pub fn main() {
    let (chat_channel, chat_messages) = mpsc::channel();
    let mut intel = intel::Intel::new();
    let mut chat = chat::Chat::new(chat_channel);

    chat.channel("Derzerek");
    chat.channel("GotG Home Intel");
    chat.player("Derzerek");

    let (intel_channel, intel_messages) = mpsc::channel();

    thread::spawn(move|| intel.run(intel_messages));
    thread::spawn(move|| chat.run());

    loop {
        match chat_messages.recv() {
            Ok(event) => { intel_channel.send(event).is_ok(); },
            Err(_) => { println!("Test"); }
        }
    }
}
