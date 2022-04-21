//! definition of VoiceVox openapi path section
//!
//!

use crate::api_schema::{AccentPhrase, AccentPhrasesResponse, HttpValidationError, KanaParseError};
use async_trait::async_trait;
use once_cell::race::OnceBox;
use reqwest::{Error, StatusCode};
use std::io::Read;

pub type CoreVersion = Option<String>;

///起動時に[crate::api::init()]を使用し初期化すること.
static CLIENT: OnceBox<reqwest::Client> = once_cell::race::OnceBox::new();

pub fn init() {
    let client = reqwest::Client::new();
    CLIENT.set(Box::new(client)).unwrap()
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
    type Response = Result<crate::api_schema::AudioQuery, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/audio_query")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .query(&[("text", &self.text)])
            .build()?;
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<HttpValidationError>().await?,
            )),
            x => Err(x.into()),
        }
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
    type Response = Result<crate::api_schema::AudioQuery, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/audio_query_from_preset")
            .query(&[("preset_id", self.preset_id)])
            .add_core_version(&self.core_version)
            .query(&[("text", &self.text)])
            .build()?;
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<HttpValidationError>().await?,
            )),
            x => Err(x.into()),
        }
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
    ApiError(APIErrorReqwest),
}

impl From<reqwest::Error> for AccentPhrasesErrors {
    fn from(e: Error) -> Self {
        AccentPhrasesErrors::ApiError(e.into())
    }
}

impl From<reqwest::StatusCode> for AccentPhrasesErrors {
    fn from(e: reqwest::StatusCode) -> Self {
        AccentPhrasesErrors::ApiError(e.into())
    }
}

#[async_trait]
impl Api for AccentPhrases {
    type Response = Result<AccentPhrasesResponse, AccentPhrasesErrors>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/audio_query")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .query(&[("is_kana", self.is_kana.unwrap_or(false))])
            .query(&[("text", &self.text)])
            .build()?;
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::BAD_REQUEST => Err(AccentPhrasesErrors::KanaParseError(
                res.json::<KanaParseError>().await?,
            )),
            StatusCode::UNPROCESSABLE_ENTITY => Err(AccentPhrasesErrors::ApiError(
                APIErrorReqwest::Validation(res.json::<HttpValidationError>().await?),
            )),
            x => Err(x.into()),
        }
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
    type Response = Result<Vec<AccentPhrase>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/mora_data")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .json(&self.accent_phrases)
            .build()?;
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<HttpValidationError>().await?,
            )),
            x => Err(x.into()),
        }
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
    type Response = Result<Vec<AccentPhrase>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/mora_length")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .json(&self.accent_phrases)
            .build()?;
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<HttpValidationError>().await?,
            )),
            x => Err(x.into()),
        }
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
    type Response = Result<Vec<AccentPhrase>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/mora_pitch")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .json(&self.accent_phrases)
            .build()?;
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<HttpValidationError>().await?,
            )),
            x => Err(x.into()),
        }
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
    type Response = Result<Vec<u8>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/synthesis")
            .query(&[("speaker", self.speaker)])
            .query(&[(
                "enable_interrogative_upspeak",
                self.enable_interrogative_upspeak.unwrap_or(true),
            )])
            .add_core_version(&self.core_version)
            .json(&self.audio_query)
            .build()?;
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.bytes().await.unwrap_or_default().to_vec()),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<HttpValidationError>().await?,
            )),
            x => Err(x.into()),
        }
    }
}

/// # 音声合成する（キャンセル可能）
pub struct CancellableSynthesis {
    // in query
    pub(crate) speaker: i64,
    pub(crate) core_version: CoreVersion,
    // in body json.
    pub(crate) audio_query: crate::api_schema::AudioQuery,
}
#[async_trait]
impl Api for CancellableSynthesis {
    type Response = Result<Vec<u8>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/cancellable_synthesis")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .json(&self.audio_query)
            .build()?;
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.bytes().await.unwrap_or_default().to_vec()),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<HttpValidationError>().await?,
            )),
            x => Err(x.into()),
        }
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
    type Response = Result<Vec<u8>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/multi_synthesis")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .json(&self.audio_query)
            .build()
            .unwrap();
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.bytes().await.unwrap_or_default().to_vec()),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<HttpValidationError>().await?,
            )),
            x => Err(x.into()),
        }
    }
}

#[tokio::test]
async fn call_multi_synthesis() {
    init();
    let aq0 = AudioQuery {
        text: "日本語".to_string(),
        speaker: 0,
        core_version: None,
    }
    .call()
    .await
    .unwrap();
    let aq1 = AudioQuery {
        text: "音声合成".to_string(),
        speaker: 0,
        core_version: None,
    }
    .call()
    .await
    .unwrap();
    MultiSynthesis {
        speaker: 0,
        core_version: None,
        audio_query: vec![aq0, aq1],
    }
    .call()
    .await
    .unwrap();
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
    type Response = Result<Vec<u8>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/synthesis_morphing")
            .query(&[
                ("base_speaker", self.base_speaker),
                ("target_speaker", self.target_speaker),
            ])
            .query(&[("morph_rate", self.morph_rate)])
            .add_core_version(&self.core_version)
            .json(&self.audio_query)
            .build()
            .unwrap();
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.bytes().await.unwrap_or_default().to_vec()),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<HttpValidationError>().await?,
            )),
            x => Err(x.into()),
        }
    }
}

#[tokio::test]
async fn call_synthesis_morphing() {
    init();
    let speakers: Vec<crate::api_schema::Speaker> =
        Speakers { core_version: None }.call().await.unwrap();
    let id_0 = speakers[0].styles[0].id;
    let id_1 = speakers[1].styles[0].id;

    let aq = AudioQuery {
        text: "音声合成".to_string(),
        speaker: id_0,
        core_version: None,
    }
    .call()
    .await
    .unwrap();
    SynthesisMorphing {
        base_speaker: id_0,
        target_speaker: id_1,
        morph_rate: 0.5,
        core_version: None,
        audio_query: aq,
    }
    .call()
    .await
    .unwrap();
}

/// # base64エンコードされた複数のwavデータを一つに結合する
///
/// base64エンコードされたwavデータを一纏めにし、wavファイルで返します。
pub struct ConnectWaves {
    waves: Vec<Vec<u8>>,
}
#[async_trait]
impl Api for ConnectWaves {
    type Response = Result<Vec<u8>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let mut buffer = Vec::new();
        for wave in &self.waves {
            buffer.push(base64::encode(wave));
        }
        let cl = CLIENT.get().unwrap();
        let request = cl
            .post("http://localhost:50021/connect_waves")
            .json(&buffer)
            .build()
            .unwrap();
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(base64::decode(res.text().await?).unwrap_or_default()),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<HttpValidationError>().await?,
            )),
            x => Err(x.into()),
        }
    }
}

#[tokio::test]
async fn call_connect_waves() {
    let waves = vec![];
    init();
    println!(
        "{:?}",
        ConnectWaves { waves }.call().await.unwrap_or_default()
    );
}

pub struct Presets;
#[async_trait]
impl Api for Presets {
    type Response = Result<Vec<crate::api_schema::Preset>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl.get("http://localhost:50021/presets").build().unwrap();
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<Vec<crate::api_schema::Preset>>().await?),
            x => Err(x.into()),
        }
    }
}

#[tokio::test]
async fn call_presets() {
    init();
    let presets = Presets;
    for preset in presets.call().await.unwrap() {
        println!("{:?}", preset);
    }
}

pub struct Version;
#[async_trait]
impl Api for Version {
    type Response = Result<Option<String>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl.get("http://localhost:50021/version").build().unwrap();
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<Option<String>>().await?),
            x => Err(x.into()),
        }
    }
}

#[tokio::test]
async fn call_version() {
    init();
    let version = Version;
    println!("{:?}", version.call().await.unwrap());
}

pub struct CoreVersions;
#[async_trait]
impl Api for CoreVersions {
    type Response = Result<Vec<String>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .get("http://localhost:50021/core_versions")
            .build()
            .unwrap();
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<Vec<String>>().await?),
            x => Err(x.into()),
        }
    }
}

#[tokio::test]
async fn call_core_versions() {
    init();
    let version = CoreVersions;
    println!("{:?}", version.call().await.unwrap());
}

pub struct Speakers {
    pub(crate) core_version: CoreVersion,
}
#[async_trait]
impl Api for Speakers {
    type Response = Result<Vec<crate::api_schema::Speaker>, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let request = cl
            .get("http://localhost:50021/speakers")
            .add_core_version(&self.core_version)
            .build()
            .unwrap();
        let res = cl.execute(request).await.unwrap();
        match res.status() {
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<crate::api_schema::HttpValidationError>().await?,
            )),
            StatusCode::OK => Ok(res.json::<Vec<crate::api_schema::Speaker>>().await?),
            x => Err(x.into()),
        }
    }
}

#[tokio::test]
async fn call_speakers() {
    init();
    let speakers = Speakers { core_version: None };
    println!("{:?}", speakers.call().await.unwrap());
}

pub struct SpeakerInfo {
    pub(crate) speaker_uuid: String,
    pub(crate) core_version: CoreVersion,
}
#[async_trait]
impl Api for SpeakerInfo {
    type Response = Result<crate::api_schema::SpeakerInfo, APIErrorReqwest>;

    async fn call(&self) -> Self::Response {
        let cl = CLIENT.get().unwrap();
        let req = cl
            .get("http://localhost:50021/speaker_info")
            .query(&[("speaker_uuid", &self.speaker_uuid)])
            .add_core_version(&self.core_version)
            .build()
            .unwrap();
        let res = cl.execute(req).await.unwrap();
        match res.status() {
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIErrorReqwest::Validation(
                res.json::<crate::api_schema::HttpValidationError>().await?,
            )),
            StatusCode::OK => Ok({
                let raw = res.json::<crate::api_schema::SpeakerInfoRaw>().await?;
                crate::api_schema::SpeakerInfo {
                    policy: raw.policy,
                    portrait: base64::decode(raw.portrait).unwrap_or_default(),
                    style_infos: raw
                        .style_infos
                        .iter()
                        .map(|si| {
                            let crate::api_schema::StyleInfoRaw {
                                id,
                                icon,
                                voice_samples,
                            } = si;
                            crate::api_schema::StyleInfo {
                                id: *id,
                                icon: base64::decode(icon).unwrap_or_default(),
                                voice_samples: voice_samples
                                    .iter()
                                    .map(|voice| base64::decode(voice).unwrap_or_default())
                                    .collect(),
                            }
                        })
                        .collect(),
                }
            }),
            x => Err(x.into()),
        }
    }
}

#[tokio::test]
async fn call_speaker_info() {
    init();
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

#[tokio::test]
async fn call_supported_devices() {
    init();
    let supported_devices = SupportedDevices { core_version: None };
    println!("{:?}", supported_devices.call().await.unwrap());
}

pub trait AddCoreVersion {
    fn add_core_version(self, core_version: &CoreVersion) -> Self;
}

impl AddCoreVersion for reqwest::RequestBuilder {
    fn add_core_version(self, core_version: &CoreVersion) -> Self {
        if let Some(cv) = &core_version {
            self.query(&[("core_version", cv)])
        } else {
            self
        }
    }
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
