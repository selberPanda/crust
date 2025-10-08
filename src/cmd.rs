use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = "crust", version)]
pub struct App {
    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    Install {
        #[clap(value_delimiter = ' ', num_args = 1..)]
        pkgname: Vec<String>,
    },
    Remove {
        #[clap(value_delimiter = ' ', num_args = 1..)]
        pkgname: Vec<String>,
    },
    Update {},
}
