use clap::Parser;
use color_eyre::eyre::Result;
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
}

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

    let hosts = get_hosts(&config);
    let host = Select::new("Select a host", hosts).prompt()?;

    Command::new("sh")
        .arg("-c")
        .arg(format!("{term} -e bash -c 'ssh {}'", host.name))
        .output()?;

    Ok(())
}
