use crate::api::Api;
use crate::project::VoiceVoxProject;
use std::collections::HashMap;

/// an interface for undo/redo operations.
pub trait Command {
    /// execute command and store current state for undo/redo.
    fn invoke(&mut self, project: &mut crate::VoiceVoxProject);
    /// rollback execution.
    ///
    /// perform like inverse function.
    fn undo(&mut self, project: &mut crate::VoiceVoxProject);
    /// re execute command
    fn redo(&mut self, project: &mut crate::VoiceVoxProject) {
        self.invoke(project)
    }
    /// used for history manager.
    fn op_name(&self) -> &str {
        "unnamed"
    }
}

pub struct HistoryManager {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    pub project: crate::VoiceVoxProject,
}

impl HistoryManager {
    pub async fn new() -> Self {
        let blank = uuid::Uuid::new_v4();
        let dummy = blank.to_string();
        let mut items = HashMap::new();
        items.insert(
            dummy.clone(),
            crate::project::AudioItem {
                text: "".to_string(),
                styleId: 2,
                query: crate::api::AudioQuery {
                    text: "".to_string(),
                    speaker: 2,
                    core_version: None,
                }
                .call()
                .await
                .ok(),
                presetKey: None,
            },
        );
        Self {
            undo_stack: vec![],
            redo_stack: vec![],
            project: VoiceVoxProject {
                appVersion: "".to_string(),
                audioKeys: vec![dummy],
                audioItems: items,
            },
        }
    }
    /// execute command and record to undo stack.
    pub fn invoke(&mut self, mut command: Box<dyn Command>) {
        command.invoke(&mut self.project);
        self.redo_stack.clear();
        log::debug!("exec {}", command.op_name());
        self.undo_stack.push(command);
    }
    pub fn undo(&mut self) {
        if let Some(mut op) = self.undo_stack.pop() {
            op.undo(&mut self.project);
            log::debug!("revert {}", op.op_name());
            self.redo_stack.push(op);
        } else {
            log::debug!("no more in undo stack")
        }
    }
    pub fn redo(&mut self) {
        if let Some(mut op) = self.redo_stack.pop() {
            op.redo(&mut self.project);
            log::debug!("redo {}", op.op_name());
            self.undo_stack.push(op);
        } else {
            log::debug!("no more in redo stack.");
        }
    }
}
