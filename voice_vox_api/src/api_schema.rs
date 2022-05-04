//! definition of VoiceVox openapi schema section.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
#[allow(non_snake_case)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AudioQuery {
    pub accent_phrases: Vec<AccentPhrase>,
    pub speedScale: f32,
    pub pitchScale: f32,
    pub intonationScale: f32,
    pub volumeScale: f32,
    pub prePhonemeLength: f32,
    pub postPhonemeLength: f32,
    pub outputSamplingRate: i32,
    pub outputStereo: bool,
    pub kana: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccentPhrase {
    pub moras: Vec<Mora>,
    pub accent: i32,
    pub pause_mora: Option<Mora>,
    pub is_interrogative: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mora {
    pub text: String,
    pub consonant: Option<String>,
    pub consonant_length: Option<f32>,
    pub vowel: String,
    pub vowel_length: f32,
    pub pitch: f32,
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
    /// character name
    pub name: String,
    /// used to call SpeakerInfo.
    pub speaker_uuid: String,
    /// collection of emotion style.
    pub styles: Vec<SpeakerStyle>,
    pub version: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct SpeakerStyle {
    /// emotion style.
    pub name: String,
    /// style_id or speaker same as [StyleInfo.id]
    pub id: i32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SpeakerInfoRaw {
    pub(crate) policy: String,
    /// base64
    pub(crate) portrait: String,

    pub(crate) style_infos: Vec<StyleInfoRaw>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct StyleInfoRaw {
    pub(crate) id: i32,
    /// base64
    pub(crate) icon: String,
    /// base64
    pub(crate) voice_samples: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpeakerInfo {
    /// markdown format.
    pub policy: String,
    /// png file.
    pub portrait: Vec<u8>,
    pub style_infos: Vec<StyleInfo>,
}

#[derive(Deserialize, Debug)]
pub struct StyleInfo {
    /// style_id or speaker. you can put into below API fields.
    /// * AudioQuery.speaker
    /// * AccentPhrases.speaker
    /// * MoraData.speaker
    /// * MoraPitch.speaker
    /// * MoraLength.speaker
    /// * Synthesis.speaker
    /// * CancellableSynthesis.speaker
    /// * MultiSynthesis.speaker
    /// * SynthesisMorphing.base_speaker
    /// * SynthesisMorphing.target_speaker
    pub id: i32,
    ///png file
    pub icon: Vec<u8>,
    ///wav file
    pub voice_samples: Vec<Vec<u8>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SupportedDevices {
    /// always support
    cpu: bool,
    /// if enabled when Nvidia gpu + 3GiB VRam
    cuda: bool,
    /// if enabled when DirectML supported by engine.
    /// in engine 0.11.4 not supported.
    dml: Option<bool>,
}
