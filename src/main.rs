use chrono::{Local, NaiveDate, ParseError};
use clap::{Arg, Command};
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use xdg::BaseDirectories;

/// Function to validate date format (YYYY-MM-DD)
///
/// This function takes a string slice `date_str` as input and attempts to parse it
/// as a date in the format "YYYY-MM-DD" using the `NaiveDate::parse_from_str` method
/// from the `chrono` crate.
///
/// # Arguments
///
/// * `date_str` - A string slice containing the date to be parsed.
///
/// # Returns
///
/// * `Ok(NaiveDate)` - If the input string can be successfully parsed as a date.
/// * `Err(ParseError)` - If the input string cannot be parsed as a valid date.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
///
/// let valid_date = "2023-04-15";
/// let invalid_date = "2023/04/15";
///
/// assert!(verify_date(valid_date).is_ok());
/// assert!(verify_date(invalid_date).is_err());
/// ```
fn verify_date(date_str: &str) -> Result<NaiveDate, ParseError> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
}

// Read the task file into a vector of strings
/// This function reads the contents of a file at the given path and returns a
/// `Vec` of `String`s, where each `String` represents a line in the file.
///
/// # Arguments
///
/// * `path` - The path to the file to be read.
///
/// # Returns
///
/// A `Result` containing a `Vec` of `String`s if the file was read successfully,
/// or an `io::Error` if there was an error reading the file.
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// let path = Path::new("tasks.md");
/// let lines = read_file(path).unwrap();
/// println!("Lines in the file:");
/// for line in lines {
///     println!("{}", line);
/// }
/// ```
fn read_file(path: &Path) -> io::Result<Vec<String>> {
    let file = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    Ok(lines)
}

/// Represents the configuration for the application.
///
/// The `Config` struct holds the file path and header information
/// read from the configuration file.
#[derive(Deserialize, Debug)]
struct Config {
    /// The path to the file to be processed.
    ///
    /// This can be an absolute path or a path relative to the current
    /// working directory. If the path starts with a tilde (`~`), it
    /// will be expanded to the user's home directory.
    file_path: PathBuf,

    /// The header text to be included at the top of the output file.
    header: String,
}

impl Config {
    /// Creates a new `Config` instance from the provided configuration data.
    ///
    /// This function takes the result of parsing the configuration file
    /// and performs additional validation and normalization on the file path.
    /// If the file path starts with a tilde (`~`), it is expanded to the
    /// user's home directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration file is invalid or if the
    /// home directory cannot be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// let config_data = r#"
    ///     file_path = "~/vault/tasks.md"
    ///     header = \"# TODO"
    /// "#;
    ///
    /// let config = toml::from_str::<Config>(config_data).unwrap();
    /// let config = Config::new(config).unwrap();
    /// assert_eq!(config.file_path, PathBuf::from("/home/user/.config/linecapture/liecapture.toml"));
    /// assert_eq!(config.header, "# TODO");
    /// ```
    fn new(config: Result<Config, toml::de::Error>) -> Result<Config, String> {
        let config = config.map_err(|e| format!("Invalid configuration file: {}", e))?;

        // Expand the tilde in the file path
        let file_path = if config.file_path.starts_with("~") {
            let home_dir = dirs::home_dir().ok_or("Could not determine home directory")?;
            let path_without_tilde = config.file_path.strip_prefix("~").unwrap();
            home_dir.join(path_without_tilde)
        } else {
            config.file_path
        };

        Ok(Config {
            file_path,
            header: config.header,
        })
    }
}

// Respect XDG Base Directory Specification
// This function reads the configuration file from the user's config directory
// according to the XDG Base Directory Specification.
// If the config file does not exist, it creates a default one.
fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    let xdg_dirs = BaseDirectories::with_prefix("linecapture")?;
    // Find the config file path based on XDG directories
    let config_path = xdg_dirs
        .find_config_file("linecapture.toml")
        .unwrap_or_else(|| {
            let mut default_path = xdg_dirs.get_config_home();
            default_path.push("linecapture/lightcapture.toml");
            default_path
        });

    // Read the config file or create a default one if it doesn't exist
    let config_str = std::fs::read_to_string(&config_path).unwrap_or_else(|_| {
        let default_config = vec![
            "header = \"## TODO\"".to_string(),
            "emoji = false".to_string(),
        ];

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }

        write_file(&config_path, &default_config).unwrap();
        default_config.join("\n")
    });

    // Parse the config file
    let config: Config = toml::from_str(&config_str)?;
    Ok(Config::new(Ok(config))?)
}

// Write the task file from a vector of strings
/// `write_file` takes a `Path` and a vector of `String`s and writes the contents
/// of the vector to the file at the specified path.
///
/// # Arguments
///
/// * `path` - The path to the file to write to.
/// * `content` - A vector of strings to write to the file.
///
/// # Returns
///
/// Returns an `io::Result` containing `Ok(())` if the file was written successfully,
/// or an `io::Error` if there was an error writing the file.
///
/// # Examples
///
/// ```
/// use std::path::Path;
///
/// let path = Path::new("tasks.md");
/// let content = vec!["- [ ] do the laundry".to_string(), "- [ ] walk the dog".to_string()];
///
/// if let Err(e) = write_file(&path, &content) {
///     eprintln!("Error writing file: {}", e);
/// }
/// ```
fn write_file(path: &Path, content: &[String]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    for line in content {
        writeln!(file, "{}", line)?;
    }
    Ok(())
}

/// This function takes a slice of `String`s and a pattern string `query`.
/// It returns the index of the first line in the slice that contains the `query` pattern.
/// If no line contains the pattern, it returns `None`.
///
/// # Examples
///
/// ```
/// let lines = vec!["hello".to_string(), "world".to_string(), "foo bar".to_string()];
/// let index = find_first_occurrence(&lines, "bar");
/// assert_eq!(index, Some(2));
///
/// let index = find_first_occurrence(&lines, "baz");
/// assert_eq!(index, None);
/// ```
fn find_first_occurrence(lines: &[String], query: &str) -> Option<usize> {
    lines.iter().position(|line| line.contains(query))
}

/// This function takes a mutable vector of strings `lines`, a string `task_line` representing the new task to be added,
/// and an optional string `header` representing the header under which the task should be added.
///
/// If the `header` is not provided, it defaults to "## TODO".
///
/// The function first finds the position of the header in the `lines` vector. If the header is not found, it is added to the end of the vector.
///
/// Next, the function finds the position of the first task after the header. If no tasks are found after the header, the new task is added to the end of the vector.
///
/// Finally, the function inserts the new `task_line` at the position of the first task after the header, or at the end of the vector if no tasks were found.
///
/// The function returns the updated `lines` vector with the new task added.
///
/// # Examples
///
/// ```
/// let mut lines = vec!["## TODO".to_string(), "- [ ] Existing task".to_string()];
/// let new_task = "- [ ] New task".to_string();
/// let updated_lines = add_task(lines, new_task, None);
/// assert_eq!(updated_lines, vec!["## TODO".to_string(), "- [ ] Existing task".to_string(), "- [ ] New task".to_string()]);
/// ```
fn add_task(mut lines: Vec<String>, task_line: String, header: Option<String>) -> Vec<String> {
    // Default header
    let header = header.unwrap_or_else(|| "## TODO".to_string());
    // Find or add the header
    let header_pos = find_first_occurrence(&lines, &header).unwrap_or_else(|| {
        lines.push(header.to_string());
        lines.len() - 1
    });

    // Find the first task position after the header
    let first_task = find_first_occurrence(&lines[header_pos..], "- [ ]")
        .map(|idx| header_pos + idx)
        .unwrap_or_else(|| lines.len());

    // Insert the new task
    lines.insert(first_task, task_line);
    lines
}

/// Defines the command-line interface for the application.
///
/// This function sets up the command-line arguments and options for the application.
/// It uses the `clap` crate to define the command-line interface.
///
/// # Returns
///
/// A `Command` object representing the command-line interface.
fn app() -> Command {
    Command::new("lineadd")
        .version("0.1")
        .author("Oleksii Luchnikov <oleksiiluchnikov@gmail.com>")
        .subcommand(
            Command::new("task")
                .about("Adds a new task to a Markdown task file")
                .arg(
                    Arg::new("description")
                        .help("Task description")
                        .required(true),
                )
                .arg(Arg::new("id").long("id").help("Task ID"))
                .arg(
                    Arg::new("created")
                        .long("created")
                        .help("Creation date (YYYY-MM-DD)"),
                )
                .arg(Arg::new("due").long("due").help("Due date (YYYY-MM-DD)"))
                .arg(
                    Arg::new("scheduled")
                        .long("scheduled")
                        .help("Scheduled date (YYYY-MM-DD)"),
                )
                .arg(Arg::new("repeat").long("repeat").help("Repeating pattern"))
                .arg(
                    Arg::new("start")
                        .long("start")
                        .help("Start date (YYYY-MM-DD)"),
                )
                .arg(Arg::new("priority").long("priority").help("Priority level"))
                .arg(
                    Arg::new("emoji")
                        .long("emoji")
                        .help("Use emoji in the task list")
                        .default_value("false"),
                ),
        )
}

/// Represents the priority level of a task or item.
///
/// The priority levels are ordered from lowest to highest, with `Lowest` being the lowest
/// priority and `Highest` being the highest priority. The `None` variant represents
/// the absence of a priority level.
#[derive(Debug)]
enum Priority {
    /// The lowest priority level.
    Lowest,
    /// A low priority level.
    Low,
    /// A medium priority level.
    Medium,
    /// A high priority level.
    High,
    /// The highest priority level.
    Highest,
    /// No priority level assigned.
    None,
}

// TODO: Add option to use emojis for different task states
// due > 📅
// start > 🛫
// scheduled > ⏳
// done > ✅
// high > ⏫
// medium > 🔼
// low > 🔽
// repeat > 🔁
// recurring > 🔁

#[derive(Debug)]
/// Task represents a task or todo item.
struct Task {
    /// A description of the task.
    description: String,
    /// A unique identifier for the task.
    id: String,
    /// The date the task was created.
    created: NaiveDate,
    /// The due date for the task, if any.
    due: Option<NaiveDate>,
    /// The start date for the task, if any.
    start: Option<NaiveDate>,
    /// The scheduled date for the task, if any.
    scheduled: Option<NaiveDate>,
    /// A string representing the repeat pattern for the task, if any.
    repeat: Option<String>,
    /// The file path where the task is located.
    file_path: PathBuf,
    /// The header text associated with the task.
    header: String,
    /// The priority level of the task, if any.
    priority: Option<Priority>,
}

impl Task {
    /// Creates a new `Task` instance.
    ///
    /// # Arguments
    ///
    /// * `description` - The description of the task.
    /// * `id` - An optional ID for the task. If not provided, a unique ID will be generated based on the current timestamp.
    /// * `created` - An optional creation date for the task. If not provided, the current date will be used.
    /// * `due` - An optional due date for the task.
    /// * `start` - An optional start date for the task.
    /// * `scheduled` - An optional scheduled date for the task.
    /// * `repeat` - An optional repeat pattern for the task (e.g., "daily", "weekly", "monthly").
    /// * `file_path` - The file path associated with the task.
    /// * `header` - The header text associated with the task.
    /// * `priority` - An optional priority level for the task.
    fn new(
        description: String,
        id: Option<String>,
        created: Option<NaiveDate>,
        due: Option<NaiveDate>,
        start: Option<NaiveDate>,
        scheduled: Option<NaiveDate>,
        repeat: Option<String>,
        file_path: PathBuf,
        header: String,
        priority: Option<Priority>,
    ) -> Self {
        let now = Local::now();
        let id = id
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}", now.format("%Y%m%d%H%M%S")));
        let created = created.unwrap_or_else(|| now.naive_local().into());
        let header = header.to_string();

        Task {
            description,
            id,
            created,
            due,
            scheduled,
            repeat,
            start,
            file_path,
            header,
            priority,
        }
    }

    /// This function generates a string representation of a task with its details.
    ///
    /// The string representation includes the task description, ID, creation date,
    /// due date (if present), start date (if present), scheduled date (if present),
    /// and repeat pattern (if present).
    ///
    /// # Examples
    ///
    /// ```
    /// let task = Task {
    ///     description: "Buy groceries".to_string(),
    ///     id: "123".to_string(),
    ///     created: chrono::Utc::now().date_naive(),
    ///     due: Some(chrono::Utc::now().date_naive() + chrono::Duration::days(3)),
    ///     start: None,
    ///     scheduled: None,
    ///     repeat: Some("weekly".to_string()),
    /// };
    ///
    /// let line = task.to_line();
    /// println!("{}", line);
    /// ```
    ///
    /// This will print a string representation of the task, similar to:
    ///
    /// ```text
    /// - [ ] Buy groceries  [id:: 123]  [created:: 2023-05-15]  [due:: 2023-05-18]  [repeat:: weekly]
    /// ```
    fn to_line(&self) -> String {
        fn wrap(k: &str, v: &str) -> String {
            format!("[{}:: {}]", k, v)
        }

        let indent = "  ";
        let mut line = format!("- [ ] {}", &self.description);

        line.push_str(indent);
        line.push_str(&wrap("id", &self.id));

        line.push_str(indent);
        line.push_str(&wrap(
            "created",
            &&self.created.format("%Y-%m-%d").to_string(),
        ));

        if let Some(due) = &self.due {
            line.push_str(indent);
            line.push_str(&wrap("due", &due.format("%Y-%m-%d").to_string()));
        }

        if let Some(start) = &self.start {
            line.push_str(indent);
            line.push_str(&wrap("start", &start.format("%Y-%m-%d").to_string()));
        }

        if let Some(scheduled) = &self.scheduled {
            line.push_str(indent);
            line.push_str(&wrap(
                "scheduled",
                &scheduled.format("%Y-%m-%d").to_string(),
            ));
        }

        if let Some(repeat) = &&self.repeat {
            line.push_str(indent);
            line.push_str(&wrap("repeat", repeat));
        }

        line
    }
}

fn main() -> io::Result<()> {
    let matches = app().get_matches();
    let matches_task = matches.subcommand_matches("task").unwrap();

    let task_description = matches_task.get_one::<String>("description").unwrap();
    let now = Local::now();
    let uuid = now.format("%Y%m%d%H%M%S").to_string();
    let id = matches_task
        .get_one::<String>("id")
        .unwrap_or(&uuid)
        .to_owned();
    let created_date = now.format("%Y-%m-%d").to_string();
    let created = matches_task
        .get_one::<String>("created")
        .unwrap_or(&created_date)
        .to_owned();

    if let Some(due) = matches_task.get_one::<String>("due") {
        if let Err(e) = verify_date(due) {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
    if let Some(scheduled) = matches_task.get_one::<String>("scheduled") {
        if let Err(e) = verify_date(scheduled) {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
    if let Some(start) = matches_task.get_one::<String>("start") {
        if let Err(e) = verify_date(start) {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    let config = read_config().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let task = Task::new(
        task_description.to_string(),
        Some(id),
        Some(NaiveDate::parse_from_str(&created, "%Y-%m-%d").unwrap()),
        matches_task
            .get_one::<String>("due")
            .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()),
        matches_task
            .get_one::<String>("start")
            .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()),
        matches_task
            .get_one::<String>("scheduled")
            .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()),
        matches_task
            .get_one::<String>("repeat")
            .map(|s| s.to_string()),
        config.file_path.clone(),
        config.header.clone(),
        matches_task
            .get_one::<String>("priority")
            .map(|s| match s.to_lowercase().as_str() {
                "lowest" => Priority::Lowest,
                "low" => Priority::Low,
                "medium" => Priority::Medium,
                "high" => Priority::High,
                "highest" => Priority::Highest,
                _ => Priority::None,
            }),
    );

    let lines = read_file(&config.file_path)?;
    let updated_lines = add_task(lines, task.to_line(), Some(config.header));
    write_file(&config.file_path, &updated_lines)?;
    Ok(())
}

// TODO: Add tests
