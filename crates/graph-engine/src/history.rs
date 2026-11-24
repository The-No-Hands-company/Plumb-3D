use crate::SceneGraph;

pub trait Command: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &str;
    fn execute(&mut self, scene: &mut SceneGraph) -> Result<(), String>;
    fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String>;
}

/// Transactional undo/redo history manager (Blender Operator & 3ds Max Command pattern)
#[derive(Debug)]
pub struct HistoryManager {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
    pub max_history: usize,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 64,
        }
    }
}

impl HistoryManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_name(&self) -> Option<&str> {
        self.undo_stack.last().map(|c| c.name())
    }

    pub fn redo_name(&self) -> Option<&str> {
        self.redo_stack.last().map(|c| c.name())
    }

    /// Execute a new command and push it onto the undo stack, clearing redo
    pub fn execute(
        &mut self,
        mut command: Box<dyn Command>,
        scene: &mut SceneGraph,
    ) -> Result<(), String> {
        command.execute(scene)?;
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(command);
        self.redo_stack.clear();
        Ok(())
    }

    /// Undo the most recent command
    pub fn undo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(mut command) = self.undo_stack.pop() {
            command.undo(scene)?;
            self.redo_stack.push(command);
            Ok(())
        } else {
            Err("No actions to undo".to_string())
        }
    }

    /// Redo the previously undone command
    pub fn redo(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        if let Some(mut command) = self.redo_stack.pop() {
            command.execute(scene)?;
            self.undo_stack.push(command);
            Ok(())
        } else {
            Err("No actions to redo".to_string())
        }
    }
}
