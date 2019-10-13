use nom::{
    IResult,
    bytes::complete::take_while,
    character::complete::char,
    sequence::{separated_pair, tuple, preceded, terminated, pair},
    character::complete::{space0},
    multi::{separated_list},
    branch::alt,
    combinator::{map, peek, cut},
};
use std::collections::HashMap;

fn eof(input: &str) -> IResult<&str, ()> {
    eof!(input,)?;
    Ok((input, ()))
}

fn eol(input: &str) -> IResult<&str, ()> {
    map(char('\n'), |_| ())(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    preceded(char('#'), take_while(|c| c != '\n'))(input)
}

fn group_header(input: &str) -> IResult<&str, &str> {
    preceded(
        char('['),
        cut(terminated(
            take_while(|c| c != ']'),
            pair(
                char(']'),
                alt((peek(eol), eof))
            )
        ))
    )(input)
}

fn key(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii_alphanumeric() || c == '-')(input)
}

fn kv_sep(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii_alphanumeric() || c == '-')(input)
}

fn entry(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(
        key, tuple((space0, char('='), space0)), take_while(|c| c != '\n')
    )(input)
}

fn entries(input: &str) -> IResult<&str, HashMap<&str, &str>> {
    map(
        separated_list(
            char('\n'),
            cut(alt((
                map(comment, |_| None),
                map(entry, |x| Some(x)),
                map(space0, |_| None),
            )))
        ),
        |kvs: Vec<_>| kvs.iter().filter_map(|&x| x).collect()
    )(input)
}
/*
named!(comment<CompleteStr, Line>,
    do_parse!(
        char!('#') >>
        take_while1!(|_| true) >>
        (Line::Ignore)
    )
);

named!(groupheader<CompleteStr, Line>,
    exact!(do_parse!(
        char!('[') >>
        groupname: take_while!(|c| c != ']') >>
        return_error!(ErrorKind::Custom(ERROR_INCOMPLETE_GROUPNAME), char!(']')) >>
        (Line::Group(groupname.as_ref()))
    ))
);




named!(line<CompleteStr, Line>,
    alt!(comment | groupheader | keyvalue)
);*/


/*
fn parse_line(input: &str) -> ParseResult<Line> {
    match line(input.trim().into()) {
        Ok((_rest, result)) => Ok(result),
        Err(_) => Err(ParseError::InvalidLine(input.into())),
    }
}

fn parse_file(input: &str) -> ParseResult<HashMap<&str, HashMap<&str, &str>>> {
    use Line::*;
    let mut cur_group: &str = &"";
    let mut result: HashMap<&str, HashMap<&str, &str>> = HashMap::new();

    for line in input.lines() {
        match parse_line(line)? {
            Ignore => {},
            Group(group) => {
                assert!(!group.is_empty());
                assert!(!result.contains_key(group));
                cur_group = group;
                result.insert(group, HashMap::new());
            },
            KeyValue(key, value) => {
                assert!(!cur_group.is_empty());
                assert!(!result[cur_group].contains_key(key));
                result.get_mut(cur_group).unwrap().insert(key, value);
            },
        }
    }

    Ok(result)
}*/

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{Err,error::ErrorKind};

    #[test]
    fn test_parse_comment() {
        assert_eq!(Ok(("\nY", " abc")), comment("# abc\nY"));
        assert_eq!(Ok(("\nY", " abc def ## []")), comment("# abc def ## []\nY"));
        assert_eq!(Ok(("\nY", "## abc ###   ")), comment("### abc ###   \nY"));
        assert_eq!(Ok(("\nY", "## abc ###   ")), comment("### abc ###   \nY"));

        assert_eq!(comment("  \nY"), Err(Err::Error(error_position!("  \nY", ErrorKind::Char))));
        assert_matches!(comment("  #\nY".into()), Err(Err::Error(_)));
        assert_matches!(comment("[#abc]\nY".into()), Err(Err::Error(_)));
    }

    #[test]
    fn test_parse_group_header() {
        assert_eq!(Ok(("\nY", "a")), group_header("[a]\nY"));
        assert_eq!(Ok(("\nY", "abc def")), group_header("[abc def]\nY"));
        // assert_eq!(Ok(("", "a")), group_header("[a]    \t\n"));

        assert_matches!(group_header(" [a]\nY"), Err(Err::Error(_)));
        assert_matches!(group_header("[\nY"), Err(Err::Failure(_)));
        assert_matches!(group_header("[a\nY"), Err(Err::Failure(_)));
        assert_matches!(group_header("[abc]def\nY"), Err(Err::Failure(_)));
    }

    #[test]
    fn test_parse_entry() {
        assert_eq!(Ok(("\nY", ("abc", "def"))), entry("abc=def\nY"));

        // ignore space before and after =
        assert_eq!(Ok(("\nY", ("abc", "def"))), entry("abc   = def\nY"));
        assert_eq!(Ok(("\nY", ("abc", ""))), entry("abc =\nY"));
        assert_eq!(Ok(("\nY", ("abc", "def  "))), entry("abc =  def  \nY"));

        // key
        assert_eq!(Ok(("\nY", ("-a-b-c-", "def"))), entry("-a-b-c-=def\nY"));
        assert_eq!(Ok(("\nY", ("ABC", "def"))), entry("ABC=def\nY"));
        assert_matches!(entry("a b=\nY"), Err(Err::Error(_)));
        assert_matches!(entry("[a=b]\nY"), Err(Err::Error(_)));

        // empty key
        assert_eq!(Ok(("\nY", ("", "def"))), entry("=def\nY"));
        assert_eq!(Ok(("\nY", ("", ""))), entry("=\nY"));
        assert_eq!(Ok(("\nY", ("", ""))), entry("  =   \nY"));
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
}