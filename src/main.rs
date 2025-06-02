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

    if let Some(subcommand) = args.command {
        match subcommand {
            BtCommand::Status => bt::status(&mut stdout),
            BtCommand::Toggle => bt::toggle(&mut stdout),
            BtCommand::Scan { args } => todo!(),
            BtCommand::Connect { args } => todo!(),
            BtCommand::Disconnect { force } => todo!(),
            BtCommand::ListDevices {
                columns,
                values,
                status,
            } => bt::list_devices(&mut stdout, columns, values, status),
        }
    } else {
        bt::status(&mut stdout)
    }?;

    Ok(())
}
