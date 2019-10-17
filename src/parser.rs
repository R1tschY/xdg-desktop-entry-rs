use std::collections::HashMap;
use nom::{
    multi::many0,
    character::complete::space1,
    IResult,
    bytes::complete::take_while,
    character::complete::char,
    sequence::{separated_pair, tuple, preceded, terminated, pair},
    character::complete::{space0},
    branch::alt,
    combinator::{map, cut},
    combinator::{all_consuming}
};
use crate::errors::{ParseResult, ParseError};
use nom::sequence::delimited;
use nom::combinator::{recognize, opt};
use nom::bytes::complete::take_while1;

fn eof(input: &str) -> IResult<&str, &str> {
    eof!(input,)
}

fn eol(input: &str) -> IResult<&str, ()> {
    alt((map(char('\n'), |_| ()), map(eof, |_| ())))(input)
}

fn empty_line(input: &str) -> IResult<&str, ()> {
    alt((
        preceded(space0, map(char('\n'), |_| ())),
        preceded(space1, map(eof, |_| ())),
    ))(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    delimited(char('#'), take_while(|c| c != '\n'), eol)(input)
}

fn group_header(input: &str) -> IResult<&str, &str> {
    preceded(
        char('['),
        cut(terminated(
            take_while(|c| c != ']'),
            pair(char(']'), eol)
        ))
    )(input)
}

fn key(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        take_while(|c: char| c.is_ascii_alphanumeric() || c == '-'),
        opt(delimited(
            char('['),
            take_while1(|c: char| c != ']'),
            char(']'),
        ))
    )))(input)
}

fn entry(input: &str) -> IResult<&str, (&str, &str)> {
    terminated(
        separated_pair(
            key, tuple((space0, char('='), space0)), take_while(|c| c != '\n')
        ),
        eol
    )(input)
}

fn entries(input: &str) -> IResult<&str, HashMap<&str, &str>> {
    map(
        many0(
            alt((
                map(comment, |_| None),
                map(entry, |x| Some(x)),
                map(empty_line, |_| None),
            ))
        ),
        |kvs: Vec<_>| kvs.into_iter().flatten().collect()
    )(input)
}


fn group(input: &str) -> IResult<&str, (&str, HashMap<&str, &str>)> {
    pair(
        group_header,
        entries
    )(input)
}

fn groups(input: &str) -> IResult<&str, HashMap<&str, HashMap<&str, &str>>> {
    map(
        many0(
            alt((
                map(comment, |_| None),
                map(group, |x| Some(x)),
                map(empty_line, |_| None),
            ))
        ),
        |grps: Vec<_>| grps.into_iter().flatten().collect()
    )(input)
}

pub fn parse_desktop_entry(input: &str) -> ParseResult<HashMap<&str, HashMap<&str, &str>>> {
    match all_consuming(groups)(input) {
        Err(_) => Err(ParseError::InvalidLine("".into())),
        Ok((_, res)) => Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{Err,error::ErrorKind};

    #[test]
    fn test_parse_comment() {
        assert_eq!(Ok(("Y", " abc")), comment("# abc\nY"));
        assert_eq!(Ok(("Y", " abc def ## []")), comment("# abc def ## []\nY"));
        assert_eq!(Ok(("Y", "## abc ###   ")), comment("### abc ###   \nY"));
        assert_eq!(Ok(("Y", "## abc ###   ")), comment("### abc ###   \nY"));

        assert_eq!(comment("  \nY"), Err(Err::Error(error_position!("  \nY", ErrorKind::Char))));
        assert_matches!(comment("  #\nY".into()), Err(Err::Error(_)));
        assert_matches!(comment("[#abc]\nY".into()), Err(Err::Error(_)));
    }

    #[test]
    fn test_parse_group_header() {
        assert_eq!(Ok(("Y", "a")), group_header("[a]\nY"));
        assert_eq!(Ok(("Y", "abc def")), group_header("[abc def]\nY"));
        // assert_eq!(Ok(("", "a")), group_header("[a]    \t\n"));

        assert_matches!(group_header(" [a]\nY"), Err(Err::Error(_)));
        assert_matches!(group_header("[\nY"), Err(Err::Failure(_)));
        assert_matches!(group_header("[a\nY"), Err(Err::Failure(_)));
        assert_matches!(group_header("[abc]def\nY"), Err(Err::Failure(_)));
    }

    #[test]
    fn test_parse_entry() {
        assert_eq!(Ok(("Y", ("abc", "def"))), entry("abc=def\nY"));

        // ignore space before and after =
        assert_eq!(Ok(("Y", ("abc", "def"))), entry("abc   = def\nY"));
        assert_eq!(Ok(("Y", ("abc", ""))), entry("abc =\nY"));
        assert_eq!(Ok(("Y", ("abc", "def  "))), entry("abc =  def  \nY"));

        // key
        assert_eq!(Ok(("Y", ("-a-b-c-", "def"))), entry("-a-b-c-=def\nY"));
        assert_eq!(Ok(("Y", ("ABC", "def"))), entry("ABC=def\nY"));
        assert_matches!(entry("a b=\nY"), Err(Err::Error(_)));
        assert_matches!(entry("[a=b]\nY"), Err(Err::Error(_)));

        // empty key
        assert_eq!(Ok(("Y", ("", "def"))), entry("=def\nY"));
        assert_eq!(Ok(("Y", ("", ""))), entry("=\nY"));
        assert_eq!(Ok(("Y", ("", ""))), entry("  =   \nY"));
    }

    #[test]
    fn test_parse_entries() {
        assert_eq!(
            Ok(("[group]\n", hashmap!(
                "abc" => "def",
                "def" => "abc"
            ))),
            entries(indoc!("
                abc=def
                def=abc

                [group]
            "))
        );

        // empty lines
        assert_eq!(
            Ok(("[group]\n", hashmap!(
                "abc" => "def",
                "def" => "abc"
            ))),
            entries(indoc!("
                abc=def

                \t\t
                def=abc

                [group]
            "))
        );

        // comments
        assert_eq!(
            Ok(("[group]\n", hashmap!(
                "abc" => "def",
                "def" => "abc"
            ))),
            entries(indoc!("
                abc=def
                # this is a comment
                def=abc

                [group]
            "))
        );
    }

    #[test]
    fn test_parse_desktop_entry_comments() {
        assert_eq!(
            Ok(hashmap!(
                "Desktop Entry" => hashmap!(
                    "Version" => "1.0",
                ),
                "Desktop Action Gallery" => hashmap!(
                    "Exec" => "fooview --gallery",
                )
            )),
            parse_desktop_entry(indoc!("

                # Copyright by
                #          Dr. Who

                [Desktop Entry]

                # app version

                Version=1.0

                ## Actions

                [Desktop Action Gallery]
                Exec=fooview --gallery

                # Last Line"))
        );
    }

    #[test]
    fn test_parse_desktop_entry_non_empty_last_line() {
        assert_eq!(
            Ok(hashmap!(
                "Desktop Entry" => hashmap!("Version" => "1.0")
            )),
            parse_desktop_entry("[Desktop Entry]\nVersion=1.0")
        );

        assert_eq!(
            Ok(hashmap!(
                "Desktop Entry" => hashmap!()
            )),
            parse_desktop_entry("[Desktop Entry]")
        );
    }

    #[test]
    fn test_parse_desktop_entry_spec_example() {
        assert_eq!(
            Ok(hashmap!(
                "Desktop Entry" => hashmap!(
                    "Version" => "1.0",
                    "Type" => "Application",
                    "Name" => "Foo Viewer",
                    "Comment" => "The best viewer for Foo objects available!",
                    "TryExec" => "fooview",
                    "Exec" => "fooview %F",
                    "Icon" => "fooview",
                    "MimeType" => "image/x-foo;",
                    "Actions" => "Gallery;Create;"
                ),
                "Desktop Action Gallery" => hashmap!(
                    "Exec" => "fooview --gallery",
                    "Name" => "Browse Gallery"
                ),
                "Desktop Action Create" => hashmap!(
                    "Exec" => "fooview --create-new",
                    "Name" => "Create a new Foo!",
                    "Icon" => "fooview-new"
                ),
            )),
            parse_desktop_entry(indoc!("
                [Desktop Entry]
                Version=1.0
                Type=Application
                Name=Foo Viewer
                Comment=The best viewer for Foo objects available!
                TryExec=fooview
                Exec=fooview %F
                Icon=fooview
                MimeType=image/x-foo;
                Actions=Gallery;Create;

                [Desktop Action Gallery]
                Exec=fooview --gallery
                Name=Browse Gallery

                [Desktop Action Create]
                Exec=fooview --create-new
                Name=Create a new Foo!
                Icon=fooview-new
            "))
        );
    }
}