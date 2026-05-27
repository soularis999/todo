use clap::Parser;
use anyhow::Result;
use todo::{io, model::{Commands, ListArgs, Todo}, process};

fn main() -> Result<()> {
    let args = todo::model::Args::parse();
    let mut data: Vec<Todo> = io::load_todos()?;

    match args.command {
        Commands::Add(args) => process::add(args, &mut data).and_then(|todos| io::save_todos(todos)),
        Commands::Edit(args) => process::edit(args, &mut data).and_then(|todos| io::save_todos(todos)),
        Commands::Complete(args) => process::set_complete(args.pos, true, &mut data).and_then(|todos| io::save_todos(todos)),
        Commands::UnComplete(args) => process::set_complete(args.pos, false, &mut data).and_then(|todos| io::save_todos(todos)),
        // non mutating
        Commands::List(args) => print_todos(args, data),
    }?;

    Ok(())
}

fn print_todos(args: ListArgs, todos: Vec<Todo>) -> Result<()> {
    let verbose = args.verbose.unwrap_or(false);
    let trimmed = process::filter_for_list(args, todos)?;

    for (index, todo) in trimmed.iter().enumerate() {
        if verbose {
            println!("{}: {}", index, todo);
        } else {
            println!("{}: {}", index, todo.title);
        }

    }
    Ok(())
}
