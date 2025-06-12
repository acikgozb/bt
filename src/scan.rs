use std::{error, io};

use clap::Args;

#[derive(Debug, Args)]
pub struct ScanArgs {
    /// Set the duration of the scan.
    #[arg(short, long, default_value_t = 5u8)]
    pub duration: u8,

    /// Filter the pretty output based on given columns.
    #[arg(short, long, value_delimiter = ',')]
    pub columns: Option<Vec<BtScanColumns>>,

    /// Filter the terse output based on given columns.
    #[arg(short, long, value_delimiter = ',')]
    pub values: Option<Vec<BtScanColumns>>,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum BtScanColumns {
    Alias,
    Address,
    RSSI,
}

pub fn scan(f: &mut impl io::Write, args: &ScanArgs) -> Result<(), Box<dyn error::Error>> {
    todo!()
}
