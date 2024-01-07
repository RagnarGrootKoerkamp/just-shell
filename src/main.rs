#![feature(let_chains, ascii_char)]
use std::{fs::File, io::Read, path::Path, process::Command};

use colored::Colorize;
use rustyline::error::ReadlineError;

fn main() {
    let justfile = read();
    print_rules(justfile);

    let mut rl = rustyline::DefaultEditor::new().unwrap();

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
        let r = Command::new("just")
            .args(line.split_whitespace())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
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
