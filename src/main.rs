use clap::Parser;
use rspass_core::initialize_repository;

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    Init,
    Grep { text: String },
    Find { text: String },
    Ls { text: String },
    Insert { text: String },
    Rm { text: String },
    Edit { text: String },
    Mv { text: String },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Init => initialize_repository(),
        _ => todo!(),
    }
}
