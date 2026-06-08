use std::env;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::path;
use anyhow::Context;
use anyhow::Result;

use crate::model::Todo;

const TODOS_FILE: &str = ".todos.json";

/**
 * Loads todos from the default todos file.
 *
 * # Returns
 *
 * A `Result` containing the loaded todos, or an error if one occurred.
 */
pub fn load_todos() -> Result<Vec<Todo>> {
    let home = env::var("HOME")  // Unix
         .or_else(|_| env::var("USERPROFILE"))  // Windows
         .expect("HOME not set");
    println!("Loading from {home}/{TODOS_FILE}");
    return load_todos_from_file(format!("{home}/{TODOS_FILE}"));
}

/**
 * Loads todos from a specified file.
 *
 * # Returns
 *
 * A `Result` containing the loaded todos, or an error if one occurred.
 *
 * # Examples
 *
 * ```
 * use todo::model::Todo;
 * let todos: Vec<Todo> = todo::io::load_todos_from_file("/tmp/test_todos.json").unwrap();
 * ```
 */
pub fn load_todos_from_file<P: AsRef<path::Path>>(file: P) -> Result<Vec<Todo>> {
    let path_ref = file.as_ref();

    let mut file = OpenOptions::new()
        .read(true)       // Enable reading
        .write(true)      // Enable writing
        .create(true)     // Create if doesn't exist
        .open(path_ref)
        .with_context(|| format!("failed to open file: {}", path_ref.display()))?;
    read_data(&mut file).with_context(|| format!("failed to read todos from file: {}", path_ref.display()))
}

/**
 * Reads todos from a file.
 *
 * # Returns
 *
 * A `Result` containing the loaded todos, or an error if one occurred.
 */
fn read_data(file: &mut std::fs::File) -> Result<Vec<Todo>, serde_json::Error> {
    let reader = BufReader::new(file);
    match serde_json::from_reader(reader) {
        Ok(data) => Ok(data),
        Err(e) if e.is_eof() => Ok(Vec::default()),
        Err(e) => Err(e.into()),
    }
}

/**
 * Saves todos to the default todos file.
 *
 * # Returns
 *
 * A `Result` indicating success or failure.
 *
 * # Examples
 *
 * ```
 * let todos: Vec<todo::model::Todo> = vec![];
 * todo::io::save_todos(&todos).unwrap();
 * ```
 */
pub fn save_todos(todos: &[Todo]) -> Result<()> {
    let home = env::var("HOME")  // Unix
         .or_else(|_| env::var("USERPROFILE"))  // Windows
         .expect("HOME not set");
    println!("Saving to {home}/{TODOS_FILE}");
    save_todos_to_file(todos, format!("{home}/{TODOS_FILE}"))
}
/**
 * Saves todos to a specified file.
 *
 * # Returns
 *
 * A `Result` indicating success or failure.
 *
 * # Examples
 *
 * ```
 * let todos: Vec<todo::model::Todo> = vec![];
 * todo::io::save_todos_to_file(&todos, "/tmp/test_todos.json").unwrap();
 * ```
 */
pub fn save_todos_to_file<P: AsRef<path::Path>>(todos: &[Todo], file: P) -> Result<()> {
    let path_ref = file.as_ref();
    let mut file = OpenOptions::new()
        .write(true)      // Enable writing
        .create(true)     // Create if doesn't exist
        .truncate(true)
        .open(path_ref)
        .with_context(|| format!("failed to open file: {}", path_ref.display()))?;
    write_data(todos, &mut file).with_context(|| format!("failed to save todos to file: {}", path_ref.display()))
}

/**
 * Writes todos to a file.
 *
 * # Returns
 *
 * A `Result` indicating success or failure.
 */
fn write_data(todos: &[Todo], file: &mut std::fs::File) -> Result<(), serde_json::Error> {
    serde_json::to_writer(file, todos)?;
    Ok(())
}
