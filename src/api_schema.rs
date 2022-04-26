//! definition of VoiceVox openapi schema section.
//!
//!

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AudioQuery {
    pub accent_phrases: Vec<AccentPhrase>,
    #[serde(rename = "speedScale")]
    pub speed_scale: f32,
    #[serde(rename = "pitchScale")]
    pub pitch_scale: f32,
    #[serde(rename = "intonationScale")]
    pub intonation_scale: f32,
    #[serde(rename = "volumeScale")]
    pub volume_scale: f32,
    #[serde(rename = "prePhonemeLength")]
    pub pre_phoneme_length: f32,
    #[serde(rename = "postPhonemeLength")]
    pub post_phoneme_length: f32,
    #[serde(rename = "outputSamplingRate")]
    pub output_sampling_rate: i32,
    #[serde(rename = "outputStereo")]
    pub output_stereo: bool,
    pub kana: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccentPhrase {
    moras: Vec<Mora>,
    accent: i32,
    pause_mora: Option<Mora>,
    is_interrogative: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mora {
    text: String,
    consonant: Option<String>,
    consonant_length: Option<f32>,
    vowel: String,
    vowel_length: f32,
    pitch: f32,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone)]
pub struct HttpValidationError {
    detail: Vec<ValidationError>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ValidationError {
    ///Location
    loc: Vec<String>,
    ///Message
    msg: String,
    ///Error Type
    #[serde(rename = "type")]
    _type: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AccentPhrasesResponse {
    accent_phrases: Vec<AccentPhrase>,
}

#[derive(Deserialize, Debug)]
pub struct KanaParseError {
    text: String,
    error_name: String,
    error_args: String,
}

#[allow(non_snake_case, unused_variables)]
#[derive(Deserialize, Debug)]
pub struct Preset {
    id: i32,
    name: String,
    speaker_uuid: String,
    style_id: i32,
    speedScale: f32,
    pitchScale: f32,
    intonationScale: f32,
    volumeScale: f32,
    prePhonemeLength: f32,
    postPhonemeLength: f32,
}

#[derive(Deserialize, Debug)]
pub struct Speaker {
    pub name: String,
    pub(crate) speaker_uuid: String,
    pub styles: Vec<SpeakerStyle>,
    pub version: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SpeakerStyle {
    pub(crate) name: String,
    pub(crate) id: i32,
}

#[derive(Debug, Deserialize)]
pub struct SpeakerInfoRaw {
    pub policy: String,
    /// base64
    pub portrait: String,

    pub style_infos: Vec<StyleInfoRaw>,
}

#[derive(Deserialize, Debug)]
pub struct StyleInfoRaw {
    pub id: i32,
    /// base64
    pub icon: String,
    /// base64
    pub voice_samples: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpeakerInfo {
    /// markdown format.
    pub(crate) policy: String,
    /// base64 encoded png file.
    pub(crate) portrait: Vec<u8>,
    pub(crate) style_infos: Vec<StyleInfo>,
}

#[derive(Deserialize, Debug)]
pub struct StyleInfo {
    pub(crate) id: i32,
    ///png file
    pub(crate) icon: Vec<u8>,
    ///wav file
    pub(crate) voice_samples: Vec<Vec<u8>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SupportedDevices {
    cpu: bool,
    cuda: bool,
    dml: Option<bool>,
}
