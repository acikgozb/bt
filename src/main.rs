use std::{error, io, process::ExitCode};

use bt::api::{BtCommand, Cli};
use clap::Parser;

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}

fn run() -> Result<(), Box<dyn error::Error>> {
    let args = Cli::parse();

    println!("{:?}", args);

    let mut stdout = io::stdout();
    let mut stdin = io::stdin();

    if let Some(subcommand) = args.command {
        match subcommand {
            BtCommand::Status => bt::status(&mut stdout),
            BtCommand::Toggle => bt::toggle(&mut stdout),
            BtCommand::Scan { args } => bt::scan(&mut stdout, &args),
            BtCommand::Connect { args } => bt::connect(&mut stdout, &mut stdin, &args),
            BtCommand::Disconnect { force, aliases } => todo!(),
            BtCommand::ListDevices { args } => bt::list_devices(&mut stdout, &args),
        }
    } else {
        bt::status(&mut stdout)
    }?;

    Ok(())
}
