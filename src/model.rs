use serde::{Deserialize, Serialize};
use uuid::Uuid;
use clap::{Parser, ValueEnum};

pub type TodoID = Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub id: TodoID,
    pub title: String,
    pub completed: bool,
    pub priority: Priority,
    pub tags: Option<Vec<String>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} priority: {:?}, completed: {}, created_at: {}, updated_at: {}, tags: {}",
            self.title,
            self.priority,
            self.completed,
            self.created_at,
            self.updated_at,
            self.tags.as_deref().map(|tags| tags.join(", ")).as_deref().unwrap_or(""),
        ))
    }
}

impl Todo {
    fn new(title: String, priority: Priority) -> Self {
        let now_utc: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
        Todo { id: Default::default(),
            title,
            completed: false,
            priority,
            tags: None,
            created_at: now_utc.clone(),
            updated_at: now_utc
        }
    }

    pub fn matches_tags(&self, tags: &[String]) -> bool {
        if tags.is_empty() {
            return true;
        }
        // don't expect a lot of tags so O(n)^2 should be faster than hashing
        let this_tags: &[String] = self.tags.as_deref().unwrap_or_default();
        if this_tags.is_empty() {
            return false;
        }

        tags.iter()
            .map(|tag| tag.to_lowercase())
            .any(|tag| this_tags.contains(&tag))
    }

    pub fn add_tags(&mut self, tags: Option<Vec<String>>) {
        if let Some(tags) = tags {
            let mut existing = self.tags.take().unwrap_or_default();
            existing.extend(tags);
            existing.iter_mut().for_each(|t| *t = t.to_lowercase());
            existing.sort();
            existing.dedup();
            self.tags = Some(existing);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ValueEnum)]
pub enum Priority {
    Low,
    Medium,
    High,
}

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
}

#[derive(Debug, Clone, PartialEq, Parser)]
pub struct AddArgs {
    pub title: String,

    #[arg(short='p', long, default_value = "medium")]
    pub priority: Priority,
    #[arg(short='t', long)]
    pub tags: Option<Vec<String>>,
}

impl Into<Todo> for AddArgs {
    fn into(self) -> Todo {
        let mut todo = Todo::new(self.title, self.priority);
        todo.add_tags(self.tags);
        todo
    }
}

#[derive(Debug, Clone, PartialEq, Parser)]
pub struct ListArgs {
    #[arg(short='v', long, action = clap::ArgAction::SetTrue)]
    pub verbose: Option<bool>,
    #[arg(short='t', long)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Parser)]
pub struct EditArgs {
    pub pos: usize,

    #[arg(short='p', long)]
    pub priority: Option<Priority>,
    #[arg(short='t', long)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Parser)]
pub struct CompleteArgs {
    pub pos: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_tags_with_no_tags() {
        let todo = Todo::new("test".to_string(), Priority::Medium);
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        assert!(!todo.matches_tags(&tags));
    }

    #[test]
    fn test_matches_tags_with_no_new_tags() {
        let mut todo = Todo::new("test".to_string(), Priority::Medium);
        todo.add_tags(Some(vec!["tag1".to_string()]));
        let tags = vec![];
        assert!(todo.matches_tags(&tags));
    }

    #[test]
    fn test_matches_tags_with_no_matching_tags() {
        let mut todo = Todo::new("test".to_string(), Priority::Medium);
        todo.add_tags(Some(vec!["tag1".to_string()]));
        let tags = vec!["tag2".to_string()];
        assert!(!todo.matches_tags(&tags));
    }

    #[test]
    fn test_matches_tags_with_matching_tags() {
        let mut todo = Todo::new("test".to_string(), Priority::Medium);
        todo.add_tags(Some(vec!["tag1".to_string(), "tag2".to_string()]));
        let tags = vec!["tag2".to_string(), "tag3".to_string()];
        assert!(todo.matches_tags(&tags));
    }

    #[test]
    fn test_matches_tags_with_matching_tags_ignore_case() {
        let mut todo = Todo::new("test".to_string(), Priority::Medium);
        todo.add_tags(Some(vec!["taG1".to_string(), "taG2".to_string()]));
        let tags = vec!["tag3".to_string(), "tag2".to_string()];
        assert!(todo.matches_tags(&tags));
    }
}
