#[allow(unused_imports)]
use dirs::home_dir;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::{env, path};

static BUILTINS: [&str; 6] = ["echo", "exit", "type", "pwd", "cd", "history"];

fn is_executable(path: &path::Path) -> bool {
    path.is_file()
        && path
            .metadata()
            .map(|m| m.permissions().readonly() == false)
            .unwrap_or(false)
}

fn load_executables() -> HashMap<String, String> {
    let mut executables: HashMap<String, String> = HashMap::new();
    let path_var = match env::var("PATH") {
        Ok(path_val) => path_val,
        Err(_) => String::new(),
    };
    let separator = if cfg!(windows) { ";" } else { ":" };

    for dir in path_var.split(separator) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if is_executable(&path) && !executables.contains_key(name) {
                        executables.insert(name.to_string(), path.to_str().unwrap().to_string());
                    }
                }
            }
        }
    }

    executables
}

fn run_builtin(cmd: String, args: Vec<String>) -> Option<String> {
    let executables = load_executables();
    match cmd.as_str() {
        "echo" => Some(args.join(" ") + "\n"),
        "pwd" => env::current_dir()
            .ok()
            .map(|p| p.display().to_string() + "\n"),
        "exit" => std::process::exit(args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0)),
        "type" => Some(if let Some(arg) = args.get(0) {
            if BUILTINS.contains(&arg.as_str()) {
                format!("{} is a shell builtin\n", arg)
            } else if let Some(path) = executables.get(arg) {
                format!("{} is {}\n", arg, path)
            } else {
                format!("{}: not found\n", arg)
            }
        } else {
            format!("type: missing operand\n")
        }),
        "cd" => {
            if let Some(dir) = args.get(0) {
                let path = home_dir()
                    .map(|mut d| {
                        d.push(&dir);
                        d
                    })
                    .unwrap()
                    .display()
                    .to_string();
                if let Err(e) = env::set_current_dir(&path) {
                    Some(format!("cd: {}: {}\n", dir, e))
                } else {
                    None
                }
            } else {
                None
            }
        }
        // "history" => {
        //     let mut out = String::new();
        //     for (idx, line) in history.history().iter().enumerate() {
        //         out.push_str(&format!("{:>4}  {}\n", idx + 1, line));
        //     }
        //     Some(out)
        // }
        _ => None,
    }
}

fn parse_arguments(
    command: String,
) -> (Vec<String>, Vec<Vec<String>>, Option<String>, String, bool) {
    let mut cmd = Vec::new();
    let mut args = Vec::new();
    let mut filename = None;
    let mut redirect_mode = String::new();

    let command_split: Vec<String> = shlex::split(&command).unwrap_or_default();
    let has_pipe = command_split.contains(&"|".to_string());

    let mut command_parts = Vec::new();
    let mut current_part = Vec::new();

    for part in command_split {
        if part == "|" {
            command_parts.push(current_part);
            current_part = Vec::new();
        } else {
            current_part.push(part);
        }
    }
    if !current_part.is_empty() {
        command_parts.push(current_part);
    }

    let redir_modes = ["1>", "2>", ">", ">>", "1>>", "2>>"];

    for part in command_parts {
        let mut part_iter = VecDeque::from(part);
        if let Some(command_name) = part_iter.pop_front() {
            cmd.push(command_name);
        }

        let mut part_args = Vec::new();
        while let Some(arg) = part_iter.pop_front() {
            if redir_modes.contains(&arg.as_str()) {
                if let Some(fname) = part_iter.pop_front() {
                    filename = Some(fname);
                    redirect_mode = arg;
                }
                break;
            } else {
                part_args.push(arg);
            }
        }
        args.push(part_args);
    }

    return (cmd, args, filename, redirect_mode, has_pipe);
}

fn parse_command(command: String) {
    let executables = load_executables();
    let (cmds, args, _filename, _redirect_mode, _has_pipe) = parse_arguments(command);

    for i in 0..cmds.len() {
        let cmd = cmds.get(i).unwrap().to_string();
        let cmd_args = args.get(i).unwrap();
        if BUILTINS.contains(&cmd.as_str()) {
            if let Some(output) = run_builtin(cmd, cmd_args.to_vec()) {
                print!("{}", output);
            }
        } else if let Some(_path) = executables.get(cmd.as_str()) {
            // let mut child = Child::new();
            // let mut stdio = Stdio::new();
        } else {
            print!("{}: command not found\n", cmd);
        }
    }
}

fn repl() {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    print!("$ ");
    stdout.flush().unwrap();

    // Wait for user input
    let mut command = String::new();
    stdin.read_line(&mut command).unwrap();

    parse_command(command);

    repl();
}

fn main() {
    repl();
}
