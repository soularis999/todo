use anyhow::Context;

use crate::model::Todo;

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Changing(InputModeState),
    ConfirmingDelete(DeleteConfirmState),
    // Filtering,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct InputModeState {
    kind: ModStep,
    buffer: String,
    index: Option<usize>,
    todo: Todo,
}

impl InputModeState {
    pub fn new(index: Option<usize>, todo: Todo) -> Self {
        Self {
            kind: ModStep::default(),
            buffer: String::new(),
            index,
            todo,
        }
    }
    
    pub fn kind(&self) -> &ModStep {
        &self.kind
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn todo(&self) -> &Todo {
        &self.todo
    }

    pub fn push(&mut self, text: char) -> &mut Self {
        self.buffer.push(text);
        self
    }
    
    pub fn pop(&mut self) -> &mut Self {
        self.buffer.pop();
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.buffer.clear();
        self
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }
    
    pub fn parse<'a>(&mut self) -> anyhow::Result<()> {
        // write state machine for parsing input
        match self.kind {
            ModStep::Todo => {
                self.todo.title = if self.buffer.is_empty() {
                    "New Task".to_string()
                } else {
                    self.buffer.to_string()
                };
                Ok(())
            }
            ModStep::Priority => {
                let priority = self.buffer
                    .parse()
                    .with_context(|| format!("Cannot parse priority: {}", self.buffer))?;
                self.todo.priority = priority;
                Ok(())
            }
            ModStep::Tags => {
                let tags: Vec<String> = self.buffer.split(',').map(|t| t.trim().to_string()).filter(|v| !v.trim().is_empty()).collect();
                if !tags.is_empty() {
                    self.todo.add_tags(Some(tags));
                }
                Ok(())
            }
        }
    }

    pub fn advance(&mut self) -> Option<()> {
        self.kind.next().map(|next| {
            self.kind = next;
            self.buffer.clear();
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModStep {
    Todo,
    Priority,
    Tags,
}

impl ModStep {
    pub fn as_str(&self) -> &str {
        match self {
            ModStep::Todo => "Add task (and press enter): ",
            ModStep::Priority => "Add priority (or press enter for default): ",
            ModStep::Tags => "Add comma-separated tags (or press enter for default): ",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            ModStep::Todo => Some(ModStep::Priority),
            ModStep::Priority => Some(ModStep::Tags),
            ModStep::Tags => None,
        }
    }
}

impl std::fmt::Display for ModStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for ModStep {
    fn default() -> Self {
        Self::Todo
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteConfirmState {
    index: usize,
}

impl DeleteConfirmState {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
    
    pub fn index(&self) -> usize {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use crate::model::Priority;

    use super::*;

    #[test]
    fn test_mod_step_next() {
        assert_eq!(ModStep::Todo.next(), Some(ModStep::Priority));
        assert_eq!(ModStep::Priority.next(), Some(ModStep::Tags));
        assert_eq!(ModStep::Tags.next(), None);
    }

    #[test]
    fn test_mod_step_as_str() {
        assert_eq!(ModStep::Todo.as_str(), "Add task (and press enter): ");
        assert_eq!(ModStep::Priority.as_str(), "Add priority (or press enter for default): ");
        assert_eq!(ModStep::Tags.as_str(), "Add comma-separated tags (or press enter for default): ");
    }

    #[test]
    fn test_delete_confirm_state() {
        let state = DeleteConfirmState::new(42);
        assert_eq!(state.index(), 42);
    }

    #[test]
    fn test_input_mode_state() {
        let todo = Todo::default();
        let state = InputModeState::new(Some(10), todo.clone());
        assert_eq!(state.kind(), &ModStep::Todo);
        assert_eq!(state.index(), Some(10));
        assert_eq!(state.todo(), &todo);
    }

    
    #[test]
    fn test_input_state_push_pop() {
        let todo = Todo::default();
        let mut state = InputModeState::new(Some(10), todo.clone());

        assert_eq!(state.buffer(), "");
        state.push('a');
        assert_eq!(state.buffer(), "a");
        state.push('b');
        assert_eq!(state.buffer(), "ab");
        state.push('c');
        assert_eq!(state.buffer(), "abc");
        state.pop();
        assert_eq!(state.buffer(), "ab");
        state.pop();
        assert_eq!(state.buffer(), "a");
        state.pop();
        assert_eq!(state.buffer(), "");
        state.pop();
        assert_eq!(state.buffer(), "");
    }

    #[test]
    fn test_input_state_parse() -> anyhow::Result<()> {
        let todo = Todo::default();
        let mut state = InputModeState::new(Some(10), todo.clone());
        
        assert_eq!(state.kind(), &ModStep::Todo);
        assert_eq!(state.buffer(), "");
        assert_eq!(state.todo().title, "");
        assert_eq!(state.todo().priority, Priority::Medium);
        assert_eq!(state.todo().tags, None);
        
        state.push('z')
            .push('a')
            .push('z')
            .parse()?;
        assert_eq!(state.buffer(), "zaz");
        assert_eq!(state.todo().title, "zaz");
        assert_eq!(state.todo().priority, Priority::Medium);
        assert_eq!(state.todo().tags, None);
        
        state.pop()
            .parse()?;
        assert_eq!(state.buffer(), "za");
        assert_eq!(state.todo().title, "za");
        assert_eq!(state.todo().priority, Priority::Medium);
        assert_eq!(state.todo().tags, None);

        assert_eq!(state.advance(), Some(()));
        assert_eq!(state.kind(), &ModStep::Priority);
        assert_eq!(state.buffer(), "");
        assert_eq!(state.todo().title, "za");
        assert_eq!(state.todo().priority, Priority::Medium);
        assert_eq!(state.todo().tags, None);
        
        assert_eq!(state.push('z')
            .push('a')
            .parse()
            .err()
            .unwrap()
            .to_string(), "Cannot parse priority: za");
        state
            .clear()
            .push('h').push('i').push('g').push('h')
            .parse()?;
        assert_eq!(state.buffer(), "high");
        assert_eq!(state.todo().title, "za");
        assert_eq!(state.todo().priority, Priority::High);
        assert_eq!(state.todo().tags, None);

        // test default - pop all t
        state.clear().parse()?;
        assert_eq!(state.buffer(), "");
        assert_eq!(state.todo().title, "za");
        assert_eq!(state.todo().priority, Priority::Medium);
        assert_eq!(state.todo().tags, None);

        assert_eq!(state.advance(), Some(()));
        assert_eq!(state.kind(), &ModStep::Tags);
        assert_eq!(state.buffer(), "");
        state.clear().parse()?;
        assert_eq!(state.todo().title, "za");
        assert_eq!(state.todo().priority, Priority::Medium);
        assert_eq!(state.todo().tags, None); // this is not correct -> should take from the original todo passed

        state.push('a').push(',').push('b').push(','); // test trailing commas
        state.parse()?;
        assert_eq!(state.todo().title, "za");
        assert_eq!(state.todo().priority, Priority::Medium);
        assert_eq!(state.todo().tags, Some(vec!["a".to_string(), "b".to_string()]));
        
        assert_eq!(state.advance(), None);
        assert_eq!(state.kind(), &ModStep::Tags);
        assert_eq!(state.buffer(), "a,b,");
        Ok(())
    }
}