//! definition of VoiceVox openapi schema section.
#![allow(dead_code)]

use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeStruct;

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
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HttpValidationError {
    pub detail: Vec<ValidationError>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ValidationError {
    ///Location
    pub loc: Vec<String>,
    ///Message
    pub msg: String,
    ///Error Type
    #[serde(rename = "type")]
    pub _type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccentPhrasesResponse {
    pub accent_phrases: Vec<AccentPhrase>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KanaParseError {
    pub text: String,
    pub error_name: String,
    pub error_args: String,
}

#[allow(non_snake_case, unused_variables)]
#[derive(Deserialize, Serialize, Debug)]
pub struct Preset {
    pub id: i32,
    pub name: String,
    pub speaker_uuid: String,
    pub style_id: i32,
    pub speedScale: f32,
    pub pitchScale: f32,
    pub intonationScale: f32,
    pub volumeScale: f32,
    pub prePhonemeLength: f32,
    pub postPhonemeLength: f32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Speaker {
    /// character name
    pub name: String,
    /// used to call SpeakerInfo.
    pub speaker_uuid: String,
    /// collection of emotion style.
    pub styles: Vec<SpeakerStyle>,
    pub version: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpeakerStyle {
    /// emotion style.
    pub name: String,
    /// style_id or speaker same as [StyleInfo.id]
    pub id: i32,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct StyleInfoRaw {
    pub(crate) id: i32,
    /// base64
    pub(crate) icon: String,
    /// base64
    pub(crate) voice_samples: Vec<String>,
}

impl TryFrom<StyleInfoRaw> for StyleInfo {
    type Error = TryFromRawError;
    fn try_from(raw: StyleInfoRaw) -> Result<Self, <Self as TryFrom<StyleInfoRaw>>::Error> {
        Ok(Self {
            id: raw.id,
            icon: base64::decode(raw.icon).map_err(|_| TryFromRawError::Base64Decode)?,
            voice_samples: raw
                .voice_samples
                .iter()
                .filter_map(|b64| base64::decode(b64).ok())
                .collect(),
        })
    }
}

impl From<StyleInfo> for StyleInfoRaw {
    fn from(mut si: StyleInfo) -> Self {
        Self {
            id: si.id,
            icon: base64::encode(si.icon),
            voice_samples: si
                .voice_samples
                .drain(..)
                .map(|bytes| base64::encode(bytes))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct SpeakerInfo {
    /// markdown format.
    pub policy: String,
    /// png file.
    pub portrait: Vec<u8>,
    pub style_infos: Vec<StyleInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SpeakerInfoRaw {
    pub(crate) policy: String,
    /// base64
    pub(crate) portrait: String,

    pub(crate) style_infos: Vec<StyleInfoRaw>,
}

pub enum TryFromRawError {
    Base64Decode,
}

impl TryFrom<SpeakerInfoRaw> for SpeakerInfo {
    type Error = TryFromRawError;
    fn try_from(mut raw: SpeakerInfoRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            policy: raw.policy,
            portrait: base64::decode(raw.portrait).map_err(|_| TryFromRawError::Base64Decode)?,
            style_infos: raw
                .style_infos
                .drain(..)
                .filter_map(|si_raw| si_raw.try_into().ok())
                .collect(),
        })
    }
}

impl From<SpeakerInfo> for SpeakerInfoRaw {
    fn from(mut si: SpeakerInfo) -> Self {
        Self {
            policy: si.policy,
            portrait: base64::encode(&si.portrait),
            style_infos: si.style_infos.drain(..).map(|si| si.into()).collect(),
        }
    }
}

#[derive(Debug)]
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
    pub cpu: bool,
    /// if enabled when Nvidia gpu + 3GiB VRam
    pub cuda: bool,
    /// if enabled when DirectML supported by engine.
    /// in engine 0.11.4 not supported.
    pub dml: Option<bool>,
}
