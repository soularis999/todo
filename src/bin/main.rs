use anyhow::{Context, Result};
use clap::Parser;
use todo::{
    cli::model::{Commands, ListArgs},
    io,
    model::{EditTodo, Todo, Todos},
};

fn main() -> Result<()> {
    let args = todo::cli::model::Args::parse();
    let mut data: Todos = io::load_todos()?;

    match args.command {
        Commands::Add(args) => {
            let todo: Todo = args.try_into()?;
            data.add(todo)?;
            io::save_todos(&data)?;
        }
        Commands::Edit(args) => {
            let mut todo: EditTodo<'_> = data
                .get_mut(&args.id)
                .with_context(|| format!("Todo with id {} not found", args.id))?;
            todo.priority = args.priority;
            todo.tags = args.tags;
            todo.finish()?;
            io::save_todos(&data)?;
        }
        Commands::Complete(args) => {
            let mut todo: EditTodo<'_> = data
                .get_mut(&args.id)
                .with_context(|| format!("Todo with id {} not found", args.id))?;
            todo.completed = Some(true);
            todo.finish()?;
            io::save_todos(&data)?;
        }
        Commands::UnComplete(args) => {
            let mut todo: EditTodo<'_> = data
                .get_mut(&args.id)
                .with_context(|| format!("Todo with id {} not found", args.id))?;
            todo.completed = Some(false);
            todo.finish()?;
            io::save_todos(&data)?;
        }
        // non mutating
        Commands::List(args) => {
            print_todos(&args, &data)?;
        },
        // TUI
        #[cfg(feature = "tui")]
        Commands::Tui => {
            //todo::tui::run(data),
        }
    };

    Ok(())
}

fn print_todos(args: &ListArgs, todos: &Todos) -> Result<()> {
    let verbose = args.verbose.unwrap_or(false);
    let filter_tags: &[String] = args.tags.as_deref().unwrap_or_default();
    
    todos.visit(|todo| {
        if !todo.has_any(filter_tags.iter().map(String::as_str)) {
            return Ok(true);
        }
        
        if verbose {
            println!(
                "{} {}: {}",
                if todo.completed { "✓" } else { "☐" },
                todo.id,
                todo
            );
        } else {
            println!(
                "{} {}: {} {}",
                if todo.completed { "✓" } else { "☐" },
                todo.id,
                todo.title,
                todo.priority
            );
        }

        
        Ok(true)
    })?;

    Ok(())
}
