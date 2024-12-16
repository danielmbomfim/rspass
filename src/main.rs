use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use colored::Colorize;
use crossterm::{
    cursor::MoveUp,
    execute,
    terminal::{Clear, ClearType},
};
use rspass_core::{
    edit_credential, generate_keys, generate_password, get_credential, initialize_repository,
    insert_credential, move_credential, remove_credential,
};
use sync::{set_remote, sync_data};
use validators::{list_credentials, CredentialValuesParser};

mod sync;
mod validators;

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    GenerateCompletions {
        #[arg(default_value = Shell::from_env().unwrap_or(Shell::Bash).to_string())]
        shell: Shell,
    },
    Init,
    #[command(subcommand)]
    Syncronization(SyncCommands),
    Ls {
        #[arg(value_parser = CredentialValuesParser::dirs())]
        name: Option<String>,
    },
    Insert {
        name: String,
        #[arg(short, long, value_delimiter = ' ', num_args = 1.., value_parser = parse_key_value)]
        metadata: Option<Vec<(String, String)>>,
        #[arg(short, default_value = "10")]
        length: u8,
    },
    Get {
        #[arg(value_parser = CredentialValuesParser::files())]
        name: String,
        #[arg(short, long, default_value = "false")]
        full: bool,
    },
    Rm {
        #[arg(value_parser = CredentialValuesParser::files())]
        name: String,
    },
    Edit {
        #[arg(value_parser = CredentialValuesParser::files())]
        name: String,
        #[arg(short, long, default_value = "false")]
        password: bool,
        #[arg(short, long, value_delimiter = ' ', num_args = 1.., value_parser = parse_key_value)]
        add_metadata: Option<Vec<(String, String)>>,
        #[arg(short, long, value_delimiter = ' ', num_args = 1..)]
        remove_metadata: Option<Vec<String>>,
    },
    Mv {
        #[arg(value_parser = CredentialValuesParser::files())]
        target: String,
        destination: String,
    },
}

#[derive(Debug, Subcommand)]
enum SyncCommands {
    Config,
    Exec,
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
        Commands::GenerateCompletions { shell } => {
            generate(
                shell,
                &mut Args::command(),
                "rspass",
                &mut std::io::stdout(),
            );
        }
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
                rpassword::prompt_password("Enter the password to be used in the pgp key: ")
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
        Commands::Syncronization(sub_command) => match sub_command {
            SyncCommands::Config => {
                let mut uri = String::new();
                let mut username = String::new();

                println!("Enter the uri of the repository to be set as remote:");
                std::io::stdin()
                    .read_line(&mut uri)
                    .expect("failed to get read uri");

                println!("Enter your git username:");
                std::io::stdin()
                    .read_line(&mut username)
                    .expect("failed to get read username");

                let password = rpassword::prompt_password(
                    "Enter the git password or your PAT in the case of github: ",
                )
                .expect("failed to get read password");

                match set_remote(username.trim(), &password, uri.trim()) {
                    Ok(_) => {
                        execute!(
                            std::io::stdout(),
                            MoveUp(5),
                            Clear(ClearType::FromCursorDown)
                        )
                        .unwrap();
                    }
                    Err(err) => eprintln!("{}", format_err(err)),
                }
            }
            SyncCommands::Exec => {
                let password = rpassword::prompt_password("Enter the password of the PGP key: ")
                    .expect("failed to get read password");

                match sync_data(&password) {
                    Ok(_) => {
                        execute!(
                            std::io::stdout(),
                            MoveUp(1),
                            Clear(ClearType::FromCursorDown)
                        )
                        .unwrap();

                        print!("Synchronization complete!");
                    }
                    Err(err) => eprintln!("{}", format_err(err)),
                }
            }
        },
        Commands::Insert {
            name,
            metadata,
            length,
        } => {
            let password = rpassword::prompt_password(
                "Enter your password or press Enter to generate a random one: ",
            )
            .expect("failed to get read password");

            let password = if password.is_empty() {
                generate_password(length as usize)
            } else {
                password.trim().to_owned()
            };

            match insert_credential(&name, &password, metadata) {
                Ok(_) => println!("Credential saved"),
                Err(err) => eprintln!("{}", format_err(err)),
            };
        }
        Commands::Get { name, full } => {
            let password = rpassword::prompt_password("Enter the password of the PGP key: ")
                .expect("failed to get read password");

            match get_credential(&name, password.trim(), full) {
                Ok(credential) => {
                    if atty::is(atty::Stream::Stdout) {
                        execute!(
                            std::io::stdout(),
                            MoveUp(1),
                            Clear(ClearType::FromCursorDown)
                        )
                        .unwrap();
                    }

                    print!("{}", credential);
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

            let mut credential_password = None;

            if password {
                credential_password = Some(
                    rpassword::prompt_password("Enter your new password: ")
                        .expect("failed to get read password"),
                );
            }

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

            let pgp_password =
                rpassword::prompt_password("Enter the pgp_password of the PGP key: ")
                    .expect("failed to get read password");

            match edit_credential(
                &name,
                pgp_password.trim(),
                credential_password.as_deref(),
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
            if let Some(values) = list_credentials(name, false) {
                values.iter().for_each(|item| println!("{}", item));
            }
        }
    }
}

fn format_err(err: rspass_core::Error) -> String {
    let kind = format!("{:?}", err.kind).red();

    format!("{} :: {}", kind, err.message)
}
