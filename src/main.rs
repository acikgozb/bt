use bt::api::Cli;
use clap::Parser;

fn main() {
    let args = Cli::parse();

    println!("{:?}", args);
}
