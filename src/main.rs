#![feature(let_chains, ascii_char)]
use std::{fs::File, io::Read, path::Path, process::Command};

use colored::Colorize;
use rustyline::completion::FilenameCompleter;
use rustyline::{error::ReadlineError, hint::HistoryHinter, history::DefaultHistory};
use rustyline::{Completer, Helper, Highlighter, Hinter, Validator};

#[derive(Helper, Completer, Hinter, Validator, Highlighter)]
struct MyHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}

fn main() {
    let justfile = read();
    print_rules(justfile);

    let mut rl = rustyline::Editor::<MyHelper, DefaultHistory>::new().unwrap();
    rl.set_helper(Some(MyHelper {
        completer: FilenameCompleter::new(),
        hinter: HistoryHinter::new(),
    }));

    let prompt = format!("{}> ", "Just".bold().red());
    loop {
        let line = rl.readline(&prompt);
        let line = match line {
            Ok(line) => line,
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };
        let r = run(line.split_whitespace());
        if r.success() {
            rl.add_history_entry(&line).unwrap();
        } else {
            eprintln!(
                "! exit code: {}",
                r.code().unwrap().to_string().bold().red()
            );
        }
    }
}

// Rule has one of the forms:
// - <rule>: [deps]
// - <rule> <arg1> ...: [deps]
struct Rule {
    name: String,
    args: Vec<String>,
}

// Alias has the form:
// alias <alias> := <rule>
struct Alias {
    alias: String,
    rule: String,
}

struct Justfile {
    rules: Vec<Rule>,
    aliases: Vec<Alias>,
}

fn read() -> Justfile {
    let justfile_path = Path::new("justfile");
    let mut justfile = String::new();
    File::open(justfile_path)
        .unwrap()
        .read_to_string(&mut justfile)
        .unwrap();

    let mut rules = Vec::new();
    let mut aliases = Vec::new();

    for line in justfile.lines() {
        if line.is_empty() {
            continue;
        }
        if let Some(line) = line.strip_prefix("alias ") {
            if let Some((alias, rule)) = line.split_once(":=") {
                aliases.push(Alias {
                    alias: alias.trim().to_string(),
                    rule: rule.trim().to_string(),
                });
            }
            continue;
        }
        if let Some((line, _deps)) = line.split_once(':')
            && !line.starts_with(' ')
        {
            let mut parts = line.split_whitespace();
            let name = parts.next().unwrap().trim().to_string();
            let args = parts.map(|s| s.to_string()).collect();
            rules.push(Rule { name, args });
        }
    }

    Justfile { rules, aliases }
}

fn print_rules(justfile: Justfile) {
    for rule in justfile.rules {
        eprint!("{}", rule.name);
        for arg in rule.args {
            eprint!(" {}", arg);
        }
        eprintln!();
    }
    for alias in justfile.aliases {
        eprintln!("{}: {}", alias.alias, alias.rule);
    }
}

fn run<I, S>(args: I) -> std::process::ExitStatus
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new("just")
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
}
