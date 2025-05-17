use clap::{Parser, ValueEnum, builder::PossibleValue};
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use inquire::Select;
use ssh2_config::{ParseRule, SshConfig, SshParserResult};
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Term {
    Alacritty,
    Konsole,
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let term = match self {
            Self::Alacritty => "alacritty",
            Self::Konsole => "konsole",
        };

        write!(f, "{term}")
    }
}

impl ValueEnum for Term {
    fn value_variants<'a>() -> &'a [Self] {
        &[Term::Alacritty, Term::Konsole]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Term::Alacritty => PossibleValue::new("alacritty"),
            Term::Konsole => PossibleValue::new("konsole"),
        })
    }
}

#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// Use this config file
    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long, default_value_t = Term::Alacritty)]
    term: Term,
}

struct HostEntry {
    name: String,
    host: String,
    user: Option<String>,
}

impl Display for HostEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.host.bright_black())
    }
}

fn get_hosts(config: &SshConfig) -> Vec<HostEntry> {
    let hosts = config
        .get_hosts()
        .iter()
        .filter(|host| {
            host.pattern.first().unwrap().pattern != "*" && host.params.host_name.is_some()
        })
        .map(|host| HostEntry {
            name: host.pattern.first().unwrap().pattern.clone(),
            host: host.params.host_name.clone().unwrap(),
            user: host.params.user.clone(),
        })
        .collect::<Vec<_>>();

    hosts
}

fn get_config_file(args: &Args) -> Result<File> {
    let home_dir = env::var("HOME")?;
    let ssh_config = home_dir + "/.ssh/config";
    let config = args.config.clone().unwrap_or(ssh_config);

    let path = PathBuf::from(config);

    Ok(File::open(path)?)
}

fn parse_config(file: File) -> SshParserResult<SshConfig> {
    let mut reader = BufReader::new(file);
    SshConfig::default().parse(&mut reader, ParseRule::STRICT)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    let config_path = get_config_file(&args)?;
    let config = parse_config(config_path)?;

    let hosts = get_hosts(&config);
    let host = Select::new("Select host:", hosts).prompt()?;

    let ssh_cmd = if let Some(user) = host.user {
        format!("ssh {}@{}", user, host.name)
    } else {
        format!("ssh {}", host.name)
    };

    Command::new("sh")
        .arg("-c")
        .arg(format!("{} -e bash -c '{ssh_cmd}'", args.term))
        .output()?;

    Ok(())
}
