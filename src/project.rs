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

pub struct UpdateTextCommand {
    uuid: String,
    prev_text: String,
    new_text: String,
}

impl UpdateTextCommand {
    pub fn new(uuid: String, new_text: String) -> Self {
        Self {
            uuid,
            prev_text: "".to_string(),
            new_text,
        }
    }
}

impl crate::history::Command for UpdateTextCommand {
    fn invoke(&mut self, project: &mut VoiceVoxProject) {
        if let Some(ai) = project.audioItems.get_mut(&self.uuid) {
            self.prev_text = ai.text.clone();
            ai.text = self.new_text.clone();
        }
    }

    fn undo(&mut self, project: &mut VoiceVoxProject) {
        if let Some(ai) = project.audioItems.get_mut(&self.uuid) {
            ai.text = self.prev_text.clone();
        }
    }
}

const DEFAULT_SAMPLING_RATE: i64 = 24000;

pub struct AudioState {
    now_playing: bool,
    now_generating: bool,
}
#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone)]
pub struct VoiceVoxProject {
    pub(crate) appVersion: String,
    pub audioKeys: Vec<String>,
    pub audioItems: HashMap<String, AudioItem>,
}
