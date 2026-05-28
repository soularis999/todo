# todo

A simple command-line to-do manager written in Rust. Tasks are stored as JSON in `~/.todos.json`.

## Installation

```bash
cargo install --path .
```

Or run directly:

```bash
cargo run -- <command> [args...]
```

## Usage

### Add a task

```bash
todo add "Buy groceries" -p high -t food -t urgent
todo ad "Write report"             # alias
```

Priority flags: `-p low`, `-p medium` (default), `-p high`.

### List tasks

```bash
todo list       # shows incomplete tasks
todo ls         # alias
todo ls -v      # verbose — shows all fields
todo ls -t food # filter by tag
```

### Edit a task

```bash
todo edit 0 -p low     # change priority (0-based index from list output)
todo md 0 -t work      # replace tags
todo md 0 -p high -t urgent -t work  # change both
```

### Mark complete / incomplete

```bash
todo complete 0   # alias: cp
todo uncomplete 0 # alias: ucp
```

## Data

All tasks are saved to `~/.todos.json` with the following fields:

| Field        | Type              |
|-------------|-------------------|
| `id`        | UUID              |
| `title`     | String            |
| `completed` | Boolean           |
| `priority`  | low / medium / high |
| `tags`      | Array of strings (optional) |
| `created_at`| UTC timestamp     |
| `updated_at`| UTC timestamp     |

## Build & Test

```bash
cargo build
cargo test
```
