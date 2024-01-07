#![feature(let_chains, ascii_char)]
use std::{fs::File, io::Read, path::Path, process::Command};

use colored::Colorize;
use fuzzy_matcher::FuzzyMatcher;
use rustyline::hint::Hinter;
use rustyline::{error::ReadlineError, history::DefaultHistory};
use rustyline::{Completer, Helper, Highlighter, Validator};

type Matcher = fuzzy_matcher::skim::SkimMatcherV2;

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

thread_local! {
    static MATCHER: Matcher = Matcher::default();
}

impl Justfile {
    fn matches(&self, pattern: &str) -> Vec<(&Rule, (i64, Vec<usize>))> {
        MATCHER.with(|m| {
            let mut matches: Vec<_> = self
                .rules
                .iter()
                .filter_map(|r| Some((r, m.fuzzy_indices(&r.name, pattern)?)))
                .collect();
            matches.sort_by_key(|(_r, (match_score, _indices))| -*match_score);
            matches
        })
    }
    fn best_match(&self, pattern: Option<&str>) -> Option<&Rule> {
        let pattern = pattern.unwrap_or("");
        if pattern.is_empty() {
            return self.rules.first();
        }
        MATCHER.with(|m| {
            let (_score, rule) = self
                .rules
                .iter()
                .rev()
                .filter_map(|r| Some((m.fuzzy_match(&r.name, pattern)?, r)))
                .max_by_key(|&(match_score, _)| match_score)?;
            Some(rule)
        })
    }
}

#[derive(Helper, Completer, Validator, Highlighter)]
struct MyHinter<'j> {
    justfile: &'j Justfile,
}

impl<'j> Hinter for MyHinter<'j> {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        let matches = self.justfile.matches(line);
        if matches.is_empty() {
            return None;
        }

        let mut s = String::new();
        let mut first = true;
        for (rule, (_score, match_positions)) in &matches[..matches.len().min(10)] {
            if !first {
                s.push_str(", ");
            }
            let mut j = 0;
            for (i, c) in rule.name.chars().enumerate() {
                if match_positions.get(j) == Some(&i) {
                    if first {
                        s.push_str(&format!("{}", c.to_string().bold().underline().green()));
                    } else {
                        s.push_str(&format!("{}", c.to_string().bold().underline()));
                    }
                    j += 1;
                } else {
                    if first {
                        s.push_str(&format!("{}", c.to_string().bold().green()));
                    } else {
                        s.push(c);
                    }
                }
            }
            first = false;
        }

        let padding = (pos + 1).next_multiple_of(10) - pos;
        Some(format!(" {:>padding$}({s})", "", padding = padding))
    }
}

fn main() {
    ctrlc::set_handler(|| {}).unwrap();
    let justfile = read();
    print_rules(&justfile);

    let mut rl = rustyline::Editor::<MyHinter, DefaultHistory>::new().unwrap();
    rl.set_helper(Some(MyHinter {
        justfile: &justfile,
    }));

    rl.bind_sequence(rustyline::KeyEvent::ctrl('k'), rustyline::Cmd::AcceptLine);

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
        let mut args = line.split_whitespace();
        let rule = justfile.best_match(args.next()).unwrap();

        let r = run(rule, args);
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

fn print_rules(justfile: &Justfile) {
    for rule in &justfile.rules {
        eprint!("{}", rule.name);
        for arg in &rule.args {
            eprint!(" {}", arg);
        }
        eprintln!();
    }
    for alias in &justfile.aliases {
        eprintln!("{}: {}", alias.alias, alias.rule);
    }
}

fn run<I, S>(r: &Rule, args: I) -> std::process::ExitStatus
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new("just")
        .arg(&r.name)
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
}
