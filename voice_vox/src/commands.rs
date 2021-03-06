use crate::history::Command;
use crate::project::VoiceVoxProject;
use crate::{api_schema, project};

pub enum AudioQueryCommands {
    Remove(usize, Option<project::AudioItem>),
    Insert(project::AudioItem),
    UpdateAccentPhrases {
        new_text: String,
        prev_text: String,
        accent_phrases: Vec<api_schema::AccentPhraseInProject>,
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
                        std::mem::swap(&mut aq.accentPhrases, accent_phrases);
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
                        std::mem::swap(&mut aq.accentPhrases, accent_phrases);
                        log::debug!("swapped {} accent_phrases", uuid);
                    }
                }
            }
        }
    }

    fn op_name(&self) -> &str {
        match self {
            AudioQueryCommands::Remove(_, _) => "?????????",
            AudioQueryCommands::Insert(_) => "?????????",
            AudioQueryCommands::UpdateAccentPhrases { .. } => "????????????/????????????",
        }
    }
}

pub enum BottomPaneCommand {
    ///
    /// [[???],[???],[???]]*[[???],[???],[???],[???]]
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
    ///  [ [???] [???] * [???] ],[[???] [???] [???] [???]]
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
                        assert!(*index + 1 < aq.accentPhrases.len());
                        let right_moras = aq.accentPhrases[*index + 1].moras.clone();
                        aq.accentPhrases[*index]
                            .moras
                            .extend_from_slice(&right_moras);
                        aq.accentPhrases.remove(*index + 1);
                    }
                }
            }
            BottomPaneCommand::Split {
                accent_phrase: index,
                mora,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        assert!(*index < aq.accentPhrases.len());
                        let insert = crate::api_schema::AccentPhraseInProject {
                            moras: aq.accentPhrases[*index].moras.split_off(*mora),
                            accent: 0,
                            pause_mora: None,
                            isInterrogative: None,
                        };
                        aq.accentPhrases.insert(*index + 1, insert);
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
                        aq.accentPhrases[*accent_phrase].accent = *new_accent as i32;
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
                        aq.accentPhrases[*accent_phrase].moras[*mora].pitch += *pitch_diff;
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
                            aq.accentPhrases[*accent_phrase].moras[*mora].vowelLength += *vd;
                        }
                        if let Some(cd) = consonant_diff {
                            if let Some(consonant) =
                                &mut aq.accentPhrases[*accent_phrase].moras[*mora].consonantLength
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
                        assert!(*index < aq.accentPhrases.len());
                        let insert = crate::api_schema::AccentPhraseInProject {
                            moras: aq.accentPhrases[*index].moras.split_off(*length),
                            accent: 0,
                            pause_mora: None,
                            isInterrogative: None,
                        };
                        aq.accentPhrases.insert(*index + 1, insert);
                    }
                }
            }
            BottomPaneCommand::Split {
                accent_phrase: index,
                mora: _,
            } => {
                if let Some(ai) = project.audioItems.get_mut(uuid) {
                    if let Some(aq) = &mut ai.query {
                        assert!(*index + 1 < aq.accentPhrases.len());
                        let right_moras = aq.accentPhrases[*index + 1].moras.clone();
                        aq.accentPhrases[*index]
                            .moras
                            .extend_from_slice(&right_moras);
                        aq.accentPhrases.remove(*index + 1);
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
                        aq.accentPhrases[*accent_phrase].accent = *prev_accent as i32;
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
                        aq.accentPhrases[*accent_phrase].moras[*mora].pitch -= *pitch_diff;
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
                            aq.accentPhrases[*accent_phrase].moras[*mora].vowelLength -= *vd;
                        }
                        if let Some(cd) = consonant_diff {
                            if let Some(consonant) =
                                &mut aq.accentPhrases[*accent_phrase].moras[*mora].consonantLength
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
            BottomPaneCommand::Concat { .. } => "?????????????????????????????????",
            BottomPaneCommand::Split { .. } => "?????????????????????????????????",
            BottomPaneCommand::AccentPhrase { .. } => "???????????????????????????",
            BottomPaneCommand::Pitch { .. } => "???????????????",
            BottomPaneCommand::VowelAndConsonant { .. } => "????????????????????????",
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
                        query.speedScale += *x;
                    }
                    AudioQueryEditCommand::PitchScale(x) => {
                        query.pitchScale += *x;
                    }
                    AudioQueryEditCommand::VolumeScale(x) => {
                        query.volumeScale += *x;
                    }
                    AudioQueryEditCommand::PrePhonemeLength(x) => {
                        query.prePhonemeLength += *x;
                    }
                    AudioQueryEditCommand::PostPhonemeLength(x) => {
                        query.postPhonemeLength += *x;
                    }
                    AudioQueryEditCommand::IntonationScale(x) => {
                        query.intonationScale += *x;
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
                        query.speedScale -= *x;
                    }
                    AudioQueryEditCommand::PitchScale(x) => {
                        query.pitchScale -= *x;
                    }
                    AudioQueryEditCommand::VolumeScale(x) => {
                        query.volumeScale -= *x;
                    }
                    AudioQueryEditCommand::PrePhonemeLength(x) => {
                        query.prePhonemeLength -= *x;
                    }
                    AudioQueryEditCommand::PostPhonemeLength(x) => {
                        query.postPhonemeLength -= *x;
                    }
                    AudioQueryEditCommand::IntonationScale(x) => {
                        query.intonationScale -= *x;
                    }
                };
            }
        }
    }

    fn op_name(&self) -> &str {
        match self {
            AudioQueryEditCommand::SpeedScale(_) => "??????",
            AudioQueryEditCommand::PitchScale(_) => "??????",
            AudioQueryEditCommand::VolumeScale(_) => "??????",
            AudioQueryEditCommand::PrePhonemeLength(_) => "????????????",
            AudioQueryEditCommand::PostPhonemeLength(_) => "????????????",
            AudioQueryEditCommand::IntonationScale(_) => "??????",
        }
    }
}
