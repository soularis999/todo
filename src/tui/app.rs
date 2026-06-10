use std::io::stdout;

use crate::{io::save_todos, model::Todo};
use crate::tui::model::{DeleteConfirmState, InputMode, InputModeState};
use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

#[derive(Debug)]
pub struct UiApp {
    todos: Vec<Todo>,
    state: ratatui::widgets::ListState,
    mode: InputMode,
    error: Option<String>,
}

impl UiApp {
    pub fn new(todos: Vec<Todo>) -> Self {
        let state = ratatui::widgets::ListState::default();
        Self {
            todos,
            state,
            mode: InputMode::Normal,
            error: None,
        }
    }

    pub fn todo_iter(&self) -> impl Iterator<Item = &Todo> {
        self.todos.iter()
    }

    pub fn len(&self) -> usize {
        self.todos.len()
    }

    pub fn state(&self) -> &ratatui::widgets::ListState {
        &self.state
    }

    pub fn mode(&self) -> &InputMode {
        &self.mode
    }
    
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    
    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        stdout().execute(Clear(ClearType::All))?;

        let backend: CrosstermBackend<std::io::Stdout> = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal);

        disable_raw_mode()?;
        stdout().execute(Clear(ClearType::All))?;

        result
    }

    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        loop {
            terminal.draw(|frame| super::ui::draw(frame, self))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.handle_key(key.code) {
                        Ok(true) => {
                            save_todos(&self.todos)?;
                            return Ok(());
                        }
                        Ok(false) => {}
                        Err(error) => {
                            self.error = Some(error.to_string());
                        }
                    }
                }
                // might want to save at specific intervals
            }
        }
    }

    fn handle_key(&mut self, key: KeyCode) -> Result<bool> {
        match &mut self.mode {
            InputMode::Normal => match key {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
                KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
                KeyCode::Char(' ') => self.toggle_complete()?,
                KeyCode::Char('a') => self.mode = InputMode::Changing(InputModeState::default()),
                KeyCode::Char('e') => {
                    if let Some((index, todo)) = self.selected() {
                        self.mode = InputMode::Changing(InputModeState::new(Some(index), todo.clone()));
                    }
                },
                KeyCode::Char('d') => {
                    if let Some((index, _)) = self.selected() {
                        self.mode = InputMode::ConfirmingDelete(DeleteConfirmState::new(index));
                    }
                }
                //         KeyCode::Char('f') => {
                //             self.input_mode = InputMode::Filtering;
                //             self.input_prompt = "Filter by tag (empty=clear): ".to_string();
                //             self.input_buffer.clear();
                //         }
                //         KeyCode::Char('c') => {
                //             self.show_completed = !self.show_completed;
                //             self.apply_filter();
                //             self.set_message(if self.show_completed { "Showing all" } else { "Hiding completed" });
                //         }
                //         KeyCode::Char('r') => {
                //             self.tasks = TodoStore::load()?;
                //             self.apply_filter();
                //             self.set_message("Reloaded");
                //         }
                _ => {}
            },
            InputMode::Changing(state) => match key {
                KeyCode::Enter => {
                    state.parse()?;
                    let res = state.advance();
                    if None == res {
                        match state.index() {
                            Some(i) => {
                                self.todos[i] = state.todo().clone();
                            }
                            None => {
                                self.todos.push(state.todo().clone());
                            }
                        }
                        self.mode = InputMode::Normal;
                        return Ok(false);
                    }
                    return Ok(false)
                }
                KeyCode::Esc => self.mode = InputMode::Normal,
                KeyCode::Char(c) => state.push(c),
                KeyCode::Backspace => state.pop(),
                _ => {}
            },
            InputMode::ConfirmingDelete(state) => {
                if key == KeyCode::Char('y') || key == KeyCode::Char('Y') {
                    self.todos.remove(state.index());
                }
                self.mode = InputMode::Normal;
            }
        }
        Ok(false)
    }

    fn clear_selected(&mut self) {
        self.state.select(None);
    }

    fn move_selection(&mut self, delta: isize) {
        if self.todos.is_empty() {
            self.clear_selected();
            return;
        }
        
        let current = self.state.selected().unwrap_or(0);
        let new_idx = if delta < 0 {
            current.saturating_sub((-delta) as usize)
        } else {
            let max = self.todos.len().saturating_sub(1);
            (current + delta as usize).min(max)
        };
        self.state.select(Some(new_idx));
    }

    fn toggle_complete(&mut self) -> Result<()> {
        self.selected_mut().map(|(_, selected)| selected.completed = !selected.completed);
        Ok(())
    }
    
    pub fn is_selected(&self, index: usize) -> bool {
        Some(index) == self.state.selected()
    }
    pub fn selected(&self) -> Option<(usize, &Todo)> {
        self.state.selected().and_then(|index| {
            let todo = self.todos.get(index);
            todo.map(|t| (index, t))
        })
    }

    pub fn selected_mut(&mut self) -> Option<(usize, &mut Todo)> {
        self.state.selected().and_then(|index| {
            let todo = self.todos.get_mut(index);
            todo.map(|t| (index, t))
        })
    }
}
