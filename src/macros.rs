//! Macros for conveniently creating `Todo` instances.

/// Create a new Todo with title, priority, and tags - returns the completed todo directly.
#[macro_export]
macro_rules! make_todo {
    () => {{
        let mut todo_edit = $crate::model::Todo::new();
        todo_edit.title = None;
        todo_edit.priority = None;
        todo_edit.completed = Some(false);
        todo_edit.tags = None;
        todo_edit.finish()
    }};
    
    ($title:expr) => {{
        let mut todo_edit = $crate::model::Todo::new();
        todo_edit.title = Some($title.into());
        todo_edit.priority = None;
        todo_edit.completed = Some(false);
        todo_edit.tags = None;
        todo_edit.finish()
    }};

    ($title:expr, $priority:path) => {{
        let mut todo_edit = $crate::model::Todo::new();
        todo_edit.title = Some($title.into());
        todo_edit.priority = Some($priority);
        todo_edit.completed = Some(false);
        todo_edit.tags = None;

        todo_edit.finish()
    }};

    ($title:expr, $priority:path, $tags_vec:expr) => {{
        let mut todo_edit = $crate::model::Todo::new();
        todo_edit.title = Some($title.into());
        todo_edit.priority = Some($priority);
        todo_edit.completed = Some(false);
        todo_edit.tags = Some($tags_vec.into_iter().map(|t| t.to_string()).collect());

        todo_edit.finish()
    }};
}
