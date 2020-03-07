use nom::{
    IResult,
    character::complete::char,
    sequence::{tuple, preceded},
    combinator::all_consuming
};
use nom::character::complete::alpha1;
use nom::combinator::opt;
use nom::bytes::complete::take_while1;
use std::fmt::{Debug, Formatter, Error};
use std::env;


fn parse_locale(input: &str) -> Option<Locale> {
    let locale: IResult<&str, _> = all_consuming(tuple((
        alpha1,
        opt(preceded(char('_'), alpha1)),
        opt(preceded(char('.'), take_while1(|c: char| c != '@'))),
        opt(preceded(char('@'), take_while1(|c: char| c != '@'))),
    )))(input);

    if let Ok((_, (lang, country, _encoding, modifier))) = locale {
        Some(Locale::new(lang, country, modifier))
    } else {
        None
    }
}


pub struct Locale {
    lang: String,
    country: Option<String>,
    modifier: Option<String>,
}

impl Locale {
    pub fn new(lang: &str, country: Option<&str>, modifier: Option<&str>) -> Self {
        Self {
            lang: lang.to_string(),
            country: country.map(|x| x.to_string()),
            modifier: modifier.map(|x| x.to_string()),
        }
    }

    pub fn from_env() -> Option<Self> {
        if let Ok(lc_messages) = env::var("LC_MESSAGES") {
            parse_locale(&lc_messages)
        } else if let Ok(lc_all) = env::var("LC_ALL") {
            parse_locale(&lc_all)
        } else {
            None
        }
    }

    pub fn from_string(lang_str: &str) -> Option<Self> {
        parse_locale(lang_str)
    }

    pub fn lang(&self) -> &str { &self.lang }
    pub fn country(&self) -> Option<&str> { self.country.as_ref().map(|x| x as &str) }
    pub fn modifier(&self) -> Option<&str> { self.modifier.as_ref().map(|x| x as &str) }
}

impl PartialEq for Locale {
    fn eq(&self, other: &Self) -> bool {
        (&self.lang, &self.country, &self.modifier)
            == (&other.lang, &other.country, &other.modifier)
    }
}

impl Debug for Locale {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_tuple("Locale")
            .field(&self.lang)
            .field(&self.country)
            .field(&self.modifier)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_locale() {
        assert_eq!(parse_locale("C"), Some(Locale::new("C", None, None)));

        assert_eq!(parse_locale("en_US.UTF-8"), Some(Locale::new("en", Some("US"), None)));
        assert_eq!(parse_locale("en_US"), Some(Locale::new("en", Some("US"), None)));
        assert_eq!(parse_locale("en"), Some(Locale::new("en", None, None)));
        assert_eq!(parse_locale("en.UTF-8"), Some(Locale::new("en", None, None)));

        assert_eq!(parse_locale("sr@latin"), Some(Locale::new("sr", None, Some("latin"))));
        assert_eq!(parse_locale("sr.UTF-8@latin"), Some(Locale::new("sr", None, Some("latin"))));

        assert_eq!(parse_locale(""), None);
        assert_eq!(parse_locale("sr.UTF@8@latin"), None);
        assert_eq!(parse_locale("en-US"), None);
        assert_eq!(parse_locale("abc xyz"), None);
        assert_eq!(parse_locale("0x0407"), None);
    }
}