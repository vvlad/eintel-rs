mod channel;
pub mod chat;


#[derive(Debug)]
pub struct Message {
    pub player: String,
    pub message: String,
    pub channel: String,
    pub sender: String
}

#[derive(Debug)]
pub struct Location {
    pub system: String,
    pub player: String 
}


#[derive(Debug)]
pub enum Event{
    ChatMessage(Message),
    Location(Location)
}

pub use self::chat::Chat;
pub use self::channel::Channel;
