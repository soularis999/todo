use serde::{Deserialize, Deserializer, Serialize, ser::SerializeSeq};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display},
    ops::Index,
    str::FromStr,
};

pub type TodoID = usize;

#[derive(Debug, Clone, PartialEq)]
pub struct Todos {
    todos: BTreeMap<TodoID, Todo>,
}

impl Todos {
    pub fn new() -> Self {
        Self {
            todos: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, args: impl Into<Todo>) -> anyhow::Result<TodoID> {
        let mut todo: Todo = args.into();
        // create a new id
        let id = self.todos.keys().max().copied().unwrap_or_default() + 1;
        todo.id = id;
        self.todos.insert(id, todo);
        Ok(id)
    }

    pub fn get(&self, id: &TodoID) -> Option<&Todo> {
        self.todos.get(id)
    }

    pub fn get_mut<'a>(&'a mut self, id: &TodoID) -> Option<EditTodo<'a>> {
        let todo = self.todos.get_mut(id)?;
        Some(EditTodo::borrow(todo))
    }

    pub fn remove(&mut self, id: &TodoID) -> Option<Todo> {
        self.todos.remove(id)
    }

    pub fn len(&self) -> usize {
        self.todos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.todos.is_empty()
    }

    pub fn filter(&self) -> anyhow::Result<Vec<&Todo>> {
        // let tags: &[String] = args.tags.as_deref().unwrap_or_default();
        // let verbose = args.verbose.unwrap_or(false);
        //
        // if tags.is_empty() {
        // return Ok(todos)
        // }
        //
        let new_todo = self
            .todos
            .iter()
            .map(|(_id, todo)| todo)
            // .filter(|todo| todo.matches_tags(tags))
            // .filter(|todo| verbose || !todo.completed)
            .collect::<Vec<_>>();
        Ok(new_todo)
    }
}

/**
 * Implement Index for Todos, allowing access to an individual Todo by its ID using `[]`.
 *
 * # Examples
 *
 * ```
 * use todo::model::Todos;
 * use todo::model::Priority;
 * 
 * let mut todos = Todos::new();
 * let todo_item = todo::make_todo!("Example", Priority::Medium).unwrap().into_owned();
 * let id = todos.add(todo_item).unwrap();
 *
 * // Accessing by index (ID)
 * let retrieved_todo = &todos[&id];
 * assert_eq!(retrieved_todo.title, "Example");
 * ```
 */
impl Index<&TodoID> for Todos {
    type Output = Todo;

    fn index(&self, index: &TodoID) -> &Self::Output {
        &self.todos[index]
    }
}

impl std::fmt::Display for Todos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            self.todos
                .values()
                .map(|todo| format!("{todo}\n"))
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

impl Default for Todos {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for Todos {
    /// Serializes `Todos` into a JSON array of its contained `Todo`s.
    /// Each element in the resulting sequence corresponds to one item stored internally, preserving their original order.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.todos.len()))?;

        for (_key, todo) in &self.todos {
            seq.serialize_element(todo)?;
        }

        seq.end()
    }
}

/// Deserializes `Todos` from a JSON array of `Todo` items.
/// Each element in the JSON sequence is deserialized into a `Todo` instance and added to the `Todos` collection.
impl<'de> Deserialize<'de> for Todos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // We use our custom visitor to handle the reading of the sequence.
        deserializer.deserialize_seq(TodoMapVisitor)
    }
}

struct TodoMapVisitor;

impl<'de> serde::de::Visitor<'de> for TodoMapVisitor {
    type Value = Todos; // The final type we are trying to construct: Todos struct

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a JSON array of Todo items")
    }

    // This function handles the actual sequence reading logic (the `[...]` part of the JSON).
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut todos = Todos::new();

        // Loop through the sequence element by element. The `next_element::<Todo>()?` call
        // reads one Todo instance directly, avoiding intermediate collection into a Vec.
        while let Some(todo_item) = seq.next_element::<Todo>()? {
            if let Err(e) = todos.add(todo_item) {
                // Add context to make error more meaningful for callers
                return Err(serde::de::Error::custom(format!(
                    "failed to add todo: {}",
                    e
                )));
            }
        }

        // Once all elements are read and placed in the map, construct and return Todos.
        Ok(todos)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn as_str(&self) -> &str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Priority {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "" => Ok(Priority::default()),
            _ => anyhow::bail!("Invalid priority: {}", s),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Todo {
    pub id: TodoID,
    pub title: String,
    pub completed: bool,
    pub priority: Priority,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    tags: BTreeSet<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl std::fmt::Display for Todo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{} priority: {:?}, completed: {}, created_at: {}, updated_at: {}, tags: {}",
            self.title,
            self.priority,
            self.completed,
            self.created_at,
            self.updated_at,
            self.tags
                .iter()
                .map(|tag| tag.as_str())
                .collect::<Vec<&str>>()
                .join(", "),
        ))
    }
}

impl Todo {
    /**
     * Creates a new todo with default values.
     *
     * Example:
     * ```
     * use todo::model::Priority;
     * use todo::model::Todo;
     *
     * let edit_todo = todo::make_todo!("Buy groceries", Priority::High).unwrap().into_owned();
     * assert_eq!(edit_todo.title, "Buy groceries");
     * assert_eq!(edit_todo.priority, Priority::High);
     * ```
     */
    pub fn new() -> EditTodo<'static> {
        EditTodo::owned(Todo::default())
    }

    /**
     * Returns an [`EditTodo`] that can be used to edit the todo.
     * The [`EditTodo`] is owned by the caller, and can be used to edit the todo.
     *
     * Example:
     * ```
     * use todo::model::Todo;
     *
     * let mut todo = todo::make_todo!().unwrap().into_owned();
     * 
     * let mut edit_todo = todo.edit();
     * edit_todo.title = Some("Buy groceries".to_string());
     * 
     * let todo = edit_todo.finish().unwrap();
     * assert_eq!(todo.title, "Buy groceries");
     * ```
     */
    pub fn edit(self) -> EditTodo<'static> {
        EditTodo::owned(self)
    }

    /**
     * Returns an iterator over the tags of the todo.
     * Tags are returned as `&str` references.
     *
     * Example:
     * ```
     * use todo::model::Todo;
     *
     * let todo = todo::make_todo!().unwrap();
     * assert_eq!(todo.get_tags().into_iter().collect::<Vec<&str>>(), Vec::<&str>::new());
     * ```
     */
    pub fn get_tags(&self) -> impl IntoIterator<Item = &str> {
        self.tags.iter().map(|t| t.as_str())
    }

    /**
     * Returns `true` if the todo has any of the given tags
     * Tags are compared case-sensitively.
     *
     * Example:
     * ```
     * let todo = todo::make_todo!("test", todo::model::Priority::High, vec!["foo", "bar"]).unwrap();
     * assert!(todo.has_any(["foo", "bar"]));
     * ```
     */
    pub fn has_any<'a>(&self, tags: impl IntoIterator<Item = &'a str>) -> bool {
        let this_tags: &BTreeSet<String> = &self.tags;
        let mut tag_exists = true;
        for tag in tags {
            let tag_lower: String = tag.to_lowercase();
            tag_exists = this_tags.contains(&tag_lower);
            if tag_exists {
                break;
            }
        }
        tag_exists
    }
}

/// Internal enum to avoid Cow's mandatory clone-on-write.
#[derive(Debug, PartialEq)]
enum TodoSource<'a> {
    Borrowed(&'a mut Todo),
    Owned(Todo),
}

#[derive(Debug, PartialEq)]
pub struct EditTodo<'a> {
    pub title: Option<String>,
    pub priority: Option<Priority>,
    pub completed: Option<bool>,
    pub tags: Option<Vec<String>>,

    todo: TodoSource<'a>,
}

impl<'a> EditTodo<'a> {
    pub fn borrow(todo: &'a mut Todo) -> Self {
        Self {
            title: None,
            priority: None,
            completed: None,
            tags: None,
            todo: TodoSource::Borrowed(todo),
        }
    }

    pub fn owned(todo: Todo) -> Self {
        Self {
            title: None,
            priority: None,
            completed: None,
            tags: None,
            todo: TodoSource::Owned(todo),
        }
    }

    pub fn finish(self) -> anyhow::Result<Cow<'a, Todo>> {
        // Destructure self so we can match on `todo` without partial-move errors.
        let Self {
            title,
            priority,
            completed,
            tags,
            todo,
        } = self;

        match todo {
            TodoSource::Borrowed(todo) => {
                Self::apply(todo, title, priority, completed, tags);
                // Return a borrowed Cow pointing at the original, mutated Todo.
                Ok(Cow::Borrowed(&*todo))
            }
            TodoSource::Owned(mut todo) => {
                Self::apply(&mut todo, title, priority, completed, tags);
                Ok(Cow::Owned(todo))
            }
        }
    }

    /// Mutates `todo` in place using the values captured from `self`.
    fn apply(
        todo: &mut Todo,
        title: Option<String>,
        priority: Option<Priority>,
        completed: Option<bool>,
        tags: Option<Vec<String>>,
    ) {
        if let Some(t) = title {
            todo.title = t.to_string();
        }
        if let Some(p) = priority {
            todo.priority = p;
        }
        if let Some(c) = completed {
            todo.completed = c;
        }

        if let Some(tags) = tags {
            todo.tags.clear();
            todo.tags.extend(tags.into_iter().map(|t| t.to_lowercase()));
        }

        todo.updated_at = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use anyhow::Ok;

    use crate::{
        make_todo,
        model::{Priority, Todo, Todos},
    };

    #[test]
    fn test_matches_tags_with_no_tags() -> anyhow::Result<()> {
        let todo = make_todo!("test", Priority::High, vec!["tag2", "TAG1"])?.into_owned();

        assert_eq!("test", todo.title);
        assert_eq!(Priority::High, todo.priority);
        assert!(!todo.completed);

        assert!(todo.has_any(vec![]));
        assert!(todo.has_any(vec!["tag1"]));
        assert!(todo.has_any(vec!["Tag1"]));
        assert!(todo.has_any(vec!["tag2"]));
        assert!(todo.has_any(vec!["taG2"]));
        assert!(!todo.has_any(vec!["tag3"]));

        assert!(todo.has_any(vec!["tag2", "tag3"]));
        assert!(todo.has_any(vec!["tag3", "tag1"]));

        assert_eq!(vec!["tag1", "tag2"], todo.tags.iter().collect::<Vec<_>>());

        Ok(())
    }

    // ********************** todos tests ***********************

    #[test]
    fn test_add() -> anyhow::Result<()> {
        let mut todos = Todos::new();
        assert_eq!(todos.len(), 0);
        assert!(todos.is_empty());

        let todo: Cow<Todo> = make_todo!("test title", Priority::Medium, vec!["tag1", "tag2"])?;
        todos.add(todo.into_owned())?;

        assert_eq!(todos.len(), 1);
        assert!(!todos.is_empty());
        assert_eq!(todos[&1usize].title, "test title");
        assert_eq!(todos[&1usize].priority, Priority::Medium);
        assert_eq!(todos[&1usize].completed, false);
        assert_eq!(todos[&1usize].tags.len(), 2);
        assert!(todos[&1usize].tags.contains(&"tag1".to_string()));
        assert!(todos[&1usize].tags.contains(&"tag2".to_string()));

        let todo2: Cow<Todo> = make_todo!("test title2", Priority::High, vec!["tag2", "tag3"])?;
        todos.add(todo2.into_owned())?;
        assert_eq!(todos.len(), 2);
        assert!(!todos.is_empty());
        assert_eq!(todos.get(&2usize).unwrap().title, "test title2");
        assert_eq!(todos.get(&2usize).unwrap().priority, Priority::High);
        assert_eq!(todos.get(&2usize).unwrap().completed, false);
        assert_eq!(todos.get(&2usize).unwrap().tags.len(), 2);
        assert!(
            todos
                .get(&2usize)
                .unwrap()
                .tags
                .contains(&"tag2".to_string())
        );
        assert!(
            todos
                .get(&2usize)
                .unwrap()
                .tags
                .contains(&"tag3".to_string())
        );

        assert_eq!(None, todos.get(&3usize));
        assert_eq!(None, todos.get_mut(&3usize));

        Ok(())
    }

    #[test]
    fn test_update() -> anyhow::Result<()> {
        let mut todos = Todos::new();
        let todo: Cow<Todo> = make_todo!("test title", Priority::Medium, vec!["tag1", "tag2"])?;
        todos.add(todo.into_owned())?;

        assert_eq!(todos[&1usize].title, "test title");
        assert_eq!(todos[&1usize].priority, Priority::Medium);
        assert_eq!(todos[&1usize].completed, false);
        assert_eq!(todos[&1usize].tags.len(), 2);
        assert!(todos[&1usize].tags.contains(&"tag1".to_string()));
        assert!(todos[&1usize].tags.contains(&"tag2".to_string()));

        // if the finish() is not called, the changes should not be applied
        {
            let mut edit_todo = todos.get_mut(&1usize).unwrap();
            edit_todo.title = Some("updated title".to_string());
            edit_todo.priority = Some(Priority::High);
            edit_todo.tags = Some(vec![
                "tag3".to_string(),
                "tag4".to_string(),
                "tag5".to_string(),
            ]);
            edit_todo.completed = Some(true);
        }

        assert_eq!(todos[&1usize].title, "test title");
        assert_eq!(todos[&1usize].priority, Priority::Medium);
        assert_eq!(todos[&1usize].completed, false);
        assert_eq!(todos[&1usize].tags.len(), 2);
        assert!(todos[&1usize].tags.contains(&"tag1".to_string()));
        assert!(todos[&1usize].tags.contains(&"tag2".to_string()));

        {
            let mut edit_todo = todos.get_mut(&1usize).unwrap();
            edit_todo.title = Some("updated title".to_string());
            edit_todo.priority = Some(Priority::High);
            edit_todo.tags = Some(vec![
                "tag3".to_string(),
                "tag4".to_string(),
                "tag5".to_string(),
            ]);
            edit_todo.completed = Some(true);

            let updated_tag = edit_todo.finish()?;

            assert_eq!(updated_tag.title, "updated title");
            assert_eq!(updated_tag.priority, Priority::High);
            assert_eq!(updated_tag.completed, true);
            assert_eq!(updated_tag.tags.len(), 3);
            assert!(updated_tag.tags.contains(&"tag3".to_string()));
            assert!(updated_tag.tags.contains(&"tag4".to_string()));
            assert!(updated_tag.tags.contains(&"tag5".to_string()));
        }

        assert_eq!(todos[&1usize].title, "updated title");
        assert_eq!(todos[&1usize].priority, Priority::High);
        assert_eq!(todos[&1usize].completed, true);
        assert_eq!(todos[&1usize].tags.len(), 3);
        assert!(todos[&1usize].tags.contains(&"tag3".to_string()));
        assert!(todos[&1usize].tags.contains(&"tag4".to_string()));
        assert!(todos[&1usize].tags.contains(&"tag5".to_string()));

        Ok(())
    }

    #[test]
    fn test_complete() -> anyhow::Result<()> {
        let mut todos = Todos::new();
        let todo: Cow<Todo> = make_todo!("test title", Priority::Medium, vec!["tag1", "tag2"])?;
        todos.add(todo.into_owned())?;

        assert_eq!(todos[&1usize].title, "test title");
        assert_eq!(todos[&1usize].priority, Priority::Medium);
        assert_eq!(todos[&1usize].completed, false);
        assert_eq!(todos[&1usize].tags.len(), 2);
        assert!(todos[&1usize].tags.contains(&"tag1".to_string()));
        assert!(todos[&1usize].tags.contains(&"tag2".to_string()));

        // if the finish() is not called, the changes should not be applied
        {
            let mut edit_todo = todos.get_mut(&1usize).unwrap();
            edit_todo.completed = Some(true);
        }

        assert_eq!(todos[&1usize].title, "test title");
        assert_eq!(todos[&1usize].priority, Priority::Medium);
        assert_eq!(todos[&1usize].completed, false);
        assert_eq!(todos[&1usize].tags.len(), 2);
        assert!(todos[&1usize].tags.contains(&"tag1".to_string()));
        assert!(todos[&1usize].tags.contains(&"tag2".to_string()));

        {
            let mut edit_todo = todos.get_mut(&1usize).unwrap();
            edit_todo.completed = Some(true);
            let updated_tag = edit_todo.finish()?;

            assert_eq!(updated_tag.title, "test title");
            assert_eq!(updated_tag.priority, Priority::Medium);
            assert_eq!(updated_tag.completed, true);
            assert_eq!(updated_tag.tags.len(), 2);
            assert!(updated_tag.tags.contains(&"tag1".to_string()));
            assert!(updated_tag.tags.contains(&"tag2".to_string()));
        }

        assert_eq!(todos[&1usize].title, "test title");
        assert_eq!(todos[&1usize].priority, Priority::Medium);
        assert_eq!(todos[&1usize].completed, true);
        assert_eq!(todos[&1usize].tags.len(), 2);
        assert!(todos[&1usize].tags.contains(&"tag1".to_string()));
        assert!(todos[&1usize].tags.contains(&"tag2".to_string()));

        Ok(())
    }

    #[test]
    fn test_remove() -> anyhow::Result<()> {
        let mut todos = Todos::new();
        assert_eq!(todos.len(), 0);
        assert!(todos.is_empty());

        let todo: Cow<Todo> = make_todo!("test title", Priority::Medium, vec!["tag1", "tag2"])?;
        todos.add(todo.clone().into_owned())?;

        assert_eq!(todos.len(), 1);
        assert!(!todos.is_empty());

        assert_eq!(None, todos.remove(&2usize));
        assert_eq!(todos.len(), 1);
        assert!(!todos.is_empty());

        let owned: Todo = todos.remove(&1usize).unwrap();
        assert_eq!(todo.title, owned.title);
        assert_eq!(todo.priority, owned.priority);
        assert_eq!(todo.completed, owned.completed);
        assert_eq!(todo.tags.len(), owned.tags.len());
        assert!(todo.tags.iter().all(|t| owned.tags.contains(t)));

        assert_eq!(todos.len(), 0);
        assert!(todos.is_empty());
        Ok(())
    }

    // #[test]
    // fn test_update() {
    //     let mut todos = Todos::new();
    //     let args = AddArgs {
    //         title: "test title".to_string(),
    //         priority: Priority::Medium,
    //         tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
    //     };
    //     todos.add(args.clone()).unwrap();
    //     assert_eq!(todos.todos.len(), 1);
    //     assert_eq!(todos.todos[&0usize].title, "test title");
    //     assert_eq!(todos.todos[&0usize].priority, Priority::Medium);
    //     assert_eq!(todos.todos[&0usize].completed, false);
    //     assert_eq!(todos.todos[&0usize].tags.len(), 2);
    //     assert!(todos.todos[&0usize].tags.contains(&"tag1".to_string()));
    //     assert!(todos.todos[&0usize].tags.contains(&"tag2".to_string()));

    //     let args = EditArgs {
    //         priority: Some(Priority::Low),
    //         tags: Some(vec!["TAG3".to_string(), "TAG2".to_string()]),
    //         id: 0,
    //     };

    //     todos.edit(args).unwrap();
    //     assert_eq!(todos.todos.len(), 1);
    //     assert_eq!(todos.todos[&0usize].title, "test title");
    //     assert_eq!(todos.todos[&0usize].priority, Priority::Low);
    //     assert_eq!(todos.todos[&0usize].completed, false);
    //     assert_eq!(todos.todos[&0usize].tags.len(), 3);
    //     assert!(todos.todos[&0usize].tags.contains(&"tag1".to_string()));
    //     assert!(todos.todos[&0usize].tags.contains(&"tag2".to_string()));
    //     assert!(todos.todos[&0usize].tags.contains(&"tag3".to_string()));
    // }
}
