use clap::{Arg, ArgAction, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::error;
use owo_colors::colored::*;

use std::{
    fs,
    io::{self, BufRead},
    path::{Path, PathBuf},
    process,
};

fn main() {
    // handle Ctrl+C
    ctrlc::set_handler(move || {
        println!(
            "{} {} {} {}",
            "Received Ctrl-C!".bold().red(),
            "🤬",
            "Exit program!".bold().red(),
            "☠",
        );
        process::exit(0)
    })
    .expect("Error setting Ctrl-C handler");

    // get config dir
    let config_dir = check_create_config_dir().unwrap_or_else(|err| {
        error!("Unable to find or create a config directory: {err}");
        process::exit(1);
    });

    // initialize the logger
    let _logger = Logger::try_with_str("info") // log warn and error
        .unwrap()
        .format_for_files(detailed_format) // use timestamp for every log
        .log_to_file(
            FileSpec::default()
                .directory(&config_dir)
                .suppress_timestamp(),
        ) // change directory for logs, no timestamps in the filename
        .append() // use only one logfile
        .duplicate_to_stderr(Duplicate::Info) // print infos, warnings and errors also to the console
        .start()
        .unwrap();

    // handle arguments
    let matches = xargs().get_matches();
    let replace_flag = matches.get_flag("replace");

    if let Some(_) = matches.subcommand_matches("log") {
        if let Ok(logs) = show_log_file(&config_dir) {
            println!("{}", "Available logs:".bold().yellow());
            println!("{}", logs);
        } else {
            error!("Unable to read logs");
            process::exit(1);
        }
    } else {
        // TODO read everything given here as ONE argument
        // FIXME only works with '"'
        // TODO read as literal string
        if let Some(args) = matches
            .get_many::<String>("args")
            .map(|a| a.collect::<Vec<_>>())
        {
            let piped_arg = read_pipe();
            // TODO remove later
            dbg!(&piped_arg);

            // FIXME handle multiple lines of input -> replace workaround below
            // check for valid input (simple strings, no invalid special chars like '\n')
            if piped_arg.contains("\n") {
                error!("Multiple lines in stdin detected:\n{}", piped_arg);
                process::exit(0);
            }

            let cmd = build_cmd(args, piped_arg, replace_flag);
            // TODO remove later
            dbg!(&cmd);

            run_cmd(&cmd);
        } else {
            let _ = xargs().print_help();
            process::exit(0);
        }
    }
}

fn read_pipe() -> String {
    let mut input = io::stdin()
        .lock()
        .lines()
        .fold("".to_string(), |acc, line| acc + &line.unwrap() + "\n");

    let _ = input.pop();

    input.trim().to_string()
}

// TODO handle multiple lines in stdin
// fn split_pipe_by_lines(pipe: String) -> Vec<String> {
//     let mut pipe_collector = Vec::new();
//     for line in pipe.lines() {
//         pipe_collector.push(line.to_string());
//     }

//     pipe_collector
// }

fn chain_args_with_space(args: Vec<&String>) -> String {
    // chain given arguments together with a spaces inbetween
    let mut strg = String::new();
    for s in &args {
        strg.push_str(s);

        if args.iter().last() == Some(&s) {
            break;
        } else {
            strg.push_str(" ");
        }
    }

    strg
}

fn build_cmd(cmd_vec: Vec<&String>, arg: String, replace_flag: bool) -> String {
    let cmd = chain_args_with_space(cmd_vec);

    // split given command if it has flags
    // respect placeholders
    let mut combined_cmd = String::new();
    if replace_flag {
        // INFO -> surround '{}' with quotation marks
        combined_cmd.push_str(&cmd.replace("{}", &arg));
    } else {
        combined_cmd.push_str(&cmd);
        combined_cmd.push_str(" ");
        combined_cmd.push_str(&arg);
    }

    combined_cmd
}

fn run_cmd(cmd: &str) {
    if cfg!(target_os = "windows") {
        std::process::Command::new("powershell")
            // TODO use arg instead of args -> split flags separatly by '-' ?
            .args(["-c", cmd])
            .status()
            .unwrap();
    } else {
        unimplemented!();
    }
}

// build cli
fn xargs() -> Command {
    Command::new("xa")
        .bin_name("xa")
        .before_help(format!(
            "{}\n{}",
            "XA".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .about("XArgs")
        .before_long_help(format!(
            "{}\n{}",
            "XA".bold().truecolor(250, 0, 104),
            "Leann Phydon <leann.phydon@gmail.com>".italic().dimmed()
        ))
        .long_about(format!("{}", "XArgs",))
        // TODO update version
        .version("1.0.0")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg(
            Arg::new("args")
                .help("The command to execute with an argument from stdin")
                .action(ArgAction::Append)
                .value_name("COMMAND"),
        )
        .arg(
            Arg::new("replace")
                .short('r')
                .long("replace")
                .help(
                    "Replace the given placeholder [curly braces: '{}'] with the string from stdin",
                )
                .long_help(format!(
                    "{}\n{}",
                    "Replace the given placeholder [curly braces: '{}'] with the string from stdin",
                    "You have to surround the curly braces with single quotes ['{}']"
                ))
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            Command::new("log")
                .short_flag('L')
                .long_flag("log")
                .about("Show content of the log file"),
        )
}

fn check_create_config_dir() -> io::Result<PathBuf> {
    let mut new_dir = PathBuf::new();
    match dirs::config_dir() {
        Some(config_dir) => {
            new_dir.push(config_dir);
            new_dir.push("xa");
            if !new_dir.as_path().exists() {
                fs::create_dir(&new_dir)?;
            }
        }
        None => {
            error!("Unable to find config directory");
        }
    }

    Ok(new_dir)
}

fn show_log_file(config_dir: &PathBuf) -> io::Result<String> {
    let log_path = Path::new(&config_dir).join("xa.log");
    return match log_path.try_exists()? {
        true => Ok(format!(
            "{} {}\n{}",
            "Log location:".italic().dimmed(),
            &log_path.display(),
            fs::read_to_string(&log_path)?
        )),
        false => Ok(format!(
            "{} {}",
            "No log file found:"
                .truecolor(250, 0, 104)
                .bold()
                .to_string(),
            log_path.display()
        )),
    };
}
