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
        self.inner() == value
    }

    fn inner(&self) -> &String {
        match self {
            CredentialItem::Dir(s) => s,
            CredentialItem::File(s) => s,
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

pub fn list_credentials(path: Option<String>, recursive: bool) -> Option<Vec<CredentialItem>> {
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

            let item_path = &value.path();
            let item = CredentialItem::from(value);

            if recursive && item.is_dir() {
                let mut sub_items =
                    list_credentials(Some(item_path.to_string_lossy().to_string()), recursive)?;

                sub_items.push(item);
                return Some(sub_items.into_iter());
            }

            Some(vec![item].into_iter())
        })
        .flatten()
        .collect();

    Some(credentials)
}

#[derive(Clone, Debug)]
pub struct CredentialValuesParser(Vec<CredentialItem>);

impl CredentialValuesParser {
    #[allow(dead_code)]
    pub fn all() -> CredentialValuesParser {
        let items = list_credentials(None, true);

        CredentialValuesParser(items.unwrap_or_else(Vec::new))
    }

    pub fn dirs() -> CredentialValuesParser {
        let items = list_credentials(None, true)
            .map(|value| value.into_iter().filter(CredentialItem::is_dir).collect());

        CredentialValuesParser(items.unwrap_or_else(Vec::new))
    }

    pub fn files() -> CredentialValuesParser {
        let items = list_credentials(None, true)
            .map(|value| value.into_iter().filter(|i| !i.is_dir()).collect());

        CredentialValuesParser(items.unwrap_or_else(Vec::new))
    }

    fn filter_by_base(&self, value: &str) -> Vec<&CredentialItem> {
        let parts: Vec<_> = value.split("/").collect();

        let base = match parts.split_last() {
            Some((_, base)) => Some(base.join("/")),
            None => None,
        };

        self.0
            .iter()
            .filter(|item| match base.as_ref() {
                Some(base) => item.inner().starts_with(base),
                None => true,
            })
            .collect()
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
            let possible_vals = self.filter_by_base(&value);

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
