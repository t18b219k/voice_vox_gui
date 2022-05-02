use serde::{Deserialize, Serialize};

use crate::api_schema;
use std::collections::HashMap;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
pub struct AudioItem {
    pub text: String,
    pub styleId: i32,
    pub query: Option<api_schema::AudioQuery>,
    pub presetKey: Option<String>,
}

const DEFAULT_SAMPLING_RATE: i64 = 24000;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
pub struct VoiceVoxProject {
    pub(crate) appVersion: String,
    pub audioKeys: Vec<String>,
    pub audioItems: HashMap<String, AudioItem>,
}
