//! definition of VoiceVox openapi path section
//!
//!

use crate::api_schema::{AccentPhrase, AccentPhrasesResponse, HttpValidationError, KanaParseError};
use async_trait::async_trait;
use once_cell::race::OnceBox;
use reqwest::{Error, StatusCode};
use std::io::ErrorKind;

pub type CoreVersion = Option<String>;

///シングルトン reqwest::Client
static CLIENT: OnceBox<reqwest::Client> = once_cell::race::OnceBox::new();

///クライアントのシングルトンの作成/取得を行う.
fn client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| Box::new(reqwest::Client::new()))
}

/// # 音声合成用のクエリを作成する
///
/// クエリの初期値を得ます。ここで得られたクエリはそのまま音声合成に利用できます。各値の意味はSchemasを参照してください。
///

pub struct AudioQuery {
    pub text: String,
    pub speaker: i32,
    pub core_version: Option<String>,
}

#[async_trait]
impl Api for AudioQuery {
    type Response = Result<crate::api_schema::AudioQuery, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .post("http://localhost:50021/audio_query")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .query(&[("text", &self.text)])
            .build()?;
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            x => Err(x.into()),
        }
    }
}

///
/// # 音声合成用のクエリをプリセットを用いて作成する
/// クエリの初期値を得ます。ここで得られたクエリはそのまま音声合成に利用できます。各値の意味は`Schemas`を参照してください。
///
///
pub struct AudioQueryFromPreset {
    pub text: String,
    pub preset_id: i32,
    pub core_version: CoreVersion,
}

#[async_trait]
impl Api for AudioQueryFromPreset {
    type Response = Result<crate::api_schema::AudioQuery, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .post("http://localhost:50021/audio_query_from_preset")
            .query(&[("preset_id", self.preset_id)])
            .add_core_version(&self.core_version)
            .query(&[("text", &self.text)])
            .build()?;
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
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
    pub text: String,
    pub speaker: i32,
    pub is_kana: Option<bool>,
    pub core_version: CoreVersion,
}

#[derive(Debug)]
pub enum AccentPhrasesErrors {
    KanaParseError(KanaParseError),
    ApiError(APIError),
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
        let request = client()
            .post("http://localhost:50021/audio_query")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .query(&[("is_kana", self.is_kana.unwrap_or(false))])
            .query(&[("text", &self.text)])
            .build()?;
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::BAD_REQUEST => {
                Err(AccentPhrasesErrors::KanaParseError(res.json::<_>().await?))
            }
            StatusCode::UNPROCESSABLE_ENTITY => Err(AccentPhrasesErrors::ApiError(
                APIError::Validation(res.json::<_>().await?),
            )),
            x => Err(x.into()),
        }
    }
}

///アクセント句から音高を得る
pub struct MoraData {
    //in query
    pub speaker: i32,
    pub core_version: CoreVersion,
    //in body
    pub accent_phrases: Vec<AccentPhrase>,
}

#[async_trait]
impl Api for MoraData {
    type Response = Result<Vec<AccentPhrase>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .post("http://localhost:50021/mora_data")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .json(&self.accent_phrases)
            .build()?;
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            x => Err(x.into()),
        }
    }
}

/// # アクセント句から音素長を得る
pub struct MoraLength {
    // in query.
    pub speaker: i32,
    pub core_version: CoreVersion,
    // in body.
    pub accent_phrases: Vec<AccentPhrase>,
}

#[async_trait]
impl Api for MoraLength {
    type Response = Result<Vec<AccentPhrase>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .post("http://localhost:50021/mora_length")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .json(&self.accent_phrases)
            .build()?;
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            x => Err(x.into()),
        }
    }
}

/// # アクセント句から音素長を得る
pub struct MoraPitch {
    // in query.
    pub speaker: i32,
    pub core_version: CoreVersion,
    // in body.
    pub accent_phrases: Vec<AccentPhrase>,
}
#[async_trait]
impl Api for MoraPitch {
    type Response = Result<Vec<AccentPhrase>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .post("http://localhost:50021/mora_pitch")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .json(&self.accent_phrases)
            .build()?;
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<_>().await?),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            x => Err(x.into()),
        }
    }
}

/// # 音声合成する
pub struct Synthesis {
    // in query
    pub speaker: i32,
    pub enable_interrogative_upspeak: Option<bool>,
    pub core_version: CoreVersion,
    // in body json.
    pub audio_query: crate::api_schema::AudioQuery,
}
#[async_trait]
impl Api for Synthesis {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .post("http://localhost:50021/synthesis")
            .query(&[("speaker", self.speaker)])
            .query(&[(
                "enable_interrogative_upspeak",
                self.enable_interrogative_upspeak.unwrap_or(true),
            )])
            .add_core_version(&self.core_version)
            .json(&self.audio_query)
            .build()?;
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.bytes().await.unwrap_or_default().to_vec()),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            x => Err(x.into()),
        }
    }
}

/// # 音声合成する（キャンセル可能）
pub struct CancellableSynthesis {
    // in query
    pub speaker: i32,
    pub core_version: CoreVersion,
    // in body json.
    pub audio_query: crate::api_schema::AudioQuery,
}
#[async_trait]
impl Api for CancellableSynthesis {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .post("http://localhost:50021/cancellable_synthesis")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .json(&self.audio_query)
            .build()?;
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.bytes().await.unwrap_or_default().to_vec()),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            x => Err(x.into()),
        }
    }
}

/// # まとめて音声合成する
///
/// 複数のwavがzipでまとめられて返されます.
pub struct MultiSynthesis {
    // in query
    pub speaker: i32,
    pub core_version: CoreVersion,
    // in body json.
    pub audio_query: Vec<crate::api_schema::AudioQuery>,
}

#[async_trait]
impl Api for MultiSynthesis {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .post("http://localhost:50021/multi_synthesis")
            .query(&[("speaker", self.speaker)])
            .add_core_version(&self.core_version)
            .json(&self.audio_query)
            .build()
            .unwrap();
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.bytes().await.unwrap_or_default().to_vec()),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            x => Err(x.into()),
        }
    }
}

/// # 2人の話者でモーフィングした音声を合成する
///
/// 指定された2人の話者で音声を合成、指定した割合でモーフィングした音声を得ます。 モーフィングの割合はmorph_rateで指定でき、0.0でベースの話者、1.0でターゲットの話者に近づきます。
pub struct SynthesisMorphing {
    // in query
    pub base_speaker: i32,
    pub target_speaker: i32,
    pub morph_rate: f64,
    pub core_version: CoreVersion,
    // in body json.
    pub audio_query: crate::api_schema::AudioQuery,
}

#[async_trait]
impl Api for SynthesisMorphing {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
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
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.bytes().await.unwrap_or_default().to_vec()),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            x => Err(x.into()),
        }
    }
}

/// # base64エンコードされた複数のwavデータを一つに結合する
///
/// base64エンコードされたwavデータを一纏めにし、wavファイルで返します。
/// 返されるwavはbase64デコードを行います.
///
pub struct ConnectWaves {
    pub waves: Vec<Vec<u8>>,
}

#[async_trait]
impl Api for ConnectWaves {
    type Response = Result<Vec<u8>, APIError>;

    async fn call(&self) -> Self::Response {
        let mut buffer = Vec::new();
        for wave in &self.waves {
            buffer.push(base64::encode(wave));
        }
        let request = client()
            .post("http://localhost:50021/connect_waves")
            .json(&buffer)
            .build()
            .unwrap();
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(base64::decode(res.text().await?).unwrap_or_default()),
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            x => Err(x.into()),
        }
    }
}

pub struct Presets;

#[async_trait]
impl Api for Presets {
    type Response = Result<Vec<crate::api_schema::Preset>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .get("http://localhost:50021/presets")
            .build()
            .unwrap();
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<Vec<crate::api_schema::Preset>>().await?),
            x => Err(x.into()),
        }
    }
}

pub struct Version;

#[async_trait]
impl Api for Version {
    type Response = Result<Option<String>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .get("http://localhost:50021/version")
            .build()
            .unwrap();
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<Option<String>>().await?),
            x => Err(x.into()),
        }
    }
}

pub struct CoreVersions;

#[async_trait]
impl Api for CoreVersions {
    type Response = Result<Vec<String>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .get("http://localhost:50021/core_versions")
            .build()
            .unwrap();
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::OK => Ok(res.json::<Vec<String>>().await?),
            x => Err(x.into()),
        }
    }
}

pub struct Speakers {
    pub core_version: CoreVersion,
}

#[async_trait]
impl Api for Speakers {
    type Response = Result<Vec<crate::api_schema::Speaker>, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .get("http://localhost:50021/speakers")
            .add_core_version(&self.core_version)
            .build()
            .unwrap();
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            StatusCode::OK => Ok(res.json::<Vec<crate::api_schema::Speaker>>().await?),
            x => Err(x.into()),
        }
    }
}

pub struct SpeakerInfo {
    pub speaker_uuid: String,
    pub core_version: CoreVersion,
}

#[async_trait]
impl Api for SpeakerInfo {
    type Response = Result<crate::api_schema::SpeakerInfo, APIError>;

    async fn call(&self) -> Self::Response {
        let req = client()
            .get("http://localhost:50021/speaker_info")
            .query(&[("speaker_uuid", &self.speaker_uuid)])
            .add_core_version(&self.core_version)
            .build()
            .unwrap();
        let res = client().execute(req).await.unwrap();
        match res.status() {
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            StatusCode::OK => res
                .json::<crate::api_schema::SpeakerInfoRaw>()
                .await?
                .try_into()
                .map_err(|_| APIError::Io(std::io::Error::from(ErrorKind::InvalidData))),
            x => Err(x.into()),
        }
    }
}

pub struct SupportedDevices {
    pub core_version: CoreVersion,
}

#[async_trait]
impl Api for SupportedDevices {
    type Response = Result<crate::api_schema::SupportedDevices, APIError>;

    async fn call(&self) -> Self::Response {
        let request = client()
            .get("http://localhost:50021/supported_devices")
            .add_core_version(&self.core_version)
            .build()
            .unwrap();
        let res = client().execute(request).await.unwrap();
        match res.status() {
            StatusCode::UNPROCESSABLE_ENTITY => Err(APIError::Validation(res.json::<_>().await?)),
            StatusCode::OK => Ok(res.json::<crate::api_schema::SupportedDevices>().await?),
            x => Err(x.into()),
        }
    }
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
pub enum APIError {
    Validation(HttpValidationError),
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    Unknown,
}

impl From<reqwest::Error> for APIError {
    fn from(e: Error) -> Self {
        APIError::Reqwest(e)
    }
}

impl Into<APIError> for std::io::Error {
    fn into(self) -> APIError {
        APIError::Io(self)
    }
}

impl From<StatusCode> for APIError {
    fn from(_: StatusCode) -> Self {
        APIError::Unknown
    }
}
