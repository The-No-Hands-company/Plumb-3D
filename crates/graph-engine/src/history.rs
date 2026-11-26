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
    pub command_counter: usize,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 64,
            command_counter: 0,
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
        self.command_counter += 1;
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

    /// Returns the names of all executed commands currently in the undo stack
    pub fn undo_stack_names(&self) -> Vec<String> {
        self.undo_stack.iter().map(|c| c.name().to_string()).collect()
    }

    /// Returns the names of all previously undone commands currently in the redo stack
    pub fn redo_stack_names(&self) -> Vec<String> {
        self.redo_stack.iter().map(|c| c.name().to_string()).collect()
    }

    /// Returns total number of undoable actions
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Returns total number of redoable actions
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Roll back to the undo step at `target_index` (0 <= target_index < undo_stack.len())
    pub fn jump_to_undo_step(&mut self, scene: &mut SceneGraph, target_index: usize) -> Result<(), String> {
        if target_index >= self.undo_stack.len() {
            return Err("Target history index out of bounds".into());
        }
        let steps_to_undo = self.undo_stack.len() - 1 - target_index;
        for _ in 0..steps_to_undo {
            self.undo(scene)?;
        }
        Ok(())
    }

    /// Roll back all executed commands in the undo stack to the initial scene state
    pub fn jump_to_initial_state(&mut self, scene: &mut SceneGraph) -> Result<(), String> {
        while self.can_undo() {
            self.undo(scene)?;
        }
        Ok(())
    }

    /// Roll forward into the redo stack by `steps`
    pub fn jump_forward_redo(&mut self, scene: &mut SceneGraph, steps: usize) -> Result<(), String> {
        let max_steps = steps.min(self.redo_stack.len());
        for _ in 0..max_steps {
            self.redo(scene)?;
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyCommand(&'static str);

    impl Command for DummyCommand {
        fn name(&self) -> &str {
            self.0
        }
        fn execute(&mut self, _scene: &mut SceneGraph) -> Result<(), String> {
            Ok(())
        }
        fn undo(&mut self, _scene: &mut SceneGraph) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_history_manager_jump() {
        let mut history = HistoryManager::new();
        let mut scene = SceneGraph::new();

        history.execute(Box::new(DummyCommand("Step 1")), &mut scene).unwrap();
        history.execute(Box::new(DummyCommand("Step 2")), &mut scene).unwrap();
        history.execute(Box::new(DummyCommand("Step 3")), &mut scene).unwrap();

        assert_eq!(history.undo_count(), 3);
        assert_eq!(history.undo_stack_names(), vec!["Step 1".to_string(), "Step 2".to_string(), "Step 3".to_string()]);

        // Jump back to Step 1 (index 0)
        history.jump_to_undo_step(&mut scene, 0).unwrap();
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.undo_stack_names(), vec!["Step 1".to_string()]);
        assert_eq!(history.redo_count(), 2);

        // Jump forward 1 step in redo
        history.jump_forward_redo(&mut scene, 1).unwrap();
        assert_eq!(history.undo_count(), 2);
        assert_eq!(history.undo_stack_names(), vec!["Step 1".to_string(), "Step 2".to_string()]);

        // Jump to initial state
        history.jump_to_initial_state(&mut scene).unwrap();
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 3);
    }
}
