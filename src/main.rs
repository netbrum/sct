mod error;
use clap::Parser;
use color_eyre::eyre::Result;
use error::Error;
use inquire::Select;
use ssh2_config::{ParseRule, SshConfig, SshParserResult};
use std::env;
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
}

fn get_hosts(config: &SshConfig) -> Vec<String> {
    let hosts = config
        .get_hosts()
        .iter()
        .filter(|host| host.pattern.first().unwrap().pattern != "*")
        .map(|host| host.pattern.first().unwrap().pattern.clone())
        .collect::<Vec<_>>();

    hosts
}

fn get_config_file(args: Args) -> Result<File> {
    let home_dir = env::var("HOME")?;
    let ssh_config = home_dir + "/.ssh/config";
    let config = args.config.unwrap_or(ssh_config);

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

    let config_path = get_config_file(args)?;
    let config = parse_config(config_path)?;
    let term = env::var("TERM")?;

    if term.is_empty() {
        return Err(Error::TermNotSet.into());
    }

    let hosts = get_hosts(&config);
    let host = Select::new("Select a host", hosts).prompt()?;

    let host_config = config.query(host);

    if let Some(host_name) = host_config.host_name {
        let ssh_cmd = if let Some(user) = host_config.user {
            format!("ssh {}@{}", user, host_name)
        } else {
            format!("ssh {}", host_name)
        };

        Command::new("sh")
            .arg("-c")
            .arg(format!("{term} -e bash -c '{ssh_cmd}'"))
            .output()?;

        Ok(())
    } else {
        Err(Error::MissingHostName.into())
    }
}
