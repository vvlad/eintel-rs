use std;

// use bincode;
use log;
// use reqwest;
// use serde_json;
// use time;
use super::chat::channel;
use super::events::Event;
use super::notifications::debounced_message;
use chrono;
use notify;
use std::sync;
use std::sync::mpsc;

error_chain!{
    foreign_links {
        LogError(log::SetLoggerError);
        IOError(std::io::Error);
        TimeFormatError(chrono::ParseError);
        ChatMessageDeliveryError(mpsc::SendError<Event>);
        ChatMessageReceivingError(mpsc::RecvError);
        FSError(notify::Error);
        ChannelUnwrapError(sync::PoisonError<channel::Channel>);
        NotificationDebounceError(mpsc::SendError<debounced_message::DebounceMessages>);
    }
}
