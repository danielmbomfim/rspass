use clap::Parser;
use colored::Colorize;
use rspass_core::{
    generate_keys, generate_password, get_credential, initialize_repository, insert_credential,
};

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
    Get {
        name: String,
        #[arg(short, long, default_value = "false")]
        full: bool,
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
            let mut name = String::new();
            let mut email = String::new();
            let mut password = String::new();

            println!("Enter the name to be used in the pgp key");
            std::io::stdin()
                .read_line(&mut name)
                .expect("failed to get read name");

            println!("Enter the email to be used in the pgp key");
            std::io::stdin()
                .read_line(&mut email)
                .expect("failed to get read email");

            println!("Enter the password to be used in the pgp key");
            std::io::stdin()
                .read_line(&mut password)
                .expect("failed to get read password");

            match generate_keys(name.trim(), email.trim(), password.trim()) {
                Ok(path) => {
                    println!("\nKeys generated at {path}");
                    println!("{} Do NOT share this key files! Anyone with access to them may be able to decrypt and use your data.", "WARNING:".yellow());
                }
                Err(err) => {
                    eprintln!("{}", format_err(err));
                    return;
                }
            }

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
        Commands::Get { name, full } => {
            let mut password = String::new();

            println!("Enter the password of the PGP key:");
            std::io::stdin().read_line(&mut password).unwrap();

            match get_credential(&name, password.trim(), full) {
                Ok(credential) => println!("{}", credential),
                Err(err) => eprintln!("{}", format_err(err)),
            }
        }
        _ => todo!(),
    }
}

fn format_err(err: rspass_core::Error) -> String {
    let kind = format!("{:?}", err.kind).red();

    format!("{} :: {}", kind, err.message)
}
