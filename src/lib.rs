use chrono::{Datelike, Duration, Local, NaiveDate};

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

fn print_welcome() -> Result<(), Box<dyn Error>> {
    println!("╷╭ ╭─╴╷  ╭─╮╭╮╷");
    println!("├┴╮├╴ │  ├─┤│╰┤");
    println!("╵ ╵╰─╴╰─╴╵ ╵╵ ╵");

    Ok(())
}

fn today_file_path() -> Result<String, Box<dyn Error>> {
    let dir = notes_dir()?;
    let date = Local::now().format("%Y-%m-%d").to_string();
    Ok(format!("{dir}/{date}.txt"))
}

fn print_month_calendar() -> Result<(), Box<dyn Error>> {
    let now = Local::now();
    let year = now.year();
    let month = now.month();

    let first_day = NaiveDate::from_ymd_opt(year, month, 1)
        .ok_or("invalid calendar date")?;
    let offset = first_day.weekday().num_days_from_sunday();

    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let next_month_first = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .ok_or("invalid next month date")?;
    let days_in_month = (next_month_first - Duration::days(1)).day();

    println!("{year}-{month:02}");
    println!("Su Mo Tu We Th Fr Sa");

    for _ in 0..offset {
        print!("   ");
    }

    for day in 1..=days_in_month {
        print!("{day:>2} ");
        if (offset + day) % 7 == 0 {
            println!();
        }
    }

    if (offset + days_in_month) % 7 != 0 {
        println!();
    }

    Ok(())
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
    println!("  stats              Shows first entry and number of total entries");
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
    println!("  cargo run -- stats");
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
        "stats" => {
            let dir = notes_dir()?;
            let mut entries = Vec::new();

            if fs::metadata(&dir).is_ok() {
                for entry in fs::read_dir(&dir)? {
                    let entry = entry?;
                    if !entry.path().is_file() {
                        continue;
                    }
                    if let Some(name) = entry.file_name().to_str() {
                        entries.push(name.to_string());
                    }
                }
            }

            entries.sort();
            let number_of_entries = entries.len();
            let first_entry = match entries.first() {
                Some(name) => name.as_str(),
                None => "none",
            };
            println!("first entry: {first_entry}");
            println!("number of entries: {number_of_entries}\n");
            println!("calendar: ");
            print_month_calendar()?; 
        }
        "welcome" => {
            print_welcome()?;
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
           return Ok(Config { command: String::from("welcome"), file_path: String::new(), text: None });
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
            "stats" => {
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
            _ => Err("unknown command: run 'help' to see usage"),
        }
    }
}
