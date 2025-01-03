# linecapture

A command-line tool to add lines to a Markdown file with a single command.

## Features

- Add tasks to Markdown files with a single command
- Adheres to the amazing [obsidian-tasks](https://github.com/obsidian-tasks-group/obsidian-tasks) format for seamless integration

## Installation

```bash
cargo install linecapture
```

## Configuration

Create a file named `linecapture.toml` in your `~/.config/linecapture` directory with the following content:

```toml
# Path to the file where captured lines will be appended
file_path = "~/vault/tasks.md"

# Header to be added before the captured lines
header = "## Tasks"
```

## Usage

```bash
linecapture task <description> [FLAGS]

FLAGS:
    --id <id>    ID of the task
    --created <created>    Created date of the task in the format YYYY-MM-DD
    --scheduled <scheduled>    Scheduled date of the task in the format YYYY-MM-DD
    --start <start>    Start date of the task in the format YYYY-MM-DD
    --due <due>    Due date of the task in the format YYYY-MM-DD
```

## Example

```bash
linecapture task "Add awesome feature to #project/linecapture" --due 2025-03-27
```

## License

[MIT](https://choosealicense.com/licenses/mit/)
