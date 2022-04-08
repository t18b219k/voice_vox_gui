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
    fn call(&self) -> Self::Response {
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
    fn call(&self) -> Self::Response {
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

    fn call(&self) -> Self::Response;
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
    fn call(&self) -> Self::Response {
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

    fn call(&self) -> Self::Response {
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

    fn call(&self) -> Self::Response {
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

    fn call(&self) -> Self::Response {
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

    fn call(&self) -> Self::Response {
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

    fn call(&self) -> Self::Response {
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

    fn call(&self) -> Self::Response {
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

    fn call(&self) -> Self::Response {
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

    fn call(&self) -> Self::Response {
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

    fn call(&self) -> Self::Response {
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

/// # base64エンコードされた複数のwavデータを一つに結合する
///
/// base64エンコードされたwavデータを一纏めにし、wavファイルで返します。
pub struct ConnectWaves {
    waves: Vec<Vec<u8>>,
}

impl Api for ConnectWaves {
    type Response = Result<Vec<u8>, AccentPhrasesErrors>;

    fn call(&self) -> Self::Response {
        let mut buffer = Vec::new();
        for wave in &self.waves {
            let b64 = base64::encode(wave);
            buffer.push(wave);
        }

        ureq::post("http://localhost:50021/connect_waves")
            .send_json(buffer)
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

pub struct Presets;

#[derive(Debug)]
pub enum PresetsError {
    UREQ_IO,
    InvalidJSON,
}

impl Api for Presets {
    type Response = Result<Vec<crate::api_schema::Preset>, PresetsError>;

    fn call(&self) -> Self::Response {
        ureq::get("http://localhost:50021/presets")
            .call()
            .map_err(|_| PresetsError::UREQ_IO)
            .and_then(|response| {
                response
                    .into_json::<Vec<crate::api_schema::Preset>>()
                    .map_err(|_| PresetsError::InvalidJSON)
            })
    }
}

#[test]
fn call_presets() {
    let presets = Presets;
    for preset in presets.call().unwrap() {
        println!("{:?}", preset);
    }
}

pub struct Version;

#[derive(Debug)]
pub enum VersionError {
    UREQ_IO,
    InvalidJSON,
}

impl Api for Version {
    type Response = Result<Option<String>, VersionError>;

    fn call(&self) -> Self::Response {
        ureq::get("http://localhost:50021/version")
            .call()
            .map_err(|_| VersionError::UREQ_IO)
            .and_then(|response| {
                response
                    .into_json::<Option<String>>()
                    .map_err(|_| VersionError::InvalidJSON)
            })
    }
}

#[test]
fn call_version() {
    let version = Version;
    println!("{:?}", version.call().unwrap());
}

pub struct CoreVersions;

#[derive(Debug)]
pub enum CoreVersionError {
    UREQ_IO,
    InvalidJSON,
}

impl Api for CoreVersions {
    type Response = Result<Vec<String>, VersionError>;

    fn call(&self) -> Self::Response {
        ureq::get("http://localhost:50021/core_versions")
            .call()
            .map_err(|_| VersionError::UREQ_IO)
            .and_then(|response| {
                response
                    .into_json::<Vec<String>>()
                    .map_err(|_| VersionError::InvalidJSON)
            })
    }
}

#[test]
fn call_core_versions() {
    let version = CoreVersions;
    println!("{:?}", version.call().unwrap());
}

pub struct Speakers {
    core_version: Option<String>,
}

impl Api for Speakers {
    type Response = Result<Vec<crate::api_schema::Speaker>, AccentPhrasesErrors>;

    fn call(&self) -> Self::Response {
        let query = ureq::get("http://localhost:50021/speakers");

        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
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
                    .into_json::<Vec<crate::api_schema::Speaker>>()
                    .map_err(|_| AccentPhrasesErrors::IO),
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

#[test]
fn call_speakers() {
    let speakers = Speakers { core_version: None };
    println!("{:?}", speakers.call().unwrap());
}

pub struct SpeakerInfo {
    speaker_uuid: String,
    core_version: Option<String>,
}

impl Api for SpeakerInfo {
    type Response = Result<crate::api_schema::SpeakerInfo, AccentPhrasesErrors>;

    fn call(&self) -> Self::Response {
        let query = ureq::get("http://localhost:50021/speaker_info")
            .query("speaker_uuid", &self.speaker_uuid);

        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
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
                    .into_json::<crate::api_schema::SpeakerInfoRaw>()
                    .map_err(|_| AccentPhrasesErrors::IO)
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

#[test]
fn call_speaker_info() {
    let speakers = Speakers { core_version: None };
    let speakers = speakers.call().unwrap();
    let info = SpeakerInfo {
        speaker_uuid: speakers[0].speaker_uuid.clone(),
        core_version: None,
    };
    println!("{:?}", info.call());
}

pub struct SupportedDevices {
    core_version: Option<String>,
}

impl Api for SupportedDevices {
    type Response = Result<crate::api_schema::SupportedDevices, AccentPhrasesErrors>;

    fn call(&self) -> Self::Response {
        let query = ureq::get("http://localhost:50021/supported_devices");

        if let Some(cv) = &self.core_version {
            query.query("core_version", cv)
        } else {
            query
        }
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
                    .into_json::<crate::api_schema::SupportedDevices>()
                    .map_err(|_| AccentPhrasesErrors::IO),
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

#[test]
fn call_supported_devices() {
    let supported_devices = SupportedDevices { core_version: None };
    println!("{:?}", supported_devices.call().unwrap());
}
