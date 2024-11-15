use clap::Parser;
use colored::Colorize;
use crossterm::{
    cursor::MoveUp,
    execute,
    terminal::{Clear, ClearType},
};
use rspass_core::{
    edit_credential, generate_keys, generate_password, get_credential, get_credentials,
    initialize_repository, insert_credential, move_credential, remove_credential,
};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    Init,
    Ls {
        name: Option<String>,
    },
    Insert {
        name: String,
        #[arg(required = false)]
        password: Option<String>,
        #[arg(short, long, value_delimiter = ' ', num_args = 1.., value_parser = parse_key_value)]
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
        name: String,
    },
    Edit {
        name: String,
        password: Option<String>,
        #[arg(short, long, value_delimiter = ' ', num_args = 1.., value_parser = parse_key_value)]
        add_metadata: Option<Vec<(String, String)>>,
        #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
        remove_metadata: Option<Vec<String>>,
    },
    Mv {
        target: String,
        destination: String,
    },
}

fn parse_key_value(value: &str) -> Result<(String, String), String> {
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

            println!("Enter the name to be used in the pgp key:");
            std::io::stdin()
                .read_line(&mut name)
                .expect("failed to get read name:");

            println!("Enter the email to be used in the pgp key:");
            std::io::stdin()
                .read_line(&mut email)
                .expect("failed to get read email");

            let password =
                rpassword::prompt_password("Enter the password to be used in the pgp key:")
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
            let password = rpassword::prompt_password("Enter the password of the PGP key:")
                .expect("failed to get read password");

            match get_credential(&name, password.trim(), full) {
                Ok(credential) => {
                    execute!(
                        std::io::stdout(),
                        MoveUp(2),
                        Clear(ClearType::FromCursorDown)
                    )
                    .unwrap();
                    println!("{}", credential);
                }
                Err(err) => eprintln!("{}", format_err(err)),
            }
        }
        Commands::Edit {
            name,
            password,
            add_metadata,
            remove_metadata,
        } => {
            let mut metadata = Vec::new();

            if let Some(data) = add_metadata {
                data.into_iter().for_each(|(key, value)| {
                    metadata.push((key, Some(value)));
                });
            }

            if let Some(data) = remove_metadata {
                data.into_iter().for_each(|key| {
                    metadata.push((key, None));
                });
            }

            let pgp_password = rpassword::prompt_password("Enter the pgp_password of the PGP key:")
                .expect("failed to get read password");

            match edit_credential(
                &name,
                pgp_password.trim(),
                password.as_deref(),
                if metadata.is_empty() {
                    None
                } else {
                    Some(metadata)
                },
            ) {
                Ok(_) => {
                    execute!(
                        std::io::stdout(),
                        MoveUp(2),
                        Clear(ClearType::FromCursorDown)
                    )
                    .unwrap();
                    println!("Credential saved");
                }
                Err(err) => eprintln!("{}", format_err(err)),
            }
        }
        Commands::Rm { name } => match remove_credential(&name) {
            Ok(_) => println!("Credential removed"),
            Err(err) => eprintln!("{}", format_err(err)),
        },
        Commands::Mv {
            target,
            destination,
        } => match move_credential(&target, &destination) {
            Ok(_) => println!("Credential moved"),
            Err(err) => eprintln!("{}", format_err(err)),
        },
        Commands::Ls { name } => {
            let list: Vec<String> = get_credentials(name.as_deref())
                .iter()
                .map(|item| {
                    let parts: Vec<&str> = item.splitn(2, "rspass/").collect();
                    parts[1].to_owned()
                })
                .collect();

            println!("{}", list.join("\n"));
        }
    }
}

fn format_err(err: rspass_core::Error) -> String {
    let kind = format!("{:?}", err.kind).red();

    format!("{} :: {}", kind, err.message)
}
