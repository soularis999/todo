use std::borrow::Cow;

use clap::Parser;

use crate::{make_todo, model::{Priority, Todo, TodoID}};

#[derive(Debug, Clone, PartialEq, Parser)]
#[command(name = "todo")]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, PartialEq, Parser)]
pub enum Commands {
    #[command(alias = "ad")]
    Add(AddArgs),
    #[command(alias = "ls")]
    List(ListArgs),
    #[command(alias = "md")]
    Edit(EditArgs),
    #[command(alias = "cp")]
    Complete(CompleteArgs),
    #[command(alias = "ucp")]
    UnComplete(CompleteArgs),
    #[cfg(feature = "tui")]
    #[command(alias = "t")]
    Tui,
}

#[derive(Debug, Clone, PartialEq, Parser)]
pub struct AddArgs {
    pub title: String,

    #[arg(short = 'p', long, default_value = "medium")]
    pub priority: Priority,
    #[arg(short = 't', long)]
    pub tags: Option<Vec<String>>,
}

impl TryInto<Todo> for AddArgs {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Todo, Self::Error> {
        let title: &str = &self.title;
        let priority: Priority = self.priority;
        let tags: Option<Vec<String>> = self.tags;
        let todo: Cow<Todo> = make_todo!(title, priority, tags.as_deref().unwrap_or_default())?;
        Ok(todo.into_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Parser)]
pub struct ListArgs {
    #[arg(short='v', long, action = clap::ArgAction::SetTrue)]
    pub verbose: Option<bool>,
    #[arg(short = 't', long)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Parser)]
pub struct EditArgs {
    pub id: TodoID,

    #[arg(short = 'p', long)]
    pub priority: Option<Priority>,
    #[arg(short = 't', long)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Parser)]
pub struct CompleteArgs {
    pub id: TodoID,
}

// pub struct CommandResult {
//     todos: Vec<Todo>,
// }