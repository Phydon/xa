use clap::{Arg, ArgAction, Command};
use flexi_logger::{detailed_format, Duplicate, FileSpec, Logger};
use log::error;
use owo_colors::colored::*;
use rayon::prelude::*;

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
    let parallel_flag = matches.get_flag("parallel");

    if let Some(_) = matches.subcommand_matches("log") {
        if let Ok(logs) = show_log_file(&config_dir) {
            println!("{}", "Available logs:".bold().yellow());
            println!("{}", logs);
        } else {
            error!("Unable to read logs");
            process::exit(1);
        }
    } else {
        if let Some(args) = matches
            .get_many::<String>("args")
            .map(|a| a.collect::<Vec<_>>())
        {
            let pipe = read_pipe();
            // TODO remove later
            // dbg!(&pipe);

            let piped_args = split_pipe_by_lines(pipe);
            // TODO remove later
            // dbg!(&piped_args);

            if parallel_flag {
                piped_args.into_par_iter().for_each(|piped_arg| {
                    let cmd = build_cmd(&args, piped_arg, replace_flag);
                    // TODO remove later
                    // dbg!(&cmd);

                    run_cmd(&cmd);
                })
            } else {
                piped_args.into_iter().for_each(|piped_arg| {
                    let cmd = build_cmd(&args, piped_arg, replace_flag);
                    // TODO remove later
                    // dbg!(&cmd);

                    run_cmd(&cmd);
                })
            }
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

fn split_pipe_by_lines(pipe: String) -> Vec<String> {
    // handle multiple lines in stdin
    let mut collector = Vec::new();
    for line in pipe.lines() {
        collector.push(line.to_string());
    }

    collector
}

fn chain_args_with_space(args: &Vec<&String>) -> String {
    // chain given arguments together with a spaces inbetween
    let mut strg = String::new();
    for s in args {
        strg.push_str(s);

        if args.iter().last() == Some(&s) {
            break;
        } else {
            strg.push_str(" ");
        }
    }

    // remove leading and trailing whitespace
    strg.trim().to_string()
}

fn build_cmd(cmd_vec: &Vec<&String>, piped_args: String, replace_flag: bool) -> String {
    let cmd = chain_args_with_space(cmd_vec);

    // split given command if it has flags
    // respect placeholders
    let mut combined_cmd = String::new();
    if replace_flag {
        // INFO -> surround '{}' with quotation marks
        combined_cmd.push_str(&cmd.replace("{}", &piped_args));
        // FIXME ignores last given argument(s) when something gets replaced with input from stdin
    } else {
        combined_cmd.push_str(&cmd);
        combined_cmd.push_str(" ");
        combined_cmd.push_str(&piped_args);
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
        .version("1.0.1")
        .author("Leann Phydon <leann.phydon@gmail.com>")
        .arg(
            Arg::new("args")
                .help("The command to execute with an argument from stdin")
                .long_help(format!(
                    "{}\n{}",
                    "The command to execute with an argument from stdin",
                    "Must be the last argument (everything will be treated as a literal string)",
                ))
                .trailing_var_arg(true)
                .value_terminator(";")
                .action(ArgAction::Append)
                .value_name("COMMAND"),
        )
        .arg(
            Arg::new("parallel")
                .short('p')
                .long("parallel")
                .help("Process input in parallel if possible")
                .long_help(format!(
                    "{}\n{}",
                    "Process input in parallel if possible",
                    "The input order will most likely change" // INFO when used with 'mg' (minigrep) -> the performance flag [-p] must be set to make sure, the found matches aren't randomly placed under different filenames because of the parallel processing (and random output)
                ))
                .action(ArgAction::SetTrue),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_pipe_by_lines_test() {
        let pipe = "This\nis\na\ntest".to_string();
        let result = split_pipe_by_lines(pipe);
        let expected = vec!["This", "is", "a", "test"];
        assert_eq!(result, expected);
    }

    #[test]
    fn chain_args_with_space_test() {
        let binding_a = "This".to_string();
        let binding_b = "is".to_string();
        let binding_c = "a".to_string();
        let binding_d = "test".to_string();
        let mut args = Vec::new();

        args.push(&binding_a);
        args.push(&binding_b);
        args.push(&binding_c);
        args.push(&binding_d);

        let result = chain_args_with_space(&args);
        let expected = "This is a test".to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn chain_args_with_space_test2() {
        let binding_a = "This".to_string();
        let binding_b = "is".to_string();
        let binding_c = "a".to_string();
        let binding_d = "test".to_string();
        let binding_e = " ".to_string();
        let mut args = Vec::new();

        args.push(&binding_a);
        args.push(&binding_b);
        args.push(&binding_c);
        args.push(&binding_d);
        args.push(&binding_e);

        let result = chain_args_with_space(&args);
        let expected = "This is a test".to_string();
        assert_eq!(result, expected);
    }
}
