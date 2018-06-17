use super::chat;
use super::intel;

#[derive(Debug)]
pub enum Event {
    PreviousMessage(chat::Message),
    NewMessage(chat::Message),
    IntelReport(intel::Message),
}
