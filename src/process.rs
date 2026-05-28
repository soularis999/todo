use crate::model::{AddArgs, EditArgs, ListArgs, Todo};
use anyhow::Result;

pub fn add(args: AddArgs, todos: &mut Vec<Todo>) -> Result<&[Todo]> {
    let todo = args.into();
    todos.push(todo);
    Ok(todos)
}

pub fn filter_for_list(args: ListArgs, todos: Vec<Todo>) -> Result<Vec<Todo>> {
    let tags: &[String] = args.tags.as_deref().unwrap_or_default();
    let verbose = args.verbose.unwrap_or(false);

    if tags.is_empty() {
        return Ok(todos)
    }

    let new_todo = todos.into_iter()
        .filter(|todo| todo.matches_tags(tags))
        .filter(|todo| verbose || !todo.completed)
        .collect::<Vec<_>>();
    Ok(new_todo)
}

pub fn edit(args: EditArgs, todos: &mut [Todo]) -> Result<&[Todo]> {
    let Some(todo): Option<&mut Todo> = todos.get_mut(args.pos) else {
        anyhow::bail!("No todo found at position {}", args.pos);
    };

    args.priority.map(|priority| todo.priority = priority);
    todo.add_tags(args.tags);

    Ok(todos)
}

pub fn set_complete(pos: usize, completed: bool, todos: &mut [Todo]) -> Result<&[Todo]> {
    let Some(todo): Option<&mut Todo> = todos.get_mut(pos) else {
        anyhow::bail!("No todo found at position {}", pos);
    };

    todo.completed = completed;
    Ok(todos)
}

#[cfg(test)]
mod tests {
    use crate::model::Priority;

    use super::*;

    #[test]
    fn test_add() {
        let mut todos = Vec::new();
        let args = AddArgs {
            title: "test title".to_string(),
            priority: Priority::Medium,
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
        };
        add(args.clone(), &mut todos).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "test title");
        assert_eq!(todos[0].priority, Priority::Medium);
        assert_eq!(todos[0].completed, false);
        assert_eq!(todos[0].tags, Some(vec!["tag1".to_string(), "tag2".to_string()]));

        add(args, &mut todos).unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[1].title, "test title");
        assert_eq!(todos[1].priority, Priority::Medium);
        assert_eq!(todos[1].completed, false);
        assert_eq!(todos[1].tags, Some(vec!["tag1".to_string(), "tag2".to_string()]));
    }

    #[test]
    fn test_add_tag_ordering_and_upper_case() {
        let mut todos = Vec::new();
        let args = AddArgs {
            title: "test title".to_string(),
            priority: Priority::Medium,
            tags: Some(vec!["taG2".to_string(), "Tag1".to_string()]),
        };
        add(args.clone(), &mut todos).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "test title");
        assert_eq!(todos[0].priority, Priority::Medium);
        assert_eq!(todos[0].completed, false);
        assert_eq!(todos[0].tags, Some(vec!["tag1".to_string(), "tag2".to_string()]));

        add(args, &mut todos).unwrap();
        assert_eq!(todos.len(), 2);
        assert_eq!(todos[1].title, "test title");
        assert_eq!(todos[1].priority, Priority::Medium);
        assert_eq!(todos[1].completed, false);
        assert_eq!(todos[1].tags, Some(vec!["tag1".to_string(), "tag2".to_string()]));

    }

    #[test]
    fn test_update() {
        let mut todos = Vec::new();
        let args = AddArgs {
            title: "test title".to_string(),
            priority: Priority::Medium,
            tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
        };
        add(args.clone(), &mut todos).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "test title");
        assert_eq!(todos[0].priority, Priority::Medium);
        assert_eq!(todos[0].completed, false);
        assert_eq!(todos[0].tags, Some(vec!["tag1".to_string(), "tag2".to_string()]));
        
        let args = EditArgs {
            priority: Some(Priority::Low),
            tags: Some(vec!["TAG3".to_string(), "TAG2".to_string()]),
            pos: 0,
        };
        
        edit(args, &mut todos).unwrap();
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].title, "test title");
        assert_eq!(todos[0].priority, Priority::Low);
        assert_eq!(todos[0].completed, false);
        assert_eq!(todos[0].tags, Some(vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]));

    }
}