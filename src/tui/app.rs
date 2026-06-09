use std::io::stdout;

use crate::{io::save_todos, model::Todo};
use anyhow::{Context, Result};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

#[derive(Debug)]
pub struct UiApp {
    pub todos: Vec<Todo>,
    pub state: ratatui::widgets::ListState,
    pub mode: InputMode,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Changing {
        step: ModStep,
        buffer: String,
        index: Option<usize>,
        todo: Todo,
    },
    ConfirmingDelete {
        index: usize,
    },
    // Filtering,
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

    pub fn parse<'a>(&self, input: &str, todo: &'a mut Todo) -> Result<()> {
        // write state machine for parsing input
        match self {
            ModStep::Todo => {
                todo.title = if input.is_empty() {
                    "New Task".to_string()
                } else {
                    input.to_string()
                };
                Ok(())
            }
            ModStep::Priority => {
                if input.is_empty() {
                    return Ok(());
                }

                let priority = input
                    .parse()
                    .with_context(|| format!("Cannot parse priority: {}", input))?;
                todo.priority = priority;
                Ok(())
            }
            ModStep::Tags => {
                let tags: Vec<String> = input.split(',').map(|t| t.trim().to_string()).collect();
                if !tags.is_empty() {
                    todo.add_tags(Some(tags));
                }
                Ok(())
            }
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
                KeyCode::Char('a') => {
                    self.mode = InputMode::Changing {
                        step: ModStep::Todo,
                        buffer: String::new(),
                        index: None,
                        todo: Todo::default(),
                    };
                }
                KeyCode::Char('d') => {
                    println!("Selected index: {:?}", self.state.selected());
                    if self.state.selected().is_some() {
                        self.mode = InputMode::ConfirmingDelete {
                            index: self.state.selected().unwrap(),
                        };
                    }
                }
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
            InputMode::Changing {
                step,
                buffer,
                index,
                todo,
            } => match key {
                KeyCode::Enter => {
                    step.parse(buffer, todo)?;

                    let Some(next) = step.next() else {
                        match index {
                            Some(i) => {
                                self.todos[*i] = todo.clone();
                            }
                            None => {
                                self.todos.push(todo.clone());
                            }
                        }
                        self.mode = InputMode::Normal;
                        return Ok(false);
                    };

                    self.mode = InputMode::Changing {
                        step: next,
                        buffer: Default::default(),
                        index: *index,
                        todo: todo.clone(),
                    };
                }
                KeyCode::Esc => {
                    self.mode = InputMode::Normal;
                }
                KeyCode::Char(c) => buffer.push(c),
                KeyCode::Backspace => {
                    buffer.pop();
                }
                _ => {}
            },
            InputMode::ConfirmingDelete { index } => {
                if key == KeyCode::Char('y') || key == KeyCode::Char('Y') {
                    self.todos.remove(*index);
                }
                self.mode = InputMode::Normal;
            } //     InputMode::EditingPriority => match key {
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
        if self.todos.is_empty() {
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
        if let Some(index) = self.state.selected() {
            self.todos[index].completed = !self.todos[index].completed;
        }
        Ok(())
    }
}
