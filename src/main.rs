use clap::Parser;
use rspass_core::{generate_password, initialize_repository, insert_credential};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    Init,
    Grep {
        text: String,
    },
    Find {
        text: String,
    },
    Ls {
        text: String,
    },
    Insert {
        name: String,
        #[arg(required = false)]
        password: Option<String>,
        #[arg(short, long, value_delimiter = ' ', num_args = 1.., value_parser = parse_value)]
        metadata: Option<Vec<(String, String)>>,
    },
    Rm {
        text: String,
    },
    Edit {
        text: String,
    },
    Mv {
        text: String,
    },
}

fn parse_value(value: &str) -> Result<(String, String), String> {
    let parts = value.split_once("=");

    match parts {
        Some(data) => {
            let (key, value) = data;

            Ok((key.to_owned(), value.to_owned()))
        }
        None => Err("Invalid metadata format. expected \"key=value\"".to_owned()),
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Init => initialize_repository(),
        Commands::Insert {
            name,
            password,
            metadata,
        } => {
            let password = password.unwrap_or_else(|| generate_password(10));

            insert_credential(&name, &password, metadata)
        }
        _ => todo!(),
    }
}
