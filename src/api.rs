//! definition of VoiceVox openapi path section
//!
//!

use crate::api_schema::{AccentPhrase, AccentPhrasesResponse, HttpValidationError, KanaParseError};
use crate::DEPTH;

use std::io::Read;
use trace::trace;

/// # 音声合成用のクエリを作成する
///
/// クエリの初期値を得ます。ここで得られたクエリはそのまま音声合成に利用できます。各値の意味はSchemasを参照してください。
///

pub struct AudioQuery {
    pub(crate) text: String,
    pub(crate) speaker: i64,
    pub(crate) core_version: Option<String>,
}

#[derive(Debug)]
pub enum AudioQueryErrors {
    Validation(HttpValidationError),
    CantParseBySerde,
    Unknown,
    IO,
}

impl Api for AudioQuery {
    type Response = Result<crate::api_schema::AudioQuery, AudioQueryErrors>;
    #[trace]
    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/audio_query")
            .query("speaker", &self.speaker.to_string());
        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .query("text", &self.text)
        .call()
        .map_err(|e| {
            log::error!("{:?}", e);
            AudioQueryErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AudioQueryErrors::IO
                    })
                    .and_then(|txt| {
                        log::debug!("{}", txt);
                        serde_json::from_str::<crate::api_schema::AudioQuery>(&txt).map_err(|err| {
                            log::error!("{}", err);
                            AudioQueryErrors::CantParseBySerde
                        })
                    }),
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AudioQueryErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AudioQueryErrors::Validation(e)),
                        _ => Err(AudioQueryErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AudioQueryErrors::Unknown)
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
    core_version: Option<String>,
}

impl AudioQueryFromPreset {}

impl Api for AudioQueryFromPreset {
    type Response = Result<crate::api_schema::AudioQuery, AudioQueryErrors>;
    #[trace]
    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/audio_query")
            .query("preset_id", &self.preset_id.to_string());
        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .query("text", &self.text)
        .call()
        .map_err(|e| {
            log::error!("{:?}", e);
            AudioQueryErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AudioQueryErrors::IO
                    })
                    .and_then(|txt| {
                        log::debug!("{}", txt);
                        serde_json::from_str::<crate::api_schema::AudioQuery>(&txt).map_err(|err| {
                            log::error!("{}", err);
                            AudioQueryErrors::CantParseBySerde
                        })
                    }),
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AudioQueryErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AudioQueryErrors::Validation(e)),
                        _ => Err(AudioQueryErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AudioQueryErrors::Unknown)
                }
            }
        })
    }
}

pub trait Api {
    type Response;
    fn post(&self) -> Self::Response;
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
    pub(crate) core_version: Option<String>,
}

#[derive(Debug)]
pub enum AccentPhrasesErrors {
    Validation(HttpValidationError),
    KanaParseError(KanaParseError),
    CantParseBySerde,
    Unknown,
    IO,
}

impl Api for AccentPhrases {
    type Response = Result<AccentPhrasesResponse, AccentPhrasesErrors>;
    #[trace]
    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/audio_query")
            .query("speaker", &self.speaker.to_string());
        let query = if let Some(v) = &self.is_kana {
            query.query("is_kana", &v.to_string())
        } else {
            query
        };
        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .query("text", &self.text)
        .call()
        .map_err(|e| {
            log::error!("{:?}", e);
            AccentPhrasesErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|txt| {
                        log::debug!("{}", txt);
                        serde_json::from_str::<AccentPhrasesResponse>(&txt).map_err(|err| {
                            log::error!("{}", err);
                            AccentPhrasesErrors::CantParseBySerde
                        })
                    }),
                400 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<KanaParseError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::KanaParseError(e)),
                        _ => Err(AccentPhrasesErrors::CantParseBySerde),
                    }),
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::Validation(e)),
                        _ => Err(AccentPhrasesErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::Unknown)
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
    core_version: Option<String>,
    // in body
    text: String,
    speaker: i64,
    is_kana: bool,
    audio_file: String,
    normalize: bool,
}
impl Api for GuidedAccentPhrase {
    type Response = Result<Vec<AccentPhrase>, AccentPhrasesErrors>;

    fn post(&self) -> Self::Response {
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
            AccentPhrasesErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|txt| {
                        log::debug!("{}", txt);
                        serde_json::from_str::<Vec<AccentPhrase>>(&txt).map_err(|err| {
                            log::error!("{}", err);
                            AccentPhrasesErrors::CantParseBySerde
                        })
                    }),
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::Validation(e)),
                        _ => Err(AccentPhrasesErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::Unknown)
                }
            }
        })
    }
}

///アクセント句から音高を得る
pub struct MoraData {
    //in query
    speaker: i64,
    core_version: Option<String>,
    //in body
    accent_phrases: Vec<AccentPhrase>,
}

impl Api for MoraData {
    type Response = Result<Vec<AccentPhrase>, AccentPhrasesErrors>;

    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/guided_accent_phrase")
            .query("speaker", &self.speaker.to_string());

        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .send_json(&self.accent_phrases)
        .map_err(|e| {
            log::error!("{:?}", e);
            AccentPhrasesErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|txt| {
                        log::debug!("{}", &txt);
                        serde_json::from_str::<Vec<AccentPhrase>>(&txt).map_err(|err| {
                            log::error!("{}", err);
                            AccentPhrasesErrors::CantParseBySerde
                        })
                    }),
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::Validation(e)),
                        _ => Err(AccentPhrasesErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::Unknown)
                }
            }
        })
    }
}

/// # アクセント句から音素長を得る
pub struct MoraLength {
    // in query.
    speaker: i64,
    core_version: Option<String>,
    // in body.
    accent_phrases: Vec<AccentPhrase>,
}

impl Api for MoraLength {
    type Response = Result<Vec<AccentPhrase>, AccentPhrasesErrors>;

    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/guided_accent_phrase")
            .query("speaker", &self.speaker.to_string());

        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .send_json(&self.accent_phrases)
        .map_err(|e| {
            log::error!("{:?}", e);
            AccentPhrasesErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|txt| {
                        log::debug!("{}", &txt);
                        serde_json::from_str::<Vec<AccentPhrase>>(&txt).map_err(|err| {
                            log::error!("{}", err);
                            AccentPhrasesErrors::CantParseBySerde
                        })
                    }),
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::Validation(e)),
                        _ => Err(AccentPhrasesErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::Unknown)
                }
            }
        })
    }
}

/// # アクセント句から音素長を得る
pub struct MoraPitch {
    // in query.
    speaker: i64,
    core_version: Option<String>,
    // in body.
    accent_phrases: Vec<AccentPhrase>,
}

impl Api for MoraPitch {
    type Response = Result<Vec<AccentPhrase>, AccentPhrasesErrors>;

    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/guided_accent_phrase")
            .query("speaker", &self.speaker.to_string());

        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .send_json(&self.accent_phrases)
        .map_err(|e| {
            log::error!("{:?}", e);
            AccentPhrasesErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|txt| {
                        log::debug!("{}", &txt);
                        serde_json::from_str::<Vec<AccentPhrase>>(&txt).map_err(|err| {
                            log::error!("{}", err);
                            AccentPhrasesErrors::CantParseBySerde
                        })
                    }),
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::Validation(e)),
                        _ => Err(AccentPhrasesErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::Unknown)
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
    pub(crate) core_version: Option<String>,
    // in body json.
    pub(crate) audio_query: crate::api_schema::AudioQuery,
}

impl Api for Synthesis {
    type Response = Result<Vec<u8>, AccentPhrasesErrors>;

    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/synthesis")
            .query("speaker", &self.speaker.to_string());
        let query = if let Some(cv) = &self.enable_interrogative_upspeak {
            query.query("enable_interrogative_upspeak", &cv.to_string())
        } else {
            query
        };
        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .send_json(&self.audio_query)
        .map_err(|e| {
            log::error!("{:?}", e);
            AccentPhrasesErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => {
                    let mut buffer = Vec::new();
                    res.into_reader()
                        .read_to_end(&mut buffer)
                        .map_err(|_| AccentPhrasesErrors::IO)?;
                    Ok(buffer)
                }
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::Validation(e)),
                        _ => Err(AccentPhrasesErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::Unknown)
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
    pub(crate) core_version: Option<String>,
    // in body json.
    pub(crate) audio_query: crate::api_schema::AudioQuery,
}

impl Api for CancellableSynthesis {
    type Response = Result<Vec<u8>, AccentPhrasesErrors>;

    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/cancellable_synthesis")
            .query("speaker", &self.speaker.to_string());
        let query = if let Some(cv) = &self.enable_interrogative_upspeak {
            query.query("enable_interrogative_upspeak", &cv.to_string())
        } else {
            query
        };
        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .send_json(&self.audio_query)
        .map_err(|e| {
            log::error!("{:?}", e);
            AccentPhrasesErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => {
                    let mut buffer = Vec::new();
                    res.into_reader()
                        .read_to_end(&mut buffer)
                        .map_err(|_| AccentPhrasesErrors::IO)?;
                    Ok(buffer)
                }
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::Validation(e)),
                        _ => Err(AccentPhrasesErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::Unknown)
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
    pub(crate) core_version: Option<String>,
    // in body json.
    pub(crate) audio_query: Vec<crate::api_schema::AudioQuery>,
}

impl Api for MultiSynthesis {
    type Response = Result<Vec<u8>, AccentPhrasesErrors>;

    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/multi_synthesis")
            .query("speaker", &self.speaker.to_string());

        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .send_json(&self.audio_query)
        .map_err(|e| {
            log::error!("{:?}", e);
            AccentPhrasesErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => {
                    let mut buffer = Vec::new();
                    res.into_reader()
                        .read_to_end(&mut buffer)
                        .map_err(|_| AccentPhrasesErrors::IO)?;
                    Ok(buffer)
                }
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::Validation(e)),
                        _ => Err(AccentPhrasesErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::Unknown)
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
    pub(crate) core_version: Option<String>,
    // in body json.
    pub(crate) audio_query: crate::api_schema::AudioQuery,
}

impl Api for SynthesisMorphing {
    type Response = Result<Vec<u8>, AccentPhrasesErrors>;

    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/synthesis_morphing")
            .query("base_speaker", &self.base_speaker.to_string())
            .query("target_speaker", &self.target_speaker.to_string())
            .query("morph_rate", &self.morph_rate.to_string());

        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
        .send_json(&self.audio_query)
        .map_err(|e| {
            log::error!("{:?}", e);
            AccentPhrasesErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => {
                    let mut buffer = Vec::new();
                    res.into_reader()
                        .read_to_end(&mut buffer)
                        .map_err(|_| AccentPhrasesErrors::IO)?;
                    Ok(buffer)
                }
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::Validation(e)),
                        _ => Err(AccentPhrasesErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::Unknown)
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
    pub(crate) core_version: Option<String>,
    // in form.
    pub(crate) form_data: crate::api_schema::GuidedSynthesisFormData,
}

impl Api for GuidedSynthesis {
    type Response = Result<Vec<u8>, AccentPhrasesErrors>;

    fn post(&self) -> Self::Response {
        let query = ureq::post("http://localhost:50021/guided_synthesis");

        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
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
            AccentPhrasesErrors::IO
        })
        .and_then(|res| {
            let status = res.status();
            log::debug!("{}", status);
            match status {
                200 => {
                    let mut buffer = Vec::new();
                    res.into_reader()
                        .read_to_end(&mut buffer)
                        .map_err(|_| AccentPhrasesErrors::IO)?;
                    Ok(buffer)
                }
                422 => res
                    .into_string()
                    .map_err(|e| {
                        log::error!("{}", e);
                        AccentPhrasesErrors::IO
                    })
                    .and_then(|x| match serde_json::from_str::<HttpValidationError>(&x) {
                        Ok(e) => Err(AccentPhrasesErrors::Validation(e)),
                        _ => Err(AccentPhrasesErrors::Unknown),
                    }),
                x => {
                    log::error!("http status code {}", x);
                    Err(AccentPhrasesErrors::Unknown)
                }
            }
        })
    }
}
