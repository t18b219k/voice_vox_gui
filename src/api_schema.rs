//! definition of VoiceVox openapi schema section.
//!
//!

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct AudioQuery {
    accent_phrases: Vec<AccentPhrase>,
    #[serde(rename = "speedScale")]
    speed_scale: f64,
    #[serde(rename = "pitchScale")]
    pitch_scale: f64,
    #[serde(rename = "intonationScale")]
    intonation_scale: f64,
    #[serde(rename = "volumeScale")]
    volume_scale: f64,
    #[serde(rename = "prePhonemeLength")]
    pre_phoneme_length: f64,
    #[serde(rename = "postPhonemeLength")]
    post_phoneme_length: f64,
    #[serde(rename = "outputSamplingRate")]
    output_sampling_rate: i64,
    #[serde(rename = "outputStereo")]
    output_stereo: bool,
    kana: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AccentPhrase {
    moras: Vec<Mora>,
    accent: i64,
    pause_mora: Option<Mora>,
    is_interrogative: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Mora {
    text: String,
    consonant: Option<String>,
    consonant_length: Option<f64>,
    vowel: String,
    vowel_length: f64,
    pitch: f64,
}

#[derive(Debug)]
pub struct GuidedSynthesisFormData {
    kana: String,
    speaker_id: i64,
    normalize: bool,
    audio_file: Vec<u8>,
    stereo: bool,
    sample_rate: i64,
    volume_scale: f64,
    pitch_scale: f64,
    speed_scale: f64,
}
impl GuidedSynthesisFormData {
    pub(crate) fn build_form(&self) -> [(&str, String); 9] {
        let mut ctr = String::new();
        let wav_container = unsafe { ctr.as_mut_vec() };
        wav_container.extend_from_slice(&self.audio_file);
        [
            ("kana", self.kana.clone()),
            ("speaker_id", self.speaker_id.to_string()),
            ("normalize", self.normalize.to_string()),
            ("audio_file", ctr),
            ("stereo", self.stereo.to_string()),
            ("sample_rate", self.sample_rate.to_string()),
            ("volume_scale", self.volume_scale.to_string()),
            ("pitch_scale", self.pitch_scale.to_string()),
            ("speed_scale", self.speed_scale.to_string()),
        ]
    }
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct HttpValidationError {
    Detail: Vec<ValidationError>,
}

#[derive(Deserialize, Debug)]
pub struct ValidationError {
    ///Location
    loc: Vec<String>,
    ///Message
    msg: String,
    ///Error Type
    #[serde(rename = "type")]
    _type: String,
}

#[derive(Deserialize, Debug)]
pub struct AccentPhrasesResponse {
    accent_phrases: Vec<AccentPhrase>,
}

#[derive(Deserialize, Debug)]
pub struct KanaParseError {
    text: String,
    error_name: String,
    error_args: String,
}
