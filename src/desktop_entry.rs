use std::collections::HashMap;
use std::path::PathBuf;
use crate::ParseResult;
use crate::parser::parse_desktop_entry;
use crate::locale::Locale;

#[derive(PartialEq, Debug)]
pub enum StandardKey {
    Type,
    Version,
    Name,
    GenericName,
    NoDisplay,
    Comment,
    Icon,
    Hidden,
    OnlyShowIn,
    NotShowIn,
    DbusActivatable,
    TryExec,
    Exec,
    Path,
    Terminal,
    Actions,
    MimeType,
    Categories,
    Implements,
    Keywords,
    StartupNotify,
    StartupWmClass,
    Url,
}

impl StandardKey {
    pub fn key_name(&self) -> &'static str {
        use StandardKey::*;
        match *self {
            Type => "Type",
            Version => "Version",
            Name => "Name",
            GenericName => "GenericName",
            NoDisplay => "NoDisplay",
            Comment => "Comment",
            Icon => "Icon",
            Hidden => "Hidden",
            OnlyShowIn => "OnlyShowIn",
            NotShowIn => "NotShowIn",
            DbusActivatable => "DBusActivatable",
            TryExec => "TryExec",
            Exec => "Exec",
            Path => "Path",
            Terminal => "Terminal",
            Actions => "Actions",
            MimeType => "MimeType",
            Categories => "Categories",
            Implements => "Implements",
            Keywords => "Keywords",
            StartupNotify => "StartupNotify",
            StartupWmClass => "StartupWMClass",
            Url => "URL",
        }
    }
}




pub struct DesktopEntry<'a> {
    path: PathBuf,
    groups: HashMap<&'a str, HashMap<&'a str, &'a str>>,
}

impl<'a> DesktopEntry<'a> {
    pub fn parse_string(input: &'a str) -> ParseResult<Self> {
        Ok(Self {
            path: PathBuf::new(),
            groups: parse_desktop_entry(input)?
        })
    }

    pub fn get_key(&self, key: StandardKey) -> Option<&str> {
        self.group_get("Desktop Entry", key.key_name())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.group_get("Desktop Entry", key)
    }

    pub fn keys(&self) -> Vec<&str> {
        self.group_keys("Desktop Entry")
    }

    pub fn group_keys(&self, group: &str) -> Vec<&str> {
        self.groups.get(group)
            .map(|grp| grp.keys().map(|&x| x).collect())
            .unwrap_or_else(|| vec![])
    }

    pub fn group_get(&self, group: &str, key: &str) -> Option<&str> {
        self.groups.get(group).and_then(|grp| grp.get(key).map(|&x| x))
    }

    pub fn group_localized_get(
        &self, group: &str, key: &str, locale: &Option<Locale>
    ) -> Option<&str> {
        let entries = if let Some(entries) = self.groups.get(group) {
            entries
        } else {
            return None;
        };

        if let Some(locale) = locale {
            if let Some(country) = locale.country() {
                if let Some(modifier) = locale.modifier() {
                    let lkey = format!(
                        "{}[{}_{}@{}]",  key, locale.lang(), country, modifier);
                    if let Some(result) = entries.get(&(&lkey as &str)) {
                        return Some(result);
                    }
                }
            }

            if let Some(country) = locale.country() {
                let lkey = format!("{}[{}_{}]", key, locale.lang(), country);
                if let Some(result) = entries.get(&(&lkey as &str)) {
                    return Some(result);
                }
            }

            if let Some(modifier) = locale.modifier() {
                let lkey = format!("{}[{}@{}]", key, locale.lang(), modifier);
                if let Some(result) = entries.get(&(&lkey as &str)) {
                    return Some(result);
                }
            }

            let lkey = format!("{}[{}]", key, locale.lang());
            if let Some(result) = entries.get(&(&lkey as &str)) {
                return Some(result);
            }
        }

        entries.get(key).map(|&x| x)
    }
}