extern crate punkt;
extern crate rusoto_core;
extern crate rusoto_polly;
extern crate xml;

use self::punkt::params::Standard;
use self::punkt::{SentenceTokenizer, TrainingData};
use self::rusoto_core::Region;
use self::rusoto_polly::{Polly, PollyClient, SynthesizeSpeechInput};
use self::xml::writer::{EmitterConfig, XmlEvent};

pub struct TTSService {
    aws: PollyClient,
    rate: &'static str,
    brain: TrainingData,
}

const SPEAK_TAG: &'static str = "speak";
const PROSODY_TAG: &'static str = "prosody";
const MAX_LENGTH: usize = 1500;

impl TTSService {
    pub fn new() -> TTSService {
        TTSService {
            aws: PollyClient::simple(Region::EuCentral1),
            rate: "fast",
            brain: TrainingData::english(),
        }
    }

    pub fn synthesize(&self, text: &str) -> Option<Vec<u8>> {
        let mut input = SynthesizeSpeechInput::default();
        input.voice_id = "Salli".to_string();
        input.output_format = "ogg_vorbis".to_string();
        input.text_type = Some("ssml".to_string());
        let mut parts: Vec<String> = vec![];

        for sentence in SentenceTokenizer::<Standard>::new(text, &self.brain) {
            let mut sentence = str::replace(sentence.trim(), "\n\n", "\n");
            while sentence.len() > MAX_LENGTH {
                parts.push(sentence[0..MAX_LENGTH].to_owned());
                sentence = sentence[MAX_LENGTH..].to_owned();
            }
            parts.push(sentence);
        }

        let mut buf = vec![];

        for part in parts {
            input.text = self.to_xml(part);
            let output = self.aws.synthesize_speech(&input.clone()).sync().ok()?;
            buf.append(&mut output.audio_stream?)
        }
        Some(buf)
    }

    fn to_xml(&self, text: String) -> String {
        let mut bytes = Vec::new();
        {
            let mut writer = EmitterConfig::new()
                .perform_indent(true)
                .create_writer(&mut bytes);

            writer.write(XmlEvent::start_element(SPEAK_TAG)).is_ok();
            writer
                .write(XmlEvent::start_element(PROSODY_TAG).attr("rate", self.rate))
                .is_ok();
            writer.write(XmlEvent::characters(&text)).is_ok();
            writer.write(XmlEvent::end_element()).is_ok();
            writer.write(XmlEvent::end_element()).is_ok();
        }
        String::from_utf8(bytes).unwrap_or_default()
    }
}
