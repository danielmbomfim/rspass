use clap::{
    builder::{PossibleValue, TypedValueParser},
    error::{ContextKind, ContextValue, ErrorKind},
    Arg, Command, Error,
};
use colored::Colorize;
use rspass_core::get_repo_path;

#[derive(Debug, Clone)]
pub enum CredentialItem {
    File(String),
    Dir(String),
}

impl CredentialItem {
    fn is_dir(&self) -> bool {
        match self {
            CredentialItem::Dir(_) => true,
            CredentialItem::File(_) => false,
        }
    }

    fn matches(&self, value: &String) -> bool {
        match self {
            CredentialItem::Dir(credential) => credential == value,
            CredentialItem::File(credential) => credential == value,
        }
    }
}

impl From<std::fs::DirEntry> for CredentialItem {
    fn from(value: std::fs::DirEntry) -> Self {
        let value = value.path();
        let name = value.to_str().unwrap_or("");
        let parts: Vec<_> = name.splitn(2, "rspass/").collect();

        if value.is_dir() {
            CredentialItem::Dir(parts[1].to_owned())
        } else {
            CredentialItem::File(parts[1].to_owned())
        }
    }
}

impl std::fmt::Display for CredentialItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialItem::File(value) => write!(f, "{}", value),
            CredentialItem::Dir(value) => {
                write!(f, "{}{}", value.bright_blue(), "/".bright_blue())
            }
        }
    }
}

pub fn list_credentials(path: Option<String>) -> Option<Vec<CredentialItem>> {
    let mut repo = get_repo_path();

    if let Some(value) = path {
        repo.push(value);
    }

    let credentials: Vec<_> = std::fs::read_dir(repo)
        .ok()?
        .into_iter()
        .flat_map(|value| {
            let value = value.ok()?;

            if value.path().ends_with(".git") {
                return None;
            }

            Some(CredentialItem::from(value))
        })
        .collect();

    Some(credentials)
}

#[derive(Clone, Debug)]
pub struct CredentialValuesParser(Vec<CredentialItem>);

#[allow(dead_code)]
impl CredentialValuesParser {
    pub fn all() -> CredentialValuesParser {
        let items = list_credentials(None);

        CredentialValuesParser(items.unwrap_or_else(Vec::new))
    }

    pub fn dirs() -> CredentialValuesParser {
        let items = list_credentials(None)
            .map(|value| value.into_iter().filter(CredentialItem::is_dir).collect());

        CredentialValuesParser(items.unwrap_or_else(Vec::new))
    }

    pub fn files() -> CredentialValuesParser {
        let items =
            list_credentials(None).map(|value| value.into_iter().filter(|i| !i.is_dir()).collect());

        CredentialValuesParser(items.unwrap_or_else(Vec::new))
    }
}

impl TypedValueParser for CredentialValuesParser {
    type Value = String;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, Error> {
        TypedValueParser::parse(self, cmd, arg, value.to_owned())
    }

    fn parse(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: std::ffi::OsString,
    ) -> Result<String, Error> {
        let mut value = match value
            .into_string()
            .map_err(|_| Error::new(ErrorKind::InvalidUtf8).with_cmd(cmd))
        {
            Ok(value) => value,
            Err(err) => return Err(err),
        };

        if value.ends_with("/") {
            value.pop();
        }

        if self.0.iter().any(|v| v.matches(&value)) {
            Ok(value)
        } else {
            let possible_vals = self.0.iter().collect::<Vec<_>>();

            let mut err = Error::new(ErrorKind::InvalidValue).with_cmd(cmd);
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::String(
                    arg.map(ToString::to_string)
                        .unwrap_or_else(|| "...".to_owned()),
                ),
            );
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::String(
                    arg.map(ToString::to_string)
                        .unwrap_or_else(|| "...".to_owned()),
                ),
            );
            err.insert(ContextKind::InvalidValue, ContextValue::String(value));
            err.insert(
                ContextKind::ValidValue,
                ContextValue::Strings(possible_vals.iter().map(|s| s.to_string()).collect()),
            );

            Err(err)
        }
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(
            self.0
                .iter()
                .cloned()
                .map(|i| PossibleValue::new(i.to_string())),
        ))
    }
}
