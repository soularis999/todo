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

    args.tags.map(|tags| todo.tags = Some(tags));
    args.priority.map(|priority| todo.priority = priority);

    Ok(todos)
}

pub fn set_complete(pos: usize, completed: bool, todos: &mut [Todo]) -> Result<&[Todo]> {
    let Some(todo): Option<&mut Todo> = todos.get_mut(pos) else {
        anyhow::bail!("No todo found at position {}", pos);
    };

    todo.completed = completed;
    Ok(todos)
}
