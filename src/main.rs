mod cli;
mod command;
mod cyclonedx;
mod error;
mod model;

use clap::Parser;

use std::process;

fn main() {
    let cli = cli::Cli::parse();

    let result = match cli.subcmd {
        cli::SubCommand::Evaluate(t) => command::evaluate::exec(&t),
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        process::exit(1);
    }
}
