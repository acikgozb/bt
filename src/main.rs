use std::{error, io, process::ExitCode};

use bt::api::{BtCommand, Cli};
use clap::Parser;

const PROGRAM: &str = "bt";

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{PROGRAM}: {}", e);

            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn error::Error>> {
    let bluez = bt::BluezClient::new()?;

    let args = Cli::parse();

    let mut stdout = io::stdout();
    let stdin = io::stdin();

    if let Some(subcommand) = args.command {
        match subcommand {
            BtCommand::Status => bt::status(&bluez, &mut stdout)?,
            BtCommand::Toggle => bt::toggle(&bluez, &mut stdout)?,
            BtCommand::Scan { args } => bt::scan(&bluez, &mut stdout, &args)?,
            BtCommand::Connect { args } => {
                let mut stdin_handle = stdin.lock();
                bt::connect(&bluez, &mut stdout, &mut stdin_handle, &args)?
            }
            BtCommand::Disconnect { force, aliases } => {
                let mut stdin_handle = stdin.lock();
                bt::disconnect(&bluez, &mut stdout, &mut stdin_handle, &force, &aliases)?
            }
            BtCommand::ListDevices { args } => bt::list_devices(&bluez, &mut stdout, &args)?,
        }
    } else {
        bt::status(&bluez, &mut stdout)?
    };

    Ok(())
}
