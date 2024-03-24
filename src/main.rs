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
        if let Some(args) = matches
            .get_many::<String>("arg")
            .map(|a| a.collect::<Vec<_>>())
        {
            // TODO check for valid input (simple strings)
            let piped_arg = read_pipe();

            let cmd = build_cmd(args, piped_arg, replace_flag);

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

fn vec_to_str(str_vec: Vec<&String>) -> String {
    str_vec.iter().map(|s| s.to_string()).collect::<String>()
}

fn build_cmd(cmd_vec: Vec<&String>, arg: String, replace_flag: bool) -> String {
    let cmd = vec_to_str(cmd_vec);
    // TODO remove later
    dbg!(&cmd);

    // split given command if it has flags
    // respect placeholders
    let mut combined_cmd = String::new();
    if replace_flag {
        // TODO in powershell: is '{}' a problem?
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
            Arg::new("arg")
                .help("The command that takes an argument from stdin")
                .action(ArgAction::Set)
                .value_name("COMMAND"),
        )
        .arg(
            Arg::new("replace")
                .short('r')
                .long("replace")
                .help("Replace the given placeholder with the string from stdin")
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
