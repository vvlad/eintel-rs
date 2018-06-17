use notifications::tts::TTSService;
extern crate alto;
extern crate lewton;

use self::alto::{Alto, Mono, Source};
use self::lewton::inside_ogg::OggStreamReader;
use std::io::Cursor;
use std::sync::mpsc;
use std::thread;
use std::time;

#[derive(Debug)]
struct Sound {
    bytes: Vec<i16>,
    rate: i32,
    channels: i32,
}

impl Sound {
    fn new(rate: i32, channels: i32) -> Self {
        Sound {
            bytes: vec![],
            channels: channels,
            rate: rate,
        }
    }
}

pub struct AudioNotification {
    tts: TTSService,
    sounds: mpsc::Sender<Sound>,
}

impl AudioNotification {
    pub fn new() -> AudioNotification {
        AudioNotification {
            tts: TTSService::new(),
            sounds: sound_loop(),
        }
    }

    pub fn notify(&self, text: &str) -> Option<()> {
        let sound = {
            let buf = self.tts.synthesize(text)?;

            let mut ogg = OggStreamReader::new(Cursor::new(buf)).ok()?;
            let rate = ogg.ident_hdr.audio_sample_rate as i32;
            let channels = ogg.ident_hdr.audio_channels as i32;
            let mut sound = Sound::new(rate, channels);

            while let Ok(Some(mut samples)) = ogg.read_dec_packet_itl() {
                sound.bytes.append(&mut samples);
            }
            sound
        };
        self.sounds.send(sound).ok()
    }
}

fn sound_loop() -> mpsc::Sender<Sound> {
    let (tx, rx) = mpsc::channel::<Sound>();
    let context = sound_context();

    thread::spawn(move || loop {
        let mut stream = context
            .new_streaming_source()
            .expect("cloud not create streaming src");
        let mut play_time = 0.0 as f32;

        if let Ok(sound) = rx.try_recv() {
            let buf = context
                .new_buffer::<Mono<i16>, _>(&sound.bytes, sound.rate)
                .unwrap();

            stream.queue_buffer(buf).is_ok();
            play_time += sound.bytes.len() as f32 / sound.rate as f32;
            stream.play();
        }

        let sleep_time = if play_time > 0.0 {
            time::Duration::from_millis((play_time * 1000.0) as u64)
                + time::Duration::from_millis(10)
        } else {
            time::Duration::from_millis(50)
        };

        thread::sleep(sleep_time);
    });

    tx
}

fn sound_context() -> alto::Context {
    let al = Alto::load_default().expect("Could not load alto");
    let device = al.open(None).expect("Could not open device");
    device.new_context(None).expect("Could not create context")
}
