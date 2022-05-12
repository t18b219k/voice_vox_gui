use crate::api::Api;
use crate::project::VoiceVoxProject;
use std::collections::HashMap;

/// an interface for undo/redo operations.
pub trait Command {
    /// execute command and store current state for undo/redo.
    fn invoke(&mut self, project: &mut crate::VoiceVoxProject, uuid: &str);
    /// rollback execution.
    ///
    /// perform like inverse function.
    fn undo(&mut self, project: &mut crate::VoiceVoxProject, uuid: &str);
    /// re execute command
    fn redo(&mut self, project: &mut crate::VoiceVoxProject, uuid: &str) {
        self.invoke(project, uuid)
    }
    /// used for history manager.
    fn op_name(&self) -> &str {
        "unnamed"
    }
}

/// manage some histories.
/// * manage undo/redo operations
/// * manage last update time for each line.
///
pub struct HistoryManager {
    undo_stack: Vec<(Box<dyn Command>, String)>,
    redo_stack: Vec<(Box<dyn Command>, String)>,
    update_times: HashMap<String, (Vec<tokio::time::Instant>, usize)>,
    pub project: crate::VoiceVoxProject,
    last_saved_snapshot: Option<crate::VoiceVoxProject>,
}

impl HistoryManager {
    pub fn new() -> Self {
        let blank = uuid::Uuid::new_v4();
        let dummy = blank.to_string();
        let mut items = HashMap::new();
        items.insert(
            dummy.clone(),
            crate::project::AudioItem {
                text: "".to_string(),
                styleId: 2,
                query: Some(crate::BLANK_AUDIO_QUERY.get().unwrap().clone().into()),
                presetKey: None,
            },
        );
        Self {
            undo_stack: vec![],
            redo_stack: vec![],
            update_times: Default::default(),
            project: VoiceVoxProject {
                appVersion: "0.11.4".to_string(),
                audioKeys: vec![dummy],
                audioItems: items,
            },
            last_saved_snapshot: None,
        }
    }
    pub fn from_project(project: VoiceVoxProject) -> Self {
        Self {
            undo_stack: vec![],
            redo_stack: vec![],
            update_times: Default::default(),
            project,
            last_saved_snapshot: None,
        }
    }
    /// execute command and record to undo stack.
    pub fn invoke(&mut self, mut command: Box<dyn Command>, uuid: String) {
        command.invoke(&mut self.project, &uuid);
        self.redo_stack.clear();
        let now = tokio::time::Instant::now();

        if let Some((times, cursor)) = self.update_times.get_mut(&uuid) {
            times.push(now);
            *cursor += 1;
            log::debug!("{} revision {}", uuid, cursor);
        } else {
            self.update_times.insert(uuid.clone(), (vec![now], 0));
        }

        log::debug!("exec {}", command.op_name());
        self.undo_stack.push((command, uuid));
    }

    pub fn undo(&mut self) {
        if let Some((mut op, uuid)) = self.undo_stack.pop() {
            op.undo(&mut self.project, &uuid);
            if let Some((_times, cursor)) = self.update_times.get_mut(&uuid) {
                if *cursor > 0 {
                    *cursor -= 1;
                    log::debug!("{} revision {}", uuid, cursor);
                }
            }
            log::debug!("revert {}", op.op_name());
            self.redo_stack.push((op, uuid));
        } else {
            log::debug!("no more in undo stack")
        }
    }

    pub fn redo(&mut self) {
        if let Some((mut op, uuid)) = self.redo_stack.pop() {
            op.redo(&mut self.project, &uuid);
            if let Some((times, cursor)) = self.update_times.get_mut(&uuid) {
                if *cursor < times.len() {
                    *cursor += 1;
                    log::debug!("{} revision {}", uuid, cursor);
                }
            }
            log::debug!("redo {}", op.op_name());
            self.undo_stack.push((op, uuid));
        } else {
            log::debug!("no more in redo stack.");
        }
    }

    pub fn get_current_time_stamp(&self, uuid: &str) -> Option<tokio::time::Instant> {
        self.update_times
            .get(uuid)
            .map(|(times, cursor)| times[*cursor])
    }

    pub fn is_empty(&self) -> bool {
        self.undo_stack.is_empty() && self.redo_stack.is_empty()
    }

    pub fn undoable(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn redoable(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn saved(&self) -> bool {
        if let Some(ss) = &self.last_saved_snapshot {
            ss == &self.project
        } else if self.is_empty() {
            true
        } else {
            false
        }
    }

    pub fn save(&mut self) {
        self.last_saved_snapshot = Some(self.project.clone());
    }
}
