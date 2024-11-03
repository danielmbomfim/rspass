use clap::Parser;
use colored::Colorize;
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
        #[arg(short, default_value = "10")]
        length: u8,
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
        Commands::Init => {
            match initialize_repository() {
                Ok(path) => println!("repository initialized in {}", path),
                Err(err) => eprintln!("{}", format_err(err)),
            };
        }
        Commands::Insert {
            name,
            password,
            metadata,
            length,
        } => {
            let password = password.unwrap_or_else(|| {
                println!("generating password with {length} characters generated");

                generate_password(length as usize)
            });

            match insert_credential(&name, &password, metadata) {
                Ok(_) => println!("Credential saved"),
                Err(err) => eprintln!("{}", format_err(err)),
            };
        }
        _ => todo!(),
    }
}

fn format_err(err: rspass_core::Error) -> String {
    let kind = format!("{:?}", err.kind).red();

    format!("{} :: {}", kind, err.message)
}
