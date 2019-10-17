#[macro_use(eof, error_position)] extern crate nom;
#[cfg(test)] #[macro_use] extern crate assert_matches;
#[cfg(test)] #[macro_use] extern crate indoc;
#[cfg(test)] #[macro_use] extern crate maplit;

mod parser;
mod errors;
mod desktop_entry;
mod locale;

pub use errors::{ParseError, ParseResult};
pub use desktop_entry::{DesktopEntry, StandardKey};