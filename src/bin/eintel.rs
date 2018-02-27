

extern crate eintel;


pub fn main() {
    let chat = eintel::ChatBuilder::new()
        .directory("/home/vvlad/Documents/EVE/logs/Chatlogs")
        .channel("GotG Home Intel")
        .channel("Derzerek")
        .player("Derzerek")
        .build();

    let messages = chat.start();

    loop {
        match messages.recv() {
            Ok(message) => println!("Got message: {:?}", message),
            Err(error) => println!("Got error: {:?}", error)
        };
    }

}
