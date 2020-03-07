use std::collections::HashMap;
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
    groups: HashMap<&'a str, HashMap<&'a str, &'a str>>,
}

impl<'a> DesktopEntry<'a> {
    pub fn parse_string(input: &'a str) -> ParseResult<Self> {
        Ok(Self {
            groups: parse_desktop_entry(input)?
        })
    }

    pub fn from_group_values(input: HashMap<&'a str, HashMap<&'a str, &'a str>>) -> Self {
        Self {
            groups: input
        }
    }

    pub fn get_key(&self, key: StandardKey) -> Option<&str> {
        self.group_get("Desktop Entry", key.key_name())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.group_get("Desktop Entry", key)
    }
    pub fn localized_get(&self, key: &str, locale: &Option<Locale>) -> Option<&str> {
        self.group_localized_get("Desktop Entry", key, locale)
    }

    pub fn keys(&self) -> Vec<&str> {
        self.group_keys("Desktop Entry")
    }

    pub fn group_keys(&self, group: &str) -> Vec<&str> {
        self.groups.get(group)
            .map(|grp| grp.keys().copied().collect())
            .unwrap_or_else(|| vec![])
    }

    pub fn group_get(&self, group: &str, key: &str) -> Option<&str> {
        self.groups.get(group).and_then(|grp| grp.get(key).copied())
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

        entries.get(key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_access() {
        let entry = DesktopEntry::from_group_values(hashmap!(
            "Desktop Entry" => hashmap!(
                "Name" => "Test"
            )
        ));

        assert_eq!(entry.get("Name"), Some("Test"));
        assert_eq!(entry.get("Exec"), None);
        assert_eq!(entry.keys(), vec!["Name"]);
        assert_eq!(entry.get_key(StandardKey::Name), Some("Test"));
    }

    #[test]
    fn test_local_access_all() {
        let locale = Locale::from_string("de_DE");
        let entry = DesktopEntry::from_group_values(hashmap!(
            "Desktop Entry" => hashmap!(
                "Name[en]" => "App",
                "Name[de]" => "Anw",
                "Name[de_DE]" => "Anwend",
                "Name[de_DE@nord]" => "Anwendung",
            )
        ));

        assert_eq!(entry.localized_get("Name", &locale), Some("Anwend"));
    }

    #[test]
    fn test_local_access_inaccurate() {
        let locale = Locale::from_string("de_DE");
        let entry = DesktopEntry::from_group_values(hashmap!(
            "Desktop Entry" => hashmap!(
                "Name" => "App",
                "Name[de]" => "Anw",
            )
        ));

        assert_eq!(entry.localized_get("Name", &locale), Some("Anw"));
    }

    #[test]
    fn test_local_access_no() {
        let locale = Locale::from_string("de_DE");
        let entry = DesktopEntry::from_group_values(hashmap!(
            "Desktop Entry" => hashmap!(
                "Name" => "App",
            )
        ));

        assert_eq!(entry.localized_get("Name", &locale), Some("App"));
    }
}