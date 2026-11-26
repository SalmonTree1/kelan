use chrono::Local;
use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};

fn notes_dir() -> Result<String, Box<dyn Error>> {
    let home = env::var("HOME")?;
    Ok(format!("{home}/notes"))
}

fn today_file_path() -> Result<String, Box<dyn Error>> {
    let dir = notes_dir()?;
    let date = Local::now().format("%Y-%m-%d").to_string();
    Ok(format!("{dir}/{date}.txt"))
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
    println!("  append [text]      Append to today's note (reads from stdin if text omitted)");
    println!("  help               Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  cargo run -- read");
    println!("  cargo run -- read 2026-11-25.txt");
    println!("  cargo run -- append \"Had a breakthrough today\"");
    println!("  cargo run -- append <<< \"First paragraph\n\nSecond paragraph\"");
    println!("  echo -e \"Line1\\nLine2\" | cargo run -- append");
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
                None => {
                    let mut buffer = String::new();
                    io::stdin().read_to_string(&mut buffer)?;
                    buffer
                }
            };

            writeln!(file, "{}", text)?;
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
                    Some(args[2].clone())
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
