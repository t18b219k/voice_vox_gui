use crate::history::Command;
use crate::project::VoiceVoxProject;
use crate::{api_schema, project};

pub enum AudioQueryCommands {
    Remove(usize, Option<project::AudioItem>),
    Insert(project::AudioItem),
    UpdateAccentPhrases {
        new_text: String,
        prev_text: String,
        accent_phrases: Vec<api_schema::AccentPhrase>,
    },
}

impl Command for AudioQueryCommands {
    fn invoke(&mut self, project: &mut VoiceVoxProject, uuid: &str) {
        match self {
            AudioQueryCommands::Remove(pos_save, ref mut save) => {
                let pos = project.audioKeys.iter().enumerate().find(|x| x.1 == uuid);
                if let Some((index, _)) = pos {
                    project.audioKeys.remove(index);
                    *pos_save = index;
                }

                if let Some(value) = project.audioItems.remove(uuid) {
                    save.replace(value);
                }
            }
            AudioQueryCommands::Insert(value) => {
                if !project.audioKeys.contains(&uuid.to_owned()) {
                    project.audioKeys.push(uuid.to_owned());
                }
                project.audioItems.insert(uuid.to_owned(), value.clone());
            }
            AudioQueryCommands::UpdateAccentPhrases {
                new_text,
                prev_text,
                accent_phrases,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    ai.text = new_text.clone();
                    log::debug!("{} text {} -> {}", uuid, prev_text, ai.text);
                    if let Some(aq) = &mut ai.query {
                        std::mem::swap(&mut aq.accent_phrases, accent_phrases);
                        log::debug!("swapped {} accent_phrases", uuid);
                    }
                }
            }
        }
    }

    fn undo(&mut self, project: &mut VoiceVoxProject, uuid: &str) {
        match self {
            AudioQueryCommands::Remove(pos, save) => {
                project.audioKeys.insert(*pos, uuid.to_owned());

                if let Some(record) = save {
                    project.audioItems.insert(uuid.to_owned(), record.clone());
                }
                *save = None;
            }
            AudioQueryCommands::Insert(_) => {
                if project.audioKeys.contains(&uuid.to_owned()) {
                    project.audioKeys.pop();
                    project.audioItems.remove(uuid);
                }
            }
            AudioQueryCommands::UpdateAccentPhrases {
                new_text,
                prev_text,
                accent_phrases,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    ai.text = prev_text.clone();
                    log::debug!("{} text {} -> {}", uuid, new_text, prev_text);
                    if let Some(aq) = &mut ai.query {
                        std::mem::swap(&mut aq.accent_phrases, accent_phrases);
                        log::debug!("swapped {} accent_phrases", uuid);
                    }
                }
            }
        }
    }

    fn op_name(&self) -> &str {
        match self {
            AudioQueryCommands::Remove(_, _) => "行削除",
            AudioQueryCommands::Insert(_) => "行挿入",
            AudioQueryCommands::UpdateAccentPhrases { .. } => "テキスト/波形変更",
        }
    }
}

pub enum BottomPaneCommand {
    ///
    /// [[ニ],[ホ],[ン]]*[[シ],[マ],[グ],[ニ]]
    ///  ->
    /// ```
    ///  Concat{
    ///     accent_phrase:0,
    ///     length:3,
    ///    }
    /// ```
    ///
    Concat { accent_phrase: usize, length: usize },
    ///
    ///
    ///  [ [ニ] [ホ] * [ン] ],[[シ] [マ] [グ] [ニ]]
    ///
    /// ->```
    /// Split{accent_phrase:0,mora:2}
    /// ```
    Split { accent_phrase: usize, mora: usize },

    AccentPhrase {
        accent_phrase: usize,
        new_accent: usize,
        prev_accent: usize,
    },
    Pitch {
        accent_phrase: usize,
        mora: usize,
        pitch_diff: f32,
    },
    VowelAndConsonant {
        accent_phrase: usize,
        mora: usize,
        vowel_diff: Option<f32>,
        consonant_diff: Option<f32>,
    },
}

impl Command for BottomPaneCommand {
    fn invoke(&mut self, project: &mut VoiceVoxProject, uuid: &str) {
        match self {
            BottomPaneCommand::Concat {
                accent_phrase: index,
                length: _,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        assert!(*index + 1 < aq.accent_phrases.len());
                        let right_moras = aq.accent_phrases[*index + 1].moras.clone();
                        aq.accent_phrases[*index]
                            .moras
                            .extend_from_slice(&right_moras);
                        aq.accent_phrases.remove(*index + 1);
                    }
                }
            }
            BottomPaneCommand::Split {
                accent_phrase: index,
                mora,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        assert!(*index < aq.accent_phrases.len());
                        let insert = crate::api_schema::AccentPhrase {
                            moras: aq.accent_phrases[*index].moras.split_off(*mora),
                            accent: 0,
                            pause_mora: None,
                            is_interrogative: None,
                        };
                        aq.accent_phrases.insert(*index + 1, insert);
                    }
                }
            }
            BottomPaneCommand::AccentPhrase {
                accent_phrase,
                new_accent,
                prev_accent: _,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        aq.accent_phrases[*accent_phrase].accent = *new_accent as i32;
                    }
                }
            }
            BottomPaneCommand::Pitch {
                accent_phrase,
                mora,
                pitch_diff,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        aq.accent_phrases[*accent_phrase].moras[*mora].pitch += *pitch_diff;
                    }
                }
            }
            BottomPaneCommand::VowelAndConsonant {
                accent_phrase,
                mora,
                vowel_diff,
                consonant_diff,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        if let Some(vd) = vowel_diff {
                            aq.accent_phrases[*accent_phrase].moras[*mora].vowel_length += *vd;
                        }
                        if let Some(cd) = consonant_diff {
                            if let Some(consonant) =
                                &mut aq.accent_phrases[*accent_phrase].moras[*mora].consonant_length
                            {
                                *consonant += *cd;
                            }
                        }
                    }
                }
            }
        }
    }

    fn undo(&mut self, project: &mut VoiceVoxProject, uuid: &str) {
        match self {
            BottomPaneCommand::Concat {
                accent_phrase: index,
                length,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        assert!(*index < aq.accent_phrases.len());
                        let insert = crate::api_schema::AccentPhrase {
                            moras: aq.accent_phrases[*index].moras.split_off(*length),
                            accent: 0,
                            pause_mora: None,
                            is_interrogative: None,
                        };
                        aq.accent_phrases.insert(*index + 1, insert);
                    }
                }
            }
            BottomPaneCommand::Split {
                accent_phrase: index,
                mora: _,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        assert!(*index + 1 < aq.accent_phrases.len());
                        let right_moras = aq.accent_phrases[*index + 1].moras.clone();
                        aq.accent_phrases[*index]
                            .moras
                            .extend_from_slice(&right_moras);
                        aq.accent_phrases.remove(*index + 1);
                    }
                }
            }
            BottomPaneCommand::AccentPhrase {
                accent_phrase,
                new_accent: _,
                prev_accent,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        aq.accent_phrases[*accent_phrase].accent = *prev_accent as i32;
                    }
                }
            }
            BottomPaneCommand::Pitch {
                accent_phrase,
                mora,
                pitch_diff,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        aq.accent_phrases[*accent_phrase].moras[*mora].pitch -= *pitch_diff;
                    }
                }
            }
            BottomPaneCommand::VowelAndConsonant {
                accent_phrase,
                mora,
                vowel_diff,
                consonant_diff,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        if let Some(vd) = vowel_diff {
                            aq.accent_phrases[*accent_phrase].moras[*mora].vowel_length -= *vd;
                        }
                        if let Some(cd) = consonant_diff {
                            if let Some(consonant) =
                                &mut aq.accent_phrases[*accent_phrase].moras[*mora].consonant_length
                            {
                                *consonant -= *cd;
                            }
                        }
                    }
                }
            }
        }
    }

    fn op_name(&self) -> &str {
        match self {
            BottomPaneCommand::Concat { .. } => "アクセントフレーズ連結",
            BottomPaneCommand::Split { .. } => "アクセントフレーズ分割",
            BottomPaneCommand::AccentPhrase { .. } => "アクセント位置変更",
            BottomPaneCommand::Pitch { .. } => "ピッチ変更",
            BottomPaneCommand::VowelAndConsonant { .. } => "母音子音長さ変更",
        }
    }
}

pub enum AudioQueryEditCommand {
    SpeedScale(f32),
    PitchScale(f32),
    IntonationScale(f32),
    VolumeScale(f32),
    PrePhonemeLength(f32),
    PostPhonemeLength(f32),
}

impl Command for AudioQueryEditCommand {
    fn invoke(&mut self, project: &mut crate::VoiceVoxProject, uuid: &str) {
        if let Some(cell) = project.audioItems.get_mut(uuid) {
            if let Some(query) = &mut cell.query {
                match self {
                    AudioQueryEditCommand::SpeedScale(x) => {
                        query.speed_scale += *x;
                    }
                    AudioQueryEditCommand::PitchScale(x) => {
                        query.pitch_scale += *x;
                    }
                    AudioQueryEditCommand::VolumeScale(x) => {
                        query.volume_scale += *x;
                    }
                    AudioQueryEditCommand::PrePhonemeLength(x) => {
                        query.pre_phoneme_length += *x;
                    }
                    AudioQueryEditCommand::PostPhonemeLength(x) => {
                        query.post_phoneme_length += *x;
                    }
                    AudioQueryEditCommand::IntonationScale(x) => {
                        query.intonation_scale += *x;
                    }
                };
            }
        }
    }

    fn undo(&mut self, project: &mut crate::VoiceVoxProject, uuid: &str) {
        if let Some(cell) = project.audioItems.get_mut(uuid) {
            if let Some(query) = &mut cell.query {
                match self {
                    AudioQueryEditCommand::SpeedScale(x) => {
                        query.speed_scale -= *x;
                    }
                    AudioQueryEditCommand::PitchScale(x) => {
                        query.pitch_scale -= *x;
                    }
                    AudioQueryEditCommand::VolumeScale(x) => {
                        query.volume_scale -= *x;
                    }
                    AudioQueryEditCommand::PrePhonemeLength(x) => {
                        query.pre_phoneme_length -= *x;
                    }
                    AudioQueryEditCommand::PostPhonemeLength(x) => {
                        query.post_phoneme_length -= *x;
                    }
                    AudioQueryEditCommand::IntonationScale(x) => {
                        query.intonation_scale -= *x;
                    }
                };
            }
        }
    }

    fn op_name(&self) -> &str {
        match self {
            AudioQueryEditCommand::SpeedScale(_) => "話速",
            AudioQueryEditCommand::PitchScale(_) => "音高",
            AudioQueryEditCommand::VolumeScale(_) => "音量",
            AudioQueryEditCommand::PrePhonemeLength(_) => "開始無音",
            AudioQueryEditCommand::PostPhonemeLength(_) => "終了無音",
            AudioQueryEditCommand::IntonationScale(_) => "抑揚",
        }
    }
}
