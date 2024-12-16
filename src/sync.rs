use rspass_core::{
    add_remote, fetch_from_remote, get_credential, insert_credential, push_to_remote, Error,
};

const GIT_CREDENTIAL: &'static str = "config/rspass";

fn get_git_credential(password: &str) -> Result<(String, String), Error> {
    let credential = get_credential(GIT_CREDENTIAL, password, true)?;
    let lines: Vec<&str> = credential.lines().collect();

    let username = lines[2].split_once("=").unwrap().1;

    Ok((username.to_owned(), lines[0].to_owned()))
}

pub fn set_remote(username: &str, password: &str, uri: &str) -> Result<(), Error> {
    insert_credential(
        GIT_CREDENTIAL,
        password,
        Some(vec![
            ("uri".to_owned(), uri.to_owned()),
            ("username".to_owned(), username.to_owned()),
        ]),
    )?;
    add_remote(uri)?;

    Ok(())
}

pub fn sync_data(password: &str) -> Result<(), Error> {
    let (username, password) = get_git_credential(&password)?;

    fetch_from_remote(&username, &password)?;
    push_to_remote(&username, &password)?;

    Ok(())
}
