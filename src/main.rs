use color_eyre::eyre::Result;
use inquire::Select;
use ssh2_config::{ParseRule, SshConfig, SshParserResult};
use std::env::{self, VarError};
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::process::Command;

struct Host {
    name: String,
    host: String,
}

impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.host)
    }
}

impl Host {
    fn new(name: &str, host: &str) -> Self {
        Self {
            name: name.to_owned(),
            host: host.to_owned(),
        }
    }
}

fn get_hosts(config: &SshConfig) -> Vec<Host> {
    let hosts = config
        .get_hosts()
        .iter()
        .filter(|host| host.pattern.first().unwrap().pattern != "*")
        .map(|host| {
            Host::new(
                &host.pattern.first().unwrap().pattern,
                &host.params.host_name.clone().unwrap(),
            )
        })
        .collect::<Vec<_>>();

    hosts
}

fn get_config_path() -> Result<PathBuf, VarError> {
    let home_dir = env::var("HOME")?;
    let ssh_config = home_dir + "/.ssh/config";
    let path = PathBuf::from(&ssh_config);

    Ok(path)
}

fn parse_config(config_file: PathBuf) -> SshParserResult<SshConfig> {
    let mut reader = BufReader::new(File::open(config_file)?);
    SshConfig::default().parse(&mut reader, ParseRule::STRICT)
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let config_path = get_config_path()?;
    let config = parse_config(config_path)?;
    let term = env::var("TERM")?;

    let hosts = get_hosts(&config);
    let host = Select::new("Select a host", hosts).prompt()?;

    println!("SSH session started");

    Command::new("sh")
        .arg("-c")
        .arg(format!("{term} -e bash -c 'ssh {}'", host.name))
        .output()?;

    println!("SSH session closed");

    Ok(())
}
