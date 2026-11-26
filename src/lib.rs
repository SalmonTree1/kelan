use chrono::Local;
use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, Write};

fn notes_dir() -> Result<String, Box<dyn Error>> {
    if let Ok(dir) = env::var("NOTES_DIR") {
        return Ok(dir);
    }
    let home = env::var("HOME")?;
    Ok(format!("{home}/notes"))
}

fn today_file_path() -> Result<String, Box<dyn Error>> {
    let dir = notes_dir()?;
    let date = Local::now().format("%Y-%m-%d").to_string();
    Ok(format!("{dir}/{date}.txt"))
}

fn read_multiline() -> Result<String, Box<dyn Error>> {
    println!("Enter lines (type 'done' on a new line to finish):");
    let stdin = io::stdin();
    let mut lines = Vec::new();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim() == "done" {
            break;
        }
        lines.push(line);
    }

    Ok(lines.join("\n"))
}

pub fn print_help() {
    let banner = r#"
╷╭ ╭─╴╷  ╭─╮╭╮╷
├┴╮├╴ │  ├─┤│╰┤
╵ ╵╰─╴╰─╴╵ ╵╵ ╵
    "#;
    println!("{banner}");
    println!("USAGE:");
    println!("  <program> <command> [arguments]");
    println!();
    println!("COMMANDS:");
    println!("  read [file_name]   Print a note from ~/notes (defaults to today's note)");
    println!("  append [text...]   Append to today's note (interactive multi-line mode if text omitted; blank line for paragraph spacing, 'done' to finish)");
    println!("  help               Show this help message");
    println!();
    println!("ENVIRONMENT:");
    println!("  NOTES_DIR          Override default notes directory (~/notes)");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run -- read");
    println!("  cargo run -- read 2026-11-25.txt");
    println!("  cargo run -- append \"Had a breakthrough today\"");
    println!("  cargo run -- append First line second line");
    println!("  cargo run -- append");
    println!("  cargo run -- help");
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.command.as_str() {
        "read" => {
            let path = if config.file_path.is_empty() {
                today_file_path()?
            } else {
                let dir = notes_dir()?;
                format!("{dir}/{}", config.file_path)
            };
            let contents = fs::read_to_string(path)?;
            println!("{contents}");
        }
        "append" => {
            let dir = notes_dir()?;
            fs::create_dir_all(&dir)?;
            let path = today_file_path()?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?;

            let text = match config.text {
                Some(t) => t,
                None => read_multiline()?,
            };

            file.write_all(text.as_bytes())?;
            if !text.ends_with('\n') {
                file.write_all(b"\n")?;
            }
            println!("Appended text successfully.");
        }
        "help" => {
            print_help();
        }
        _ => {
            return Err("unknown command: run 'help' to see usage".into());
        }
    }

    Ok(())
}

pub struct Config {
    pub command: String,
    pub file_path: String,
    pub text: Option<String>,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments (run 'help' to see usage)");
        }

        let command = args[1].clone();

        if command == "help" || command == "-h" || command == "--help" {
            return Ok(Config {
                command: String::from("help"),
                file_path: String::new(),
                text: None,
            });
        }

        match command.as_str() {
            "read" => {
                let file_path = if args.len() >= 3 {
                    args[2].clone()
                } else {
                    String::new()
                };

                Ok(Config {
                    command,
                    file_path,
                    text: None,
                })
            }
            "append" => {
                let text = if args.len() >= 3 {
                    Some(args[2..].join(" "))
                } else {
                    None
                };

                Ok(Config {
                    command,
                    file_path: String::new(),
                    text,
                })
            }
            _ => Err("unknown command: run 'help' to see usage"),
        }
    }
}
