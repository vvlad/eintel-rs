use super::audio_notification::AudioNotification;
use super::desktop_notification::desktop_notification;
use super::intel;
use super::Notification;

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

pub struct DebouncedMessage(intel::Message);

impl DebouncedMessage {
    pub fn new(message: intel::Message) -> DebouncedMessage {
        DebouncedMessage(message)
    }
}

impl Hash for DebouncedMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.message.hash(state);
    }
}

impl PartialEq for DebouncedMessage {
    fn eq(&self, other: &DebouncedMessage) -> bool {
        self.0.message == other.0.message
    }
}

impl Eq for DebouncedMessage {}

pub enum DebounceMessages {
    Intel(intel::Message),
    Tick,
}

impl DebounceMessages {
    pub fn channel() -> mpsc::Sender<DebounceMessages> {
        let (tx, rx) = mpsc::channel();
        let tick = tx.clone();
        let audio = AudioNotification::new();
        let queue: Arc<Mutex<HashSet<DebouncedMessage>>> = Arc::new(Mutex::new(HashSet::new()));
        let q = queue.clone();
        thread::spawn(move || loop {
            thread::sleep(time::Duration::from_millis(200));
            let q = queue.lock().unwrap();
            if q.len() > 0 {
                tick.send(DebounceMessages::Tick).is_ok();
            }
        });

        let queue = q;
        thread::spawn(move || loop {
            match rx.recv() {
                Ok(DebounceMessages::Intel(intel)) => {
                    let debounced = DebouncedMessage::new(intel);
                    {
                        let mut q = queue.lock().unwrap();
                        let message = if let Some(existing) = q.take(&debounced) {
                            if existing.0.route.distance > debounced.0.route.distance {
                                existing
                            } else {
                                debounced
                            }
                        } else {
                            debounced
                        };
                        q.insert(message);
                    }
                }
                Ok(DebounceMessages::Tick) => {
                    let mut q = queue.lock().unwrap();
                    for message in q.drain() {
                        let notification = Notification::from(message.0.clone());
                        match notification {
                            Notification::Sound(text) => {
                                audio.notify(&text);
                            }
                            Notification::Desktop(text) => {
                                desktop_notification(&text, &message.0.message);
                            }
                            Notification::None => {}
                        };
                    }
                }
                Err(error) => {
                    error!("{:?}: {}", error, error);
                    return;
                }
            };
        });

        tx
    }
}
