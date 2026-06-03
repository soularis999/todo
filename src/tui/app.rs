use std::io::stdout;

use crate::model::Todo;
use anyhow::Result;
use crossterm::{ExecutableCommand, event::{self, Event, KeyCode, KeyEventKind}, terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode}};
use ratatui::{Terminal, prelude::CrosstermBackend};


#[derive(Debug)]
pub struct UiApp {
    pub todos: Vec<Todo>,
    pub state: ratatui::widgets::ListState,
    pub mode: InputMode,
    pub input_buffer: String,
    pub input_prompt: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    // Adding,
    // ConfirmingDelete,
    // EditingPriority,
    // Filtering,
}

impl UiApp {
    pub fn new(todos: Vec<Todo>) -> Self {
        let state = ratatui::widgets::ListState::default();
        Self {
            todos,
            state,
            mode: InputMode::Normal,
            input_buffer: String::new(),
            input_prompt: String::new(),
        }
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

    fn run_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|frame| super::ui::draw(frame, self))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if self.handle_key(key.code)? {
                        return Ok(());
                    }
                }
            }
        }
    }

    fn handle_key(&mut self, key: KeyCode) -> Result<bool> {
        match self.mode {
            InputMode::Normal => match key {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
                KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
                KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
        //         KeyCode::Char('g') => self.state.select(Some(0)),
        //         KeyCode::Char('G') => {
        //             let last = self.filtered_indices.len().saturating_sub(1);
        //             self.state.select(Some(last));
        //         }
        //         KeyCode::Char(' ') => self.toggle_complete()?,
        //         KeyCode::Char('a') => {
        //             self.input_mode = InputMode::Adding;
        //             self.input_prompt = "New task: ".to_string();
        //             self.input_buffer.clear();
        //         }
        //         KeyCode::Char('d') => {
        //             if self.get_selected_id().is_some() {
        //                 self.input_mode = InputMode::ConfirmingDelete;
        //                 self.input_prompt = "Delete? (y/n): ".to_string();
        //                 self.input_buffer.clear();
        //             }
        //         }
        //         KeyCode::Char('e') => {
        //             if self.get_selected_id().is_some() {
        //                 self.input_mode = InputMode::EditingPriority;
        //                 self.input_prompt = "Priority (low/medium/high): ".to_string();
        //                 self.input_buffer.clear();
        //             }
        //         }
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
        //     InputMode::Adding => match key {
        //         KeyCode::Enter => {
        //             let input = self.input_buffer.trim();
        //             if !input.is_empty() {
        //                 let (title, priority, tags) = self.parse_task_input(input);
        //                 operations::add_task(&mut self.tasks, title, priority, tags);
        //                 TodoStore::save(&self.tasks)?;
        //                 self.apply_filter();
        //                 self.set_message("Task added");
        //             }
        //             self.input_mode = InputMode::Normal;
        //             self.input_buffer.clear();
        //         }
        //         KeyCode::Esc => {
        //             self.input_mode = InputMode::Normal;
        //             self.input_buffer.clear();
        //         }
        //         KeyCode::Char(c) => self.input_buffer.push(c),
        //         KeyCode::Backspace => { self.input_buffer.pop(); }
        //         _ => {}
        //     },
        //     InputMode::ConfirmingDelete => {
        //         if key == KeyCode::Char('y') || key == KeyCode::Char('Y') {
        //             if let Some(id) = self.get_selected_id() {
        //                 operations::delete_task(&mut self.tasks, id);
        //                 TodoStore::save(&self.tasks)?;
        //                 self.apply_filter();
        //                 self.set_message("Deleted");
        //             }
        //         }
        //         self.input_mode = InputMode::Normal;
        //         self.input_buffer.clear();
        //     }
        //     InputMode::EditingPriority => match key {
        //         KeyCode::Enter => {
        //             if let Some(id) = self.get_selected_id() {
        //                 let priority = Priority::from_str(&self.input_buffer);
        //                 operations::update_priority(&mut self.tasks, id, priority);
        //                 TodoStore::save(&self.tasks)?;
        //                 self.set_message("Priority updated");
        //             }
        //             self.input_mode = InputMode::Normal;
        //             self.input_buffer.clear();
        //         }
        //         KeyCode::Esc => {
        //             self.input_mode = InputMode::Normal;
        //             self.input_buffer.clear();
        //         }
        //         KeyCode::Char(c) => self.input_buffer.push(c),
        //         KeyCode::Backspace => { self.input_buffer.pop(); }
        //         _ => {}
        //     }
        //     InputMode::Filtering => match key {
        //         KeyCode::Enter => {
        //             let input = self.input_buffer.trim();
        //             self.filter_tag = if input.is_empty() { None } else { Some(input.to_string()) };
        //             self.apply_filter();
        //             self.set_message(&format!("Filter: {}", self.filter_tag.as_deref().unwrap_or("none")));
        //             self.input_mode = InputMode::Normal;
        //             self.input_buffer.clear();
        //         }
        //         KeyCode::Esc => {
        //             self.input_mode = InputMode::Normal;
        //             self.input_buffer.clear();
        //         }
        //         KeyCode::Char(c) => self.input_buffer.push(c),
        //         KeyCode::Backspace => { self.input_buffer.pop(); }
        //         _ => {}
        }
        Ok(false)
    }

    
    fn move_selection(&mut self, delta: isize) {
        let current = self.state.selected().unwrap_or(0);
        let new_idx = if delta < 0 {
            current.saturating_sub((-delta) as usize)
        } else {
            (current + delta as usize).min(self.todos.len() - 1)
        };
        self.state.select(Some(new_idx));
    }
}
