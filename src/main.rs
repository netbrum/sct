mod error;
use clap::Parser;
use color_eyre::eyre::Result;
use color_eyre::owo_colors::OwoColorize;
use error::Error;
use inquire::Select;
use ssh2_config::{ParseRule, SshConfig, SshParserResult};
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(version)]
pub struct Args {
    /// Use this config file
    #[arg(short, long)]
    config: Option<String>,

    #[arg(short, long)]
    term: Option<String>,
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
        .filter_map(|host| {
            host.pattern
                .first()
                .filter(|hc| hc.pattern != "*")
                .and_then(|_| {
                    host.params.host_name.clone().map(|host_name| HostEntry {
                        name: host.pattern.first().unwrap().pattern.clone(),
                        host: host_name,
                        user: host.params.user.clone(),
                    })
                })
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

    let env_term = env::var("TERM")
        .ok()
        .filter(|s| !s.is_empty())
        .ok_or(Error::TermNotSet)?;

    let term = args.term.as_ref().unwrap_or(&env_term);

    let config_path = get_config_file(&args)?;
    let config = parse_config(config_path)?;

    let hosts = get_hosts(&config);
    let host = Select::new("Select host:", hosts).prompt()?;

    let ssh_cmd = if let Some(user) = host.user {
        format!("ssh {}@{}", user, host.name)
    } else {
        format!("ssh {}", host.name)
    };

    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("{term} -e bash -c '{ssh_cmd}'"))
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stderr.is_empty() {
        print!("{stderr}");
    }

    Ok(())
}
