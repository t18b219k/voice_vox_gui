//! definition of VoiceVox openapi path section
//!
//!

use crate::api_schema::{AccentPhrase, AccentPhrasesResponse, HttpValidationError, KanaParseError};
use crate::DEPTH;
use async_trait::async_trait;
use once_cell::race::OnceBox;
use reqwest::{Client, Error, Response, StatusCode};
use std::io::Read;
use trace::trace;

pub type CoreVersion = Option<String>;

static CLIENT: OnceBox<reqwest::Client> = once_cell::race::OnceBox::new();

pub fn init() -> Result<(), Box<Client>> {
    let client = reqwest::Client::new();
    CLIENT.set(Box::new(client))
}

/// # 音声合成用のクエリを作成する
///
/// クエリの初期値を得ます。ここで得られたクエリはそのまま音声合成に利用できます。各値の意味はSchemasを参照してください。
///

pub struct AudioQuery {
    pub(crate) text: String,
    pub(crate) speaker: i64,
    pub(crate) core_version: Option<String>,
}
#[async_trait]
impl Api for AudioQuery {
    type Response = Result<crate::api_schema::AudioQuery, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::post("http://localhost:50021/audio_query")
            .query("speaker", &self.speaker.to_string())
            .add_core_version(&self.core_version)
            .query("text", &self.text)
            .call()
            .map_err(|e| {
                log::error!("{:?}", e);
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => res
                        .into_json::<crate::api_schema::AudioQuery>()
                        .map_err(|e| APIError::Io(e)),

                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

///
/// # 音声合成用のクエリをプリセットを用いて作成する
/// クエリの初期値を得ます。ここで得られたクエリはそのまま音声合成に利用できます。各値の意味は`Schemas`を参照してください。
///
///
struct AudioQueryFromPreset {
    text: String,
    preset_id: i64,
    core_version: CoreVersion,
}
#[async_trait]
impl Api for AudioQueryFromPreset {
    type Response = Result<crate::api_schema::AudioQuery, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::post("http://localhost:50021/audio_query")
            .query("preset_id", &self.preset_id.to_string())
            .add_core_version(&self.core_version)
            .query("text", &self.text)
            .call()
            .map_err(|e| {
                log::error!("{:?}", e);
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => res
                        .into_json::<crate::api_schema::AudioQuery>()
                        .map_err(|e| APIError::Io(e)),
                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

#[async_trait]
pub trait Api {
    type Response;

    async fn call(&self) -> Self::Response;
}

/// # テキストからアクセント句を得る
/// テキストからアクセント句を得ます。
///
/// is_kanaが`true`のとき、テキストは次のようなAquesTalkライクな記法に従う読み仮名として処理されます。デフォルトは`false`です。
///
/// * 全てのカナはカタカナで記述される
/// * アクセント句は`/`または`、`で区切る。`、`で区切った場合に限り無音区間が挿入される。
/// * カナの手前に`_`を入れるとそのカナは無声化される
/// * アクセント位置を`'`で指定する。全てのアクセント句にはアクセント位置を1つ指定する必要がある。
/// * アクセント句末に`？`(全角)を入れることにより疑問文の発音ができる。
///
pub struct AccentPhrases {
    pub(crate) text: String,
    pub(crate) speaker: i64,
    pub(crate) is_kana: Option<bool>,
    pub(crate) core_version: CoreVersion,
}

#[derive(Debug)]
pub enum AccentPhrasesErrors {
    KanaParseError(KanaParseError),
    ApiError(APIError),
}
#[async_trait]
impl Api for AccentPhrases {
    type Response = Result<AccentPhrasesResponse, AccentPhrasesErrors>;

    async fn call(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/audio_query")
            .query("speaker", &self.speaker.to_string())
            .add_core_version(&self.core_version);
        if let Some(v) = &self.is_kana {
            query.query("is_kana", &v.to_string())
        } else {
            query
        }
        .query("text", &self.text)
        .call()
        .map_err(|e| {
            log::error!("{:?}", e);
            if let ureq::Error::Status(422, res) = e {
                AccentPhrasesErrors::ApiError(gen_http_validation_error(res))
            } else if let ureq::Error::Status(400, res) = e {
                match res
                    .into_json::<KanaParseError>()
                    .map_err(|e| APIError::Io(e))
                {
                    Ok(e) => AccentPhrasesErrors::KanaParseError(e),
                    Err(e) => AccentPhrasesErrors::ApiError(e),
                }
            } else {
                AccentPhrasesErrors::ApiError(APIError::Ureq(e))
            }
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => res
                    .into_json::<crate::api::AccentPhrasesResponse>()
                    .map_err(|e| AccentPhrasesErrors::ApiError(APIError::Io(e))),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::ApiError(APIError::Unknown))
                }
            }
        })
    }
}

///Create Accent Phrase from External Audio
///
/// Extracts f0 and aligned phonemes, calculates average f0 for every phoneme. Returns a list of AccentPhrase. This API works in the resolution of phonemes.
pub struct GuidedAccentPhrase {
    //in query
    core_version: CoreVersion,
    // in body
    text: String,
    speaker: i64,
    is_kana: bool,
    audio_file: String,
    normalize: bool,
}
#[async_trait]
impl Api for GuidedAccentPhrase {
    type Response = Result<Vec<AccentPhrase>, AccentPhrasesErrors>;

    async fn call(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/guided_accent_phrase");

        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .send_form(&[
            ("text", &self.text),
            ("speaker", &self.speaker.to_string()),
            ("is_kana", &self.is_kana.to_string()),
            ("audio_file", &self.audio_file),
            ("normalize", &self.normalize.to_string()),
        ])
        .map_err(|e| {
            log::error!("{:?}", e);
            AccentPhrasesErrors::ApiError(if let ureq::Error::Status(422, res) = e {
                gen_http_validation_error(res)
            } else {
                APIError::Ureq(e)
            })
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => res
                    .into_json::<Vec<AccentPhrase>>()
                    .map_err(|e| AccentPhrasesErrors::ApiError(APIError::Io(e))),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::ApiError(APIError::Unknown))
                }
            }
        })
    }
}

///アクセント句から音高を得る
pub struct MoraData {
    //in query
    speaker: i64,
    core_version: CoreVersion,
    //in body
    accent_phrases: Vec<AccentPhrase>,
}
#[async_trait]
impl Api for MoraData {
    type Response = Result<Vec<AccentPhrase>, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::post("http://localhost:50021/guided_accent_phrase")
            .query("speaker", &self.speaker.to_string())
            .add_core_version(&self.core_version)
            .send_json(&self.accent_phrases)
            .map_err(|e| {
                log::error!("{:?}", e);
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => res.into_json::<_>().map_err(|e| APIError::Io(e)),
                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

/// # アクセント句から音素長を得る
pub struct MoraLength {
    // in query.
    speaker: i64,
    core_version: CoreVersion,
    // in body.
    accent_phrases: Vec<AccentPhrase>,
}
#[async_trait]
impl Api for MoraLength {
    type Response = Result<Vec<AccentPhrase>, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::post("http://localhost:50021/mora_length")
            .query("speaker", &self.speaker.to_string())
            .add_core_version(&self.core_version)
            .send_json(&self.accent_phrases)
            .map_err(|e| {
                log::error!("{:?}", e);
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => res.into_json::<_>().map_err(|e| APIError::Io(e)),
                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

/// # アクセント句から音素長を得る
pub struct MoraPitch {
    // in query.
    speaker: i64,
    core_version: CoreVersion,
    // in body.
    accent_phrases: Vec<AccentPhrase>,
}
#[async_trait]
impl Api for MoraPitch {
    type Response = Result<Vec<AccentPhrase>, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::post("http://localhost:50021/mora_pitch")
            .query("speaker", &self.speaker.to_string())
            .add_core_version(&self.core_version)
            .send_json(&self.accent_phrases)
            .map_err(|e| {
                log::error!("{:?}", e);
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => res.into_json::<_>().map_err(|e| APIError::Io(e)),
                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

/// # 音声合成する
pub struct Synthesis {
    // in query
    pub(crate) speaker: i64,
    pub(crate) enable_interrogative_upspeak: Option<bool>,
    pub(crate) core_version: CoreVersion,
    // in body json.
    pub(crate) audio_query: crate::api_schema::AudioQuery,
}
#[async_trait]
impl Api for Synthesis {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/synthesis")
            .query("speaker", &self.speaker.to_string());
        if let Some(cv) = &self.enable_interrogative_upspeak {
            query.query("enable_interrogative_upspeak", &cv.to_string())
        } else {
            query
        }
        .add_core_version(&self.core_version)
        .send_json(&self.audio_query)
        .map_err(|e| {
            log::error!("{:?}", e);
            if let ureq::Error::Status(422, res) = e {
                gen_http_validation_error(res)
            } else {
                APIError::Ureq(e)
            }
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => {
                    let mut buffer = Vec::new();
                    res.into_reader()
                        .read_to_end(&mut buffer)
                        .map_err(|e| APIError::Io(e))?;
                    Ok(buffer)
                }
                x => {
                    log::error!("http status code {}", x);
                    Err(APIError::Unknown)
                }
            }
        })
    }
}

/// # 音声合成する（キャンセル可能）
pub struct CancellableSynthesis {
    // in query
    pub(crate) speaker: i64,
    pub(crate) enable_interrogative_upspeak: Option<bool>,
    pub(crate) core_version: CoreVersion,
    // in body json.
    pub(crate) audio_query: crate::api_schema::AudioQuery,
}
#[async_trait]
impl Api for CancellableSynthesis {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/cancellable_synthesis")
            .query("speaker", &self.speaker.to_string());
        if let Some(cv) = &self.enable_interrogative_upspeak {
            query.query("enable_interrogative_upspeak", &cv.to_string())
        } else {
            query
        }
        .add_core_version(&self.core_version)
        .send_json(&self.audio_query)
        .map_err(|e| {
            log::error!("{:?}", e);
            if let ureq::Error::Status(422, res) = e {
                gen_http_validation_error(res)
            } else {
                APIError::Ureq(e)
            }
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => {
                    let mut buffer = Vec::new();
                    res.into_reader()
                        .read_to_end(&mut buffer)
                        .map_err(|e| APIError::Io(e))?;
                    Ok(buffer)
                }

                x => {
                    log::error!("http status code {}", x);
                    Err(APIError::Unknown)
                }
            }
        })
    }
}

/// # まとめて音声合成する
///
/// 複数のwavがzipでまとめられて返されます.
pub struct MultiSynthesis {
    // in query
    pub(crate) speaker: i64,
    pub(crate) core_version: CoreVersion,
    // in body json.
    pub(crate) audio_query: Vec<crate::api_schema::AudioQuery>,
}
#[async_trait]
impl Api for MultiSynthesis {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::post("http://localhost:50021/multi_synthesis")
            .query("speaker", &self.speaker.to_string())
            .add_core_version(&self.core_version)
            .send_json(&self.audio_query)
            .map_err(|e| {
                log::error!("{:?}", e);
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => {
                        let mut buffer = Vec::new();
                        res.into_reader()
                            .read_to_end(&mut buffer)
                            .map_err(|e| APIError::Io(e))?;
                        Ok(buffer)
                    }
                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

/// # 2人の話者でモーフィングした音声を合成する
///
/// 指定された2人の話者で音声を合成、指定した割合でモーフィングした音声を得ます。 モーフィングの割合はmorph_rateで指定でき、0.0でベースの話者、1.0でターゲットの話者に近づきます。
pub struct SynthesisMorphing {
    // in query
    pub(crate) base_speaker: i64,
    pub(crate) target_speaker: i64,
    pub(crate) morph_rate: f64,
    pub(crate) core_version: CoreVersion,
    // in body json.
    pub(crate) audio_query: crate::api_schema::AudioQuery,
}
#[async_trait]
impl Api for SynthesisMorphing {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::post("http://localhost:50021/synthesis_morphing")
            .query("base_speaker", &self.base_speaker.to_string())
            .query("target_speaker", &self.target_speaker.to_string())
            .query("morph_rate", &self.morph_rate.to_string())
            .add_core_version(&self.core_version)
            .send_json(&self.audio_query)
            .map_err(|e| {
                log::error!("{:?}", e);
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => {
                        let mut buffer = Vec::new();
                        res.into_reader()
                            .read_to_end(&mut buffer)
                            .map_err(|e| APIError::Io(e))?;
                        Ok(buffer)
                    }

                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

/// # Audio synthesis guided by external audio and phonemes
///
/// Extracts and passes the f0 and aligned phonemes to engine. Returns the synthesized audio. This API works in the resolution of frame.
///
pub struct GuidedSynthesis {
    // in query
    pub(crate) core_version: CoreVersion,
    // in form.
    pub(crate) form_data: crate::api_schema::GuidedSynthesisFormData,
}
#[async_trait]
impl Api for GuidedSynthesis {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::post("http://localhost:50021/guided_synthesis")
            .add_core_version(&self.core_version)
            .send_form(
                &self
                    .form_data
                    .build_form()
                    .iter()
                    .map(|(k, v)| (*k, v.as_str()))
                    .collect::<Vec<(&str, &str)>>(),
            )
            .map_err(|e| {
                log::error!("{:?}", e);
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => {
                        let mut buffer = Vec::new();
                        res.into_reader()
                            .read_to_end(&mut buffer)
                            .map_err(|e| APIError::Io(e))?;
                        Ok(buffer)
                    }
                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

/// # base64エンコードされた複数のwavデータを一つに結合する
///
/// base64エンコードされたwavデータを一纏めにし、wavファイルで返します。
pub struct ConnectWaves {
    waves: Vec<Vec<u8>>,
}
#[async_trait]
impl Api for ConnectWaves {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        let mut buffer = Vec::new();
        for wave in &self.waves {
            buffer.push(base64::encode(wave));
        }

        ureq::post("http://localhost:50021/connect_waves")
            .send_json(buffer)
            .map_err(|e| {
                log::error!("{:?}", e);
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => {
                        let mut buffer = Vec::new();
                        res.into_reader()
                            .read_to_end(&mut buffer)
                            .map_err(|e| APIError::Io(e))?;
                        Ok(buffer)
                    }
                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

pub struct Presets;
#[async_trait]
impl Api for Presets {
    type Response = Result<Vec<crate::api_schema::Preset>, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::get("http://localhost:50021/presets")
            .call()
            .map_err(|e| APIError::Ureq(e))
            .and_then(|response| {
                response
                    .into_json::<Vec<crate::api_schema::Preset>>()
                    .map_err(|e| APIError::Io(e))
            })
    }
}

#[tokio::test]
async fn call_presets() {
    let presets = Presets;
    for preset in presets.call().await.unwrap() {
        println!("{:?}", preset);
    }
}

pub struct Version;
#[async_trait]
impl Api for Version {
    type Response = Result<Option<String>, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::get("http://localhost:50021/version")
            .call()
            .map_err(|e| APIError::Ureq(e))
            .and_then(|response| {
                response
                    .into_json::<Option<String>>()
                    .map_err(|e| APIError::Io(e))
            })
    }
}

#[tokio::test]
async fn call_version() {
    let version = Version;
    println!("{:?}", version.call().await.unwrap());
}

pub struct CoreVersions;
#[async_trait]
impl Api for CoreVersions {
    type Response = Result<Vec<String>, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::get("http://localhost:50021/core_versions")
            .call()
            .map_err(|e| APIError::Ureq(e))
            .and_then(|response| {
                response
                    .into_json::<Vec<String>>()
                    .map_err(|e| APIError::Io(e))
            })
    }
}

#[tokio::test]
async fn call_core_versions() {
    let version = CoreVersions;
    println!("{:?}", version.call().await.unwrap());
}

pub struct Speakers {
    pub(crate) core_version: CoreVersion,
}
#[async_trait]
impl Api for Speakers {
    type Response = Result<Vec<crate::api_schema::Speaker>, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::get("http://localhost:50021/speakers")
            .add_core_version(&self.core_version)
            .call()
            .map_err(|e| {
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => res
                        .into_json::<Vec<crate::api_schema::Speaker>>()
                        .map_err(|e| APIError::Io(e)),

                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

#[tokio::test]
async fn call_speakers() {
    let speakers = Speakers { core_version: None };
    println!("{:?}", speakers.call().await.unwrap());
}

pub struct SpeakerInfo {
    pub(crate) speaker_uuid: String,
    pub(crate) core_version: CoreVersion,
}
#[async_trait]
impl Api for SpeakerInfo {
    type Response = Result<crate::api_schema::SpeakerInfo, APIError>;

    async fn call(&self) -> Self::Response {
        ureq::get("http://localhost:50021/speaker_info")
            .query("speaker_uuid", &self.speaker_uuid)
            .add_core_version(&self.core_version)
            .call()
            .map_err(|e| {
                log::error!("{:?}", e);
                if let ureq::Error::Status(422, res) = e {
                    gen_http_validation_error(res)
                } else {
                    APIError::Ureq(e)
                }
            })
            .and_then(|res| {
                let status = res.status();
                log::debug!("{}", status);
                match status {
                    200 => res
                        .into_json::<crate::api_schema::SpeakerInfoRaw>()
                        .map_err(|e| APIError::Io(e))
                        .map(|raw| crate::api_schema::SpeakerInfo {
                            policy: raw.policy.clone(),
                            portrait: base64::decode(&raw.portrait).unwrap_or_default(),
                            style_infos: raw
                                .style_infos
                                .iter()
                                .map(|raw| crate::api_schema::StyleInfo {
                                    id: raw.id,
                                    icon: base64::decode(&raw.icon).unwrap_or_default(),
                                    voice_samples: raw
                                        .voice_samples
                                        .iter()
                                        .map(|raw| base64::decode(raw).unwrap_or_default())
                                        .collect(),
                                })
                                .collect(),
                        }),
                    x => {
                        log::error!("http status code {}", x);
                        Err(APIError::Unknown)
                    }
                }
            })
    }
}

#[tokio::test]
async fn call_speaker_info() {
    let speakers = Speakers { core_version: None };
    let speakers = speakers.call().await.unwrap();
    let info = SpeakerInfo {
        speaker_uuid: speakers[0].speaker_uuid.clone(),
        core_version: None,
    };
    println!("{:?}", info.call().await);
}

pub struct SupportedDevices {
    core_version: CoreVersion,
}
#[async_trait]
impl Api for SupportedDevices {
    type Response = Result<crate::api_schema::SupportedDevices, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .get("http://localhost:50021/supported_devices")
            .add_core_version(&self.core_version)
            .build()
            .unwrap();
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<crate::api_schema::HttpValidationError>().await?,
            )),
            StatusCode::OK => Ok(res.json::<crate::api_schema::SupportedDevices>().await?),
            x => Err(x.into()),
        }
    }
}

use tokio::test;


#[tokio::test]
async fn call_supported_devices() {
    init();
    let supported_devices = SupportedDevices { core_version: None };
    println!("{:?}", supported_devices.call().await.unwrap());
}

fn gen_http_validation_error(res: ureq::Response) -> APIError {
    match res.into_json::<HttpValidationError>() {
        Ok(error_detail) => APIError::Validation(error_detail),
        Err(e) => APIError::Io(e),
    }
}

pub trait AddCoreVersion {
    fn add_core_version(self, core_version: &CoreVersion) -> Self;
}

impl AddCoreVersion for ureq::Request {
    fn add_core_version(self, core_version: &CoreVersion) -> Self {
        if let Some(cv) = &core_version {
            self.query("core_version", cv)
        } else {
            self
        }
    }
}

impl AddCoreVersion for reqwest::RequestBuilder {
    fn add_core_version(self, core_version: &CoreVersion) -> Self {
        if let Some(cv) = &core_version {
            self.query(&("core_version", cv))
        }else{
            self
        }
    }
}

#[derive(Debug)]
pub enum APIError {
    Validation(HttpValidationError),
    Io(std::io::Error),
    Ureq(ureq::Error),
    Unknown,
}

#[derive(Debug)]
pub enum APIErrorReqwest {
    Validation(HttpValidationError),
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    Unknown,
}

impl From<reqwest::Error> for APIErrorReqwest {
    fn from(e: Error) -> Self {
        APIErrorReqwest::Reqwest(e)
    }
}

impl Into<APIErrorReqwest> for std::io::Error {
    fn into(self) -> APIErrorReqwest {
        APIErrorReqwest::Io(self)
    }
}
impl From<StatusCode> for APIErrorReqwest {
    fn from(_: StatusCode) -> Self {
        APIErrorReqwest::Unknown
    }
}
